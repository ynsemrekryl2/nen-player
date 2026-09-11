import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// `NEN-044`: the macOS adapter's own text extraction, over
/// `libavformat`/`libavcodec` (ADR-0045). `ContractTests.
/// theRealAdapterPassesTheSharedContractKit` already proves the shared kit's
/// two `ExtractText` scenarios pass now that the capability is declared; this
/// file is the golden text, the mandatory bitmap negative, and the two typed
/// refusals the kit does not cover.
struct EmbeddedTextExtractionTests {
    private static func golden(_ name: String) throws -> String {
        let path = ContractTests.repositoryRoot.appendingPathComponent("fixtures/media/\(name)").path
        return try String(contentsOfFile: path, encoding: .utf8)
    }

    // MARK: - Golden — the extracted text matches the source SRT exactly

    @Test func theEnglishTrackExtractsToItsGoldenText() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        let text = try engine.extractText(track: 3)
        #expect(text == (try Self.golden("contract-clip.sub-eng.golden")))
    }

    @Test func theTurkishTrackExtractsToItsGoldenText() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        let text = try engine.extractText(track: 4)
        #expect(text == (try Self.golden("contract-clip.sub-tur.golden")))
    }

    // MARK: - Negative (mandatory, ADR-0045 Karar 3) — a bitmap track refuses

    @Test func aBitmapTrackIsATypedRefusalNotEmptyText() throws {
        let engine = try TrackEnumerationTests.load("bitmap-subs-clip.mkv")
        defer { try? engine.shutdown() }

        #expect(throws: FfiPlaybackError.TrackCarriesNoText) {
            _ = try engine.extractText(track: 3)
        }
        // The sibling text track in the *same* file still extracts — the
        // refusal above is about the track, not something wrong with the
        // container or the codec lookup in general.
        let text = try engine.extractText(track: 2)
        #expect(text.contains("First English line."))
        #expect(text.contains("Second English line."))
    }

    // MARK: - Typed refusals outside the shared kit's fixture

    @Test func anUnknownTrackIsATypedRefusal() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }

        #expect(throws: FfiPlaybackError.UnknownTrack(kind: .subtitle)) {
            _ = try engine.extractText(track: 9_999)
        }
    }

    @Test func extractingBeforeAnyMediumIsLoadedRefuses() throws {
        let engine = try MPVPlaybackEngine()
        #expect(throws: FfiPlaybackError.NotLoaded) {
            _ = try engine.extractText(track: 3)
        }
    }

    /// **Local files only** (`NEN-109`, backlog, owns the remote case):
    /// reading an entire remote container to demux one stream has no
    /// cancellation and no bound on how much it would download. This is
    /// checked before `libavformat` ever opens anything, which is what a
    /// deterministic test can prove without a real remote resource — the
    /// locator this loaded medium answers to is overwritten by hand, the
    /// same track id that extracts happily above.
    @Test func aRemoteLocatorIsUnsupportedNotAttempted() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }
        engine.currentLocator = "https://example.com/stream.mkv"

        #expect(throws: FfiPlaybackError.Unsupported(capability: .embeddedTextExtraction)) {
            _ = try engine.extractText(track: 3)
        }
    }
}
