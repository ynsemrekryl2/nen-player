import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-058's evidence: opening one medium after another, and what the engine
/// does with whatever sits beside them.
///
/// The symptom NEN-025 reported was a medium that refused to open when a
/// **symlinked** `.srt` sat next to it. Measured here, the sidecar turned out
/// to be a bystander: no shape of symlink — sibling target, relative, dangling,
/// outside the directory, a directory, a loop — makes a medium fail. What
/// failed was the **second** medium opened in a session, whichever file it was,
/// because `loadfile` ends the outgoing entry and the adapter read that end as
/// the new file's failure. NEN-025's manual run opened `Probe.mkv` first and
/// `Linked.mkv` second, so the symlink and the order were confounded.
///
/// The scan did find a real second defect, and it is the one the hypothesis was
/// about: mpv's own `sub-auto` was opening the sidecar behind the catalog's
/// back.
///
/// Measured with libmpv, not the `mpv` CLI: NEN-022 recorded that
/// `mpv --frames=1` does not return on this machine while libmpv finishes the
/// same clip in half a second, and NEN-058's own opening attempt hung on the
/// control file too. The CLI is not a measuring instrument here.
struct SidecarLoadingTests {
    // MARK: - Neighbourhood

    /// A copy of a fixture in a fresh directory, with a chosen `.srt` beside it.
    ///
    /// The sidecar is written as a **real** symlink where one is asked for.
    /// A fake that merely claims to be one would say nothing about symlinks,
    /// which is the whole subject here — the same reason NEN-025 built real
    /// links, real FIFOs and real directory links for its gates.
    struct Neighbourhood {
        enum Sidecar {
            case none
            case plainFile
            case symbolicLink
        }

        let directory: URL
        let mediumPath: String

        /// `in:` puts a second medium beside the first, which is how NEN-025
        /// laid them out and the only way the order can be the variable.
        init(named name: String, sidecar: Sidecar, in existing: URL? = nil) throws {
            directory = existing ?? URL(
                fileURLWithPath: NSTemporaryDirectory(),
                isDirectory: true
            ).appendingPathComponent("nen-058-\(UUID().uuidString)", isDirectory: true)
            try FileManager.default.createDirectory(
                at: directory, withIntermediateDirectories: true
            )

            let medium = directory.appendingPathComponent("\(name).mkv")
            try FileManager.default.copyItem(
                at: URL(fileURLWithPath: ContractTests.fixturePath("contract-clip.mkv")),
                to: medium
            )
            mediumPath = medium.path

            let subtitle = directory.appendingPathComponent("\(name).srt")
            switch sidecar {
            case .none:
                break
            case .plainFile:
                try Self.subtitleText.write(to: subtitle, atomically: true, encoding: .utf8)
            case .symbolicLink:
                // The link target is a valid subtitle in the same directory, so
                // nothing about the *content* can explain a refusal.
                let target = directory.appendingPathComponent("\(name)-target.srt")
                try Self.subtitleText.write(to: target, atomically: true, encoding: .utf8)
                try FileManager.default.createSymbolicLink(at: subtitle, withDestinationURL: target)
            }
        }

        func remove() {
            try? FileManager.default.removeItem(at: directory)
        }

        static let subtitleText = """
        1
        00:00:01,000 --> 00:00:04,000
        Sidecar line.

        """
    }

    /// Loads a medium and waits for the engine to settle either way.
    ///
    /// `load` returns before the medium is open — the port requires it — so the
    /// answer is whatever state the engine reaches, `ready` or `failed`.
    static func settle(_ engine: MPVPlaybackEngine, locator: String) throws -> FfiPlaybackState {
        var ignored: [FfiPlaybackEvent] = []
        return try settle(engine, locator: locator, collecting: &ignored)
    }

    static func settle(
        _ engine: MPVPlaybackEngine,
        locator: String,
        collecting seen: inout [FfiPlaybackEvent]
    ) throws -> FfiPlaybackState {
        try engine.load(locator: locator)
        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline, engine.state() == .idle || engine.state() == .buffering {
            seen += engine.drainEvents()
            Thread.sleep(forTimeInterval: 0.01)
        }
        seen += engine.drainEvents()
        return engine.state()
    }

    // MARK: - DoD #2 — the medium opens, whatever sits beside it

