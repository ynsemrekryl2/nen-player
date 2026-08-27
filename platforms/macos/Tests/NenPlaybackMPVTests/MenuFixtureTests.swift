import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// The menu fixture on the real engine (NEN-026).
///
/// `menu-clip.mkv` exists so the menu's evidence can show §8's own example —
/// English, Français, Türkçe and a track with no language at all. Everything
/// asserted here was **measured** against the committed fixture, not assumed,
/// and it is measured because selecting a row in the menu turned out to be the
/// one step nothing had exercised end to end.
struct MenuFixtureTests {
    @Test func theMenuFixtureReportsFourSubtitleTracks() throws {
        let engine = try TrackEnumerationTests.load("menu-clip.mkv")
        defer { try? engine.shutdown() }

        let subtitles = try engine.tracks(kind: .subtitle)
        #expect(subtitles.map(\.language) == ["eng", "fre", "tur", nil])
        #expect(subtitles.map(\.title) == ["English", "Français", "Türkçe", nil])
        #expect(subtitles.allSatisfy { $0.codec == "subrip" })
    }

    /// Every row the menu can offer really selects its own track.
    ///
    /// The shell hands back an `ff-index` and the adapter maps it to mpv's
    /// per-kind `sid`; the two numbering schemes differ, so a mapping that is
    /// right for one track is not automatically right for the next. Reading the
    /// selection back is what makes this an assertion rather than a hope.
    @Test func everySubtitleTrackCanBeSelectedAndReadsBack() throws {
        let engine = try TrackEnumerationTests.load("menu-clip.mkv")
        defer { try? engine.shutdown() }

        let subtitles = try engine.tracks(kind: .subtitle)
        #expect(subtitles.count == 4)

        for track in subtitles {
            try engine.selectTrack(kind: .subtitle, track: track.id)
            #expect(
                try engine.selectedTrack(kind: .subtitle) == track.id,
                "ff-index \(track.id) did not come back as the selected track"
            )
        }

        try engine.selectTrack(kind: .subtitle, track: nil)
        #expect(try engine.selectedTrack(kind: .subtitle) == nil, "Kapalı really turns it off")
    }

    /// Nothing is shown until the core asks for it (NEN-058).
    @Test func nothingIsSelectedBeforeTheCoreChooses() throws {
        let engine = try TrackEnumerationTests.load("menu-clip.mkv")
        defer { try? engine.shutdown() }
        #expect(try engine.selectedTrack(kind: .subtitle) == nil)
    }
}
