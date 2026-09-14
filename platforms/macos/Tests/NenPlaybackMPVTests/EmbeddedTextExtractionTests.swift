import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// A deterministic HTTP stream for the remote extraction contract. The
/// adapter still uses its production URLSessionDataDelegate path; this only
/// replaces the network transport so tests never contact a provider.
private final class EmbeddedTextStubURLProtocol: URLProtocol, @unchecked Sendable {
    enum Mode: Sendable {
        case reply(statusCode: Int, headers: [String: String], body: Data)
        case hang
    }

    nonisolated(unsafe) static var mode: Mode = .reply(statusCode: 200, headers: [:], body: Data())
    nonisolated(unsafe) static var lastRange: String?
    nonisolated(unsafe) static var started = DispatchSemaphore(value: 0)
    nonisolated(unsafe) static var stopped = DispatchSemaphore(value: 0)
    nonisolated(unsafe) static var release = DispatchSemaphore(value: 0)
    nonisolated(unsafe) static var wasStopped = false

    static func reset() {
        mode = .reply(statusCode: 200, headers: [:], body: Data())
        lastRange = nil
        started = DispatchSemaphore(value: 0)
        stopped = DispatchSemaphore(value: 0)
        release = DispatchSemaphore(value: 0)
        wasStopped = false
    }

    override class func canInit(with request: URLRequest) -> Bool { true }
    override class func canonicalRequest(for request: URLRequest) -> URLRequest { request }

    override func startLoading() {
        Self.lastRange = request.value(forHTTPHeaderField: "Range")
        switch Self.mode {
        case let .reply(statusCode, headers, body):
            guard let response = HTTPURLResponse(
                url: request.url ?? URL(string: "https://invalid")!,
                statusCode: statusCode,
                httpVersion: nil,
                headerFields: headers
            ) else { return }
            client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
            if !body.isEmpty { client?.urlProtocol(self, didLoad: body) }
            client?.urlProtocolDidFinishLoading(self)
        case .hang:
            // Do not block URLProtocol's startLoading callback: cancellation
            // must be able to reach stopLoading on another URLSession turn.
            DispatchQueue.global(qos: .utility).async { [weak self] in
                guard let self else { return }
                guard let response = HTTPURLResponse(
                    url: self.request.url ?? URL(string: "https://invalid")!,
                    statusCode: 200,
                    httpVersion: nil,
                    headerFields: [:]
                ) else { return }
                self.client?.urlProtocol(self, didReceive: response, cacheStoragePolicy: .notAllowed)
                self.client?.urlProtocol(self, didLoad: Data([0]))
                Self.started.signal()
                Self.release.wait()
                if !Self.wasStopped { self.client?.urlProtocolDidFinishLoading(self) }
            }
        }
    }

    override func stopLoading() {
        Self.wasStopped = true
        Self.stopped.signal()
        Self.release.signal()
    }
}

/// `NEN-044`: the macOS adapter's own text extraction, over
/// `libavformat`/`libavcodec` (ADR-0045). `ContractTests.
/// theRealAdapterPassesTheSharedContractKit` already proves the shared kit's
/// two `ExtractText` scenarios pass now that the capability is declared; this
/// file is the golden text, the mandatory bitmap negative, and the two typed
/// refusals the kit does not cover.
@Suite(.serialized)
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

    @Test func aRemoteHTTPStreamExtractsThroughTheBoundedAdapter() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }
        EmbeddedTextStubURLProtocol.reset()
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [EmbeddedTextStubURLProtocol.self]
        EmbeddedTextStubURLProtocol.mode = .reply(
            statusCode: 206,
            headers: [
                "Content-Length": "73482",
                "Content-Range": "bytes 0-73481/73482"
            ],
            body: try Data(contentsOf: URL(fileURLWithPath: ContractTests.fixturePath("contract-clip.mkv")))
        )
        engine.remoteExtractionConfiguration = configuration
        engine.currentLocator = "https://example.com/stream.mkv"

        let text = try engine.extractText(track: 3)
        #expect(text == (try Self.golden("contract-clip.sub-eng.golden")))
        #expect(EmbeddedTextStubURLProtocol.lastRange == "bytes=0-8388607")
    }

    /// Optional local acceptance probe for NEN-109. The test is inert in CI;
    /// the evidence run supplies a real URL emitted by the local Stremio
    /// EngineFS HTTP server, without ever printing that URL or the subtitle.
    @Test func aConfiguredStremioHTTPStreamExtractsEmbeddedText() throws {
        guard let locator = ProcessInfo.processInfo.environment["NEN_STREMIO_M6_URL"],
              !locator.isEmpty
        else { return }

        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }
        engine.currentLocator = locator

        let text = try engine.extractText(track: 3)
        #expect(text == (try Self.golden("contract-clip.sub-eng.golden")))
    }

    @Test func anOversizedRemoteResponseIsATypeRefusalNotTruncatedText() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }
        EmbeddedTextStubURLProtocol.reset()
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [EmbeddedTextStubURLProtocol.self]
        EmbeddedTextStubURLProtocol.mode = .reply(
            statusCode: 200,
            headers: ["Content-Length": "\(EmbeddedTextExtractor.maxRemoteBytes + 1)"],
            body: Data()
        )
        engine.remoteExtractionConfiguration = configuration
        engine.currentLocator = "https://example.com/stream.mkv"

        #expect(throws: FfiPlaybackError.RemoteResponseTooLarge) {
            _ = try engine.extractText(track: 3)
        }
    }

    @Test func cancellingAReadStopsTheUnderlyingURLSessionTask() throws {
        let engine = try TrackEnumerationTests.load("contract-clip.mkv")
        defer { try? engine.shutdown() }
        EmbeddedTextStubURLProtocol.reset()
        let configuration = URLSessionConfiguration.ephemeral
        configuration.protocolClasses = [EmbeddedTextStubURLProtocol.self]
        EmbeddedTextStubURLProtocol.mode = .hang
        engine.remoteExtractionConfiguration = configuration
        engine.currentLocator = "https://example.com/stream.mkv"

        let finished = DispatchSemaphore(value: 0)
        Thread {
            defer { finished.signal() }
            _ = try? engine.extractText(track: 3)
        }.start()
        #expect(EmbeddedTextStubURLProtocol.started.wait(timeout: .now() + 2) == .success)
        let cancellationStartedAt = Date()
        engine.cancelExtractText()
        #expect(EmbeddedTextStubURLProtocol.stopped.wait(timeout: .now() + 2) == .success)
        #expect(finished.wait(timeout: .now() + 2) == .success)
        #expect(Date().timeIntervalSince(cancellationStartedAt) < 1.0)
    }
}
