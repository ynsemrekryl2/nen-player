import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-027's evidence on the real engine: a user's subtitle file is drawn, and
/// the cue on screen after a seek is the one the document says it should be.
///
/// The Rust side already sweeps 4 000 moments against `CueIndex` through a
/// reference engine (`core/crates/nen-app/tests/subtitle_rendering.rs`). What
/// only this side can show is that **libmpv** draws it: the document reaches
/// mpv over `memory://`, mpv puts a line on the screen, and the line it puts
/// there is the core's answer for the moment mpv is actually at.
///
/// **Security (K23 #4).** The cues below are invented and the assertions never
/// print them: a mismatch reports the moment, not the dialogue.
struct SubtitleRenderingTests {
    // MARK: - Fixture

    /// A cue every 2 s: `[i·2000 + 300, i·2000 + 1500)`, so the clip's 30 s
    /// holds fourteen of them with a 800 ms gap after each.
    static let cuePeriodMs: UInt64 = 2_000
    static let cueStartOffsetMs: UInt64 = 300
    static let cueLengthMs: UInt64 = 1_200
    static let cueCount: UInt64 = 14

    /// How far a sampled moment stays from any cue boundary.
    ///
    /// The fixture clip runs at 5 fps, so one frame is 200 ms and the frame mpv
    /// draws after an exact seek can only be judged to that resolution. Whether
    /// the millisecond *at* a boundary belongs to the cue before or after is
    /// pinned exactly, 4 000 times, in the Rust parity sweep; this test is
    /// about which cue reaches the screen, so it samples away from the edges
    /// rather than pretending a frame is a millisecond.
    static let boundaryMarginMs: UInt64 = 250

    static func srt() -> String {
        func stamp(_ ms: UInt64) -> String {
            String(
                format: "%02d:%02d:%02d,%03d",
                ms / 3_600_000, (ms / 60_000) % 60, (ms / 1_000) % 60, ms % 1_000
            )
        }
        var out = ""
        for index in 0..<cueCount {
            let start = index * cuePeriodMs + cueStartOffsetMs
            // One two-line cue and one non-ASCII cue, so the comparison covers
            // both the line joining and the UTF-8 round trip through
            // `memory://` rather than only plain single lines.
            let body: String
            switch index {
            case 3: body = "satır bir\nsatır iki"
            case 7: body = "Merhaba dünya"
            default: body = "cue \(index)"
            }
            out += "\(index + 1)\n\(stamp(start)) --> \(stamp(start + cueLengthMs))\n\(body)\n\n"
        }
        return out
    }

    /// A directory holding a copy of the fixture clip and a sidecar beside it.
    struct Neighbourhood {
        let directory: URL
        let mediumPath: String
        let sidecarPath: String

        init() throws {
            directory = URL(fileURLWithPath: NSTemporaryDirectory(), isDirectory: true)
                .appendingPathComponent("nen-027-\(UUID().uuidString)", isDirectory: true)
            try FileManager.default.createDirectory(
                at: directory, withIntermediateDirectories: true
            )
            let medium = directory.appendingPathComponent("Clip.mkv")
            try FileManager.default.copyItem(
                at: URL(fileURLWithPath: ContractTests.fixturePath("contract-clip.mkv")),
                to: medium
            )
            let sidecar = directory.appendingPathComponent("Clip.tr.srt")
            try SubtitleRenderingTests.srt().write(to: sidecar, atomically: true, encoding: .utf8)
            mediumPath = medium.path
            sidecarPath = sidecar.path
        }

        func remove() {
            try? FileManager.default.removeItem(at: directory)
        }
    }