    @Test func aMediumOpensBesideASymlinkedSidecar() throws {
        let neighbourhood = try Neighbourhood(named: "Linked", sidecar: .symbolicLink)
        defer { neighbourhood.remove() }

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: neighbourhood.mediumPath) == .ready)
    }

    /// The symptom, reproduced in the shape that actually produces it.
    ///
    /// Two byte-equal copies in one directory, opened through one engine in
    /// NEN-025's order: the plain-sidecar file first, the symlinked one second.
    /// The shell reuses a single session across `⌘O`, so this is the path a
    /// user takes.
    @Test func asecondMediumOpensWithoutReportingAFailure() throws {
        let first = try Neighbourhood(named: "Probe", sidecar: .plainFile)
        defer { first.remove() }
        let second = try Neighbourhood(
            named: "Linked", sidecar: .symbolicLink, in: first.directory
        )

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: first.mediumPath) == .ready)
        try engine.play()
        _ = engine.drainEvents()

        var seen: [FfiPlaybackEvent] = []
        #expect(try Self.settle(engine, locator: second.mediumPath, collecting: &seen) == .ready)

        // The state is not enough on its own: before the fix the engine still
        // reached `ready`, having passed through `failed` on the way — and the
        // shell shows a fatal message the moment it sees that, whatever comes
        // after.
        let failures = seen.filter { event in
            if case .failed = event { return true }
            if case let .stateChanged(state) = event { return state == .failed }
            return false
        }
        #expect(failures.isEmpty, "the outgoing medium's end was reported as this one's failure")
    }

    /// The reverse order, so the test above cannot pass by accident of which
    /// file is which. Neither file is the variable; the position is.
    @Test func theOrderOfTheTwoMediaDoesNotMatter() throws {
        let first = try Neighbourhood(named: "Linked", sidecar: .symbolicLink)
        defer { first.remove() }
        let second = try Neighbourhood(named: "Probe", sidecar: .plainFile, in: first.directory)

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: first.mediumPath) == .ready)
        try engine.play()
        _ = engine.drainEvents()

        var seen: [FfiPlaybackEvent] = []
        #expect(try Self.settle(engine, locator: second.mediumPath, collecting: &seen) == .ready)
        #expect(!seen.contains { if case .failed = $0 { return true } else { return false } })
    }

    /// A load that genuinely fails must still be reported, or the guard above
    /// would have bought silence by going deaf.
    @Test func aGenuineFailureIsStillReportedAfterAMediumIsOpen() throws {
        let neighbourhood = try Neighbourhood(named: "Probe", sidecar: .none)
        defer { neighbourhood.remove() }

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: neighbourhood.mediumPath) == .ready)
        try engine.play()
        _ = engine.drainEvents()

        var seen: [FfiPlaybackEvent] = []
        let broken = neighbourhood.directory.appendingPathComponent("nothing-here.mkv").path
        #expect(try Self.settle(engine, locator: broken, collecting: &seen) == .failed)
        #expect(seen.contains { if case .failed = $0 { return true } else { return false } })
    }

    // MARK: - DoD #4 — the engine loads no subtitle of its own

    @Test func aSidecarBesideTheMediumDoesNotEnterTheTrackList() throws {
        // The catalog is the only way a subtitle becomes a source (ADR-0031
        // Karar 4/5). A file the engine opened by itself would be a source
        // nothing catalogued, nothing showed in the menu, and none of NEN-025's
        // gates ever examined.
        //
        // The fixture's own tracks are measured constants — 2 audio, 2 subtitle
        // at ff-index 3 and 4 (`fixtures/media/contract-clip.ffmpeg.txt`) — so a
        // further track can only have come from beside the file. Measured before
        // the fix: `[3, 4, 0]`, the trailing 0 being an external track, which has
        // no ff-index of its own.
        let neighbourhood = try Neighbourhood(named: "Probe", sidecar: .plainFile)
        defer { neighbourhood.remove() }

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: neighbourhood.mediumPath) == .ready)
        #expect(try engine.tracks(kind: .subtitle).map(\.id) == [3, 4],
                "a subtitle the catalog never saw entered the list")
    }

    @Test func aSymlinkedSidecarDoesNotEnterTheTrackListEither() throws {
        // The gate NEN-025 built refuses a symlinked subtitle outright. If mpv
        // opened one on its own it would be smuggling past exactly that gate.
        let neighbourhood = try Neighbourhood(named: "Linked", sidecar: .symbolicLink)
        defer { neighbourhood.remove() }

        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        #expect(try Self.settle(engine, locator: neighbourhood.mediumPath) == .ready)
        #expect(try engine.tracks(kind: .subtitle).map(\.id) == [3, 4])
    }
}
