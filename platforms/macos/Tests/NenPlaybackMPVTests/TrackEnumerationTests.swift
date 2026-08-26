import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-023's evidence on the real engine: what a container actually holds, as
/// the adapter reports it.
///
/// Every number and string below was **measured** against the committed
/// fixtures with libmpv 2.5.0 on 2026-08-26, not assumed. The ids are
/// `ff-index` values: mpv numbers tracks per kind, so audio and subtitle would
/// both start at 1, while the port's ids are one space across kinds.
struct TrackEnumerationTests {
    /// Loads a fixture and waits until the engine has a track list.
    ///
    /// `load` returns before the medium is open — the port requires that, and
    /// M3's first exit criterion depends on it — so a test that asked for
    /// tracks immediately would be asking too early.
    static func load(_ name: String) throws -> MPVPlaybackEngine {
        let engine = try MPVPlaybackEngine()
        try engine.load(locator: ContractTests.fixturePath(name))

        let deadline = Date().addingTimeInterval(10)
        while Date() < deadline, engine.state() == .idle || engine.state() == .buffering {
            _ = engine.drainEvents()
            Thread.sleep(forTimeInterval: 0.01)
        }
        _ = engine.drainEvents()
        #expect(engine.state() == .ready, "\(name) never became ready")
        return engine
    }

    // MARK: - DoD #1 — the list is right

    @Test func aMultiTrackContainerReportsEveryTrackOfEachKind() throws {
        let engine = try Self.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        let audio = try engine.tracks(kind: .audio)
        #expect(audio.map(\.id) == [1, 2])
        #expect(audio.map(\.language) == ["eng", "tur"])
        #expect(audio.allSatisfy { $0.codec == "opus" })
        #expect(audio.map(\.isDefault) == [true, false])

        let subtitles = try engine.tracks(kind: .subtitle)
        #expect(subtitles.map(\.id) == [3, 4])
        #expect(subtitles.map(\.language) == ["eng", "tur"])
        #expect(subtitles.allSatisfy { $0.codec == "subrip" })
        #expect(subtitles.map(\.isDefault) == [true, false])
    }

    @Test func theVideoTrackIsNeitherAudioNorSubtitle() throws {
        // The fixture has one. A container invents track types freely, and
        // anything the port has no kind for must simply not appear.
        let engine = try Self.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        let reported = try engine.tracks(kind: .audio).count + engine.tracks(kind: .subtitle).count
        #expect(reported == 4, "the video track leaked into a kind")
    }

    @Test func aTrackTitleCrossesWhenTheContainerDeclaresOne() throws {
        // The menu's label (§8). `contract-clip` declares none, so both cases
        // are covered by the two fixtures.
        let bitmapClip = try Self.load("bitmap-subs-clip.mkv")
        defer { try? bitmapClip.shutdown() }
        let titled = try bitmapClip.tracks(kind: .subtitle).first { $0.id == 2 }
        #expect(titled?.title == "English")

        let plain = try Self.load("contract-clip.mkv")
        defer { try? plain.shutdown() }
        #expect(try plain.tracks(kind: .subtitle).allSatisfy { $0.title == nil })
    }

    // MARK: - DoD #3 — a bitmap track is recognizable as one

    @Test func aBitmapTrackIsReportedByItsCodec() throws {
        let engine = try Self.load("bitmap-subs-clip.mkv")
        defer { try? engine.shutdown() }

        let subtitles = try engine.tracks(kind: .subtitle)
        #expect(subtitles.map(\.id) == [2, 3])

        // The adapter classifies nothing: it reports the codec, and
        // `nen-ports::subtitle_carries_text` decides what that means, once,
        // for every platform. These two strings are the hinge that decision
        // turns on, so they are what this test pins.
        #expect(subtitles.map(\.codec) == ["subrip", "hdmv_pgs_subtitle"])
        #expect(subtitles.map(\.language) == ["eng", "fre"])
    }

    // MARK: - DoD #2 — selection reaches playback

    @Test func everySubtitleTrackCanBeSelectedAndReadBack() throws {
        let engine = try Self.load("bitmap-subs-clip.mkv")
        defer { try? engine.shutdown() }

        // Including the bitmap one: §7 says such a track may be shown, so
        // "untranslatable" must not have become "unselectable" anywhere.
        for id in try engine.tracks(kind: .subtitle).map(\.id) {
            try engine.selectTrack(kind: .subtitle, track: id)
            #expect(try engine.selectedTrack(kind: .subtitle) == id)
        }
    }

    @Test func selectingNothingIsWhatTheClosedEntryDoes() throws {
        let engine = try Self.load("bitmap-subs-clip.mkv")
        defer { try? engine.shutdown() }

        try engine.selectTrack(kind: .subtitle, track: 2)
        #expect(try engine.selectedTrack(kind: .subtitle) == 2)

        try engine.selectTrack(kind: .subtitle, track: nil)
        #expect(try engine.selectedTrack(kind: .subtitle) == nil)
    }

    @Test func audioAndSubtitleSelectionsDoNotDisturbEachOther() throws {
        // The ids share one space across kinds, so a mix-up here would be
        // invisible in either kind on its own.
        let engine = try Self.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        try engine.selectTrack(kind: .audio, track: 2)
        try engine.selectTrack(kind: .subtitle, track: 3)

        #expect(try engine.selectedTrack(kind: .audio) == 2)
        #expect(try engine.selectedTrack(kind: .subtitle) == 3)
    }
}