    /// The library, and the token naming the user's file in it.
    private func library(for neighbourhood: Neighbourhood) throws -> (FfiSubtitleLibrary, UInt32) {
        let library = FfiSubtitleLibrary()
        #expect(library.addFile(path: neighbourhood.sidecarPath) == .added)
        let token = try #require(
            library.menu(primary: nil, secondary: nil)
                .flatMap(\.entries)
                .first { $0.defect == nil && $0.kind == .user }?
                .token,
            "the sidecar has to be a usable row, or the test measures the gate instead"
        )
        return (library, token)
    }

    private func loadedSession(_ neighbourhood: Neighbourhood) throws -> FfiPlaybackSession {
        let session = FfiPlaybackSession(engine: try MPVPlaybackEngine())
        try session.load(locator: neighbourhood.mediumPath)
        try waitUntil("the medium is ready") { (try? session.state()) == .ready }
        return session
    }

    // MARK: - The DoD

    @Test func aUserFileIsDrawnAndTheCueMatchesTheCoreAfterEverySeek() throws {
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let session = try loadedSession(neighbourhood)
        defer { try? session.shutdown() }
        let (library, token) = try library(for: neighbourhood)

        #expect(try session.showSubtitle(library: library, token: token) == .shown)

        var drawnMoments = 0
        var blankMoments = 0

        for step in 0..<Int(Self.cueCount * 2) {
            let index = UInt64(step / 2)
            let insideACue = step % 2 == 0
            let moment = Self.moment(inCue: index, inside: insideACue)

            try session.seek(toMs: moment)
            try waitUntil("the seek to \(moment) ms lands") {
                (try? session.positionMs()).map { $0.absoluteDistance(to: moment) < 120 } ?? false
            }

            // Judged at the position the engine is **at**, not the one it was
            // asked for: the comparison is about the cue on screen, and a seek
            // that landed a few milliseconds away is still a moment the
            // document has an answer for.
            let landed = try session.positionMs()
            let expected = session.expectedSubtitleText(atMs: landed)
            var drawn: String?
            try waitUntil("mpv settles on a subtitle for \(landed) ms") {
                drawn = try? session.renderedSubtitleText()
                return drawn == expected
            }

            let complaint: Comment = """
                at \(landed) ms the screen and the document disagree \
                (drawn: \(drawn == nil ? "nothing" : "a cue"), \
                expected: \(expected == nil ? "nothing" : "a cue"))
                """
            #expect(drawn == expected, complaint)
            if expected == nil { blankMoments += 1 } else { drawnMoments += 1 }
        }

        // Both halves of the DoD have to have happened: a run that only ever
        // landed inside cues would say nothing about the empty moment.
        #expect(drawnMoments == Int(Self.cueCount))
        #expect(blankMoments == Int(Self.cueCount))
    }

    @Test func aUserFileReplacesAnEmbeddedTrackThatIsAlreadyDrawing() throws {
        // The shell's real order, which the tests above skip: automatic
        // selection opens an embedded track first (ADR-0010 Karar 9), and the
        // user picks their own file afterwards. What must not happen is the
        // file arriving on a surface that is still drawing the track.
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let session = try loadedSession(neighbourhood)
        defer { try? session.shutdown() }
        let (library, token) = try library(for: neighbourhood)

        // The fixture clip's own subtitle track, selected the way the shell
        // selects one.
        try session.selectTrack(kind: .subtitle, track: 3)
        try session.seek(toMs: 2_000)
        try waitUntil("the embedded track is drawing") {
            ((try? session.renderedSubtitleText()) ?? nil) != nil
        }

        #expect(try session.showSubtitle(library: library, token: token) == .shown)

        let moment = Self.moment(inCue: 2, inside: true)
        try session.seek(toMs: moment)
        try waitUntil("the seek lands") {
            (try? session.positionMs()).map { $0.absoluteDistance(to: moment) < 120 } ?? false
        }
        let landed = try session.positionMs()
        let expected = session.expectedSubtitleText(atMs: landed)
        var drawn: String?
        try waitUntil("the user's own cue is on screen at \(landed) ms") {
            drawn = try? session.renderedSubtitleText()
            return drawn == expected
        }
        #expect(drawn == expected)
        #expect(expected != nil, "the moment has to have a cue, or this proves nothing")
    }

    @Test func turningSubtitlesOffTakesTheDocumentOffScreen() throws {
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let session = try loadedSession(neighbourhood)
        defer { try? session.shutdown() }
        let (library, token) = try library(for: neighbourhood)

        #expect(try session.showSubtitle(library: library, token: token) == .shown)
        try session.seek(toMs: Self.moment(inCue: 2, inside: true))
        try waitUntil("a cue is on screen") { (try? session.renderedSubtitleText()) ?? nil != nil }

        try session.hideSubtitle()

        try waitUntil("the screen goes blank") {
            ((try? session.renderedSubtitleText()) ?? nil) == nil
        }
        #expect(session.expectedSubtitleText(atMs: Self.moment(inCue: 2, inside: true)) == nil)
    }

    // MARK: - What injection must not do to the medium

    @Test func anInjectedDocumentIsNotOneOfTheMediumsTracks() throws {
        // ADR-0013 Karar 5, and the reason it is not cosmetic: mpv gives an
        // external track `ff-index = 0`, which is the *video* track's number
        // and collides with the catalog's own ids. A document that showed up
        // here would be listed as an embedded source nobody catalogued —
        // NEN-058's defect, made by our own hand.
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: neighbourhood.mediumPath)
        try waitUntil("the medium is ready") { engine.state() == .ready }
        let before = try engine.tracks(kind: .subtitle).map(\.id)
        #expect(before == [3, 4], "the fixture's own subtitle tracks")

        try engine.injectSubtitle(webvtt: Self.webvtt)

        #expect(try engine.tracks(kind: .subtitle).map(\.id) == before)
        #expect(try engine.tracks(kind: .audio).map(\.id) == [1, 2])
    }

    @Test func showingASecondDocumentReplacesTheFirstRatherThanStacking() throws {
        // `sub-add` appends. Without the removal, every subtitle the user picks
        // would leave the previous one loaded for the rest of the medium.
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: neighbourhood.mediumPath)
        try waitUntil("the medium is ready") { engine.state() == .ready }
        let before = try engine.int("track-list/count")

        for _ in 0..<3 {
            try engine.injectSubtitle(webvtt: Self.webvtt)
        }

        #expect(
            try engine.int("track-list/count") == before + 1,
            "one document is loaded at a time, whatever the user clicks"
        )
    }

    @Test func anInjectedDocumentSurvivesNothingOfThePreviousMedium() throws {
        // mpv drops external subtitles with the outgoing file, so the id this
        // adapter holds stops meaning anything at that moment. Injecting again
        // after a reload must not try to remove a track that belongs to
        // whatever mpv has numbered since.
        let neighbourhood = try Neighbourhood()
        defer { neighbourhood.remove() }
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: neighbourhood.mediumPath)
        try waitUntil("the medium is ready") { engine.state() == .ready }
        try engine.injectSubtitle(webvtt: Self.webvtt)

        try engine.load(locator: ContractTests.fixturePath("contract-clip.mkv"))
        try waitUntil("the second medium is ready") { engine.state() == .ready }
        let before = try engine.int("track-list/count")
        try engine.injectSubtitle(webvtt: Self.webvtt)

        #expect(try engine.int("track-list/count") == before + 1)
        #expect(try engine.tracks(kind: .subtitle).map(\.id) == [3, 4])
    }

    // MARK: - Helpers

    /// A moment inside cue `index`, or in the gap that follows it, at least
    /// ``boundaryMarginMs`` away from either edge.
    static func moment(inCue index: UInt64, inside: Bool) -> UInt64 {
        let start = index * cuePeriodMs + cueStartOffsetMs
        return inside
            ? start + boundaryMarginMs
            : start + cueLengthMs + boundaryMarginMs
    }

    /// A one-cue document, for the tests that only need *a* document.
    static let webvtt = """
        WEBVTT

        1
        00:00:00.600 --> 00:00:01.500
        injected

        """

    private func waitUntil(
        _ what: String,
        timeout: TimeInterval = 5,
        _ condition: () -> Bool
    ) throws {
        let deadline = Date().addingTimeInterval(timeout)
        while Date() < deadline {
            if condition() { return }
            Thread.sleep(forTimeInterval: 0.005)
        }
        Issue.record("never happened: \(what)")
    }
}

private extension UInt64 {
    func absoluteDistance(to other: UInt64) -> UInt64 {
        self > other ? self - other : other - self
    }
}
