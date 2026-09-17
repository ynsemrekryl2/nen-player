import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

@Suite("OpenSubtitles selection (NEN-123)")
@MainActor
struct OpenSubtitlesSelectionTests {
    @Test("opt-in automatic download falls back from the first to the second preference")
    func automaticDownloadUsesSecondaryPreference() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, _) = makeModel(
            primary: "fr",
            secondary: "en",
            automaticDownloadEnabled: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        let row = try #require(providerEntry(in: model))
        #expect(row.language == "en")
        #expect(model.selectedSubtitleToken == row.token)
        #expect(http.downloadRequests == 2, "download link and subtitle body")
    }

    @Test("when neither preferred language has a candidate, automatic selection closes")
    func automaticDownloadClosesWithoutCandidate() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, _) = makeModel(
            primary: "fr",
            secondary: "de",
            automaticDownloadEnabled: true,
            noCandidates: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        #expect(providerEntry(in: model) == nil)
        #expect(model.selectedSubtitleToken == nil)
        #expect(http.downloadRequests == 0)
    }

    @Test("disabled automatic download leaves provider candidates unselected")
    func disabledAutomaticDownloadDoesNotDownload() async throws {
        let (model, fixture, http, _) = makeModel()
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        #expect(providerEntry(in: model) != nil)
        #expect(model.selectedSubtitleToken == nil)
        #expect(!model.isDownloadingSubtitle)
        #expect(http.downloadRequests == 0)
    }

    @Test("an identity below the automatic gate never downloads")
    func unqualifiedIdentityDoesNotDownload() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, _) = makeModel(
            automaticDownloadEnabled: true,
            identityStatus: .noMatch,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        #expect(providerEntry(in: model) != nil)
        #expect(model.selectedSubtitleToken == nil)
        #expect(http.downloadRequests == 0)
    }

    @Test("a local source always wins over an automatic provider download")
    func localSourceWinsBeforeAutomaticDownload() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, _) = makeModel(
            automaticDownloadEnabled: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        _ = fixture.write("Film.en.srt", TempFixture.validSrt)

        model.openMedia(at: media)
        await model.awaitSidecarScan()
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()

        let local = try #require(
            model.subtitleMenu.flatMap(\.entries).first { $0.kind == .user }
        )
        #expect(model.selectedSubtitleToken == local.token)
        #expect(http.downloadRequests == 0)
    }

    @Test("a local source arriving after ready suppresses provider download without opening")
    func lateLocalSourceSuppressesAutomaticDownloadWithoutOpening() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, session) = makeModel(
            automaticDownloadEnabled: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        _ = fixture.write("Film.en.srt", TempFixture.validSrt)

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        #expect(model.selectedSubtitleToken == nil, "the local scan has not landed")

        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        #expect(model.subtitleMenu.flatMap(\.entries).contains { $0.kind == .user })
        #expect(model.selectedSubtitleToken == nil)
        #expect(session.drawnSubtitles.isEmpty)
        #expect(http.downloadRequests == 0)
    }

    @Test("the automatic download budget allows one attempt per media and day")
    func automaticDownloadDoesNotRetrySameMediaOnReopen() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let (model, fixture, http, _) = makeModel(
            automaticDownloadEnabled: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        http.failDownloads = true

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()
        #expect(http.downloadRequests == 1)

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await model.awaitSubtitleDownload()

        #expect(http.downloadRequests == 1)
        #expect(model.selectedSubtitleToken == nil)
    }

    @Test("an automatic result never overwrites a subtitle selected while downloading")
    func automaticDownloadPreservesUserChoice() async throws {
        let (defaults, cleanup) = isolatedDefaults()
        defer { cleanup() }
        let gate = DownloadGate()
        let (model, fixture, _, _) = makeModel(
            downloadGate: gate,
            automaticDownloadEnabled: true,
            automaticDownloadAttemptDefaults: defaults
        )
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let manual = fixture.write("Manual.en.srt", TempFixture.validSrt)

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        await gate.waitForStart()

        model.loadSubtitleFile(at: manual)
        let local = try #require(
            model.subtitleMenu.flatMap(\.entries).first { $0.kind == .user }
        )
        model.selectSubtitle(token: local.token)
        gate.proceed.signal()
        await model.awaitSubtitleDownload()

        #expect(model.selectedSubtitleToken == local.token)
    }

    @Test("a provider row downloads first, then becomes the selected subtitle")
    func providerSelectionDownloadsBeforeShowing() async throws {
        let (model, fixture, http, session) = makeModel()
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()

        let row = try #require(providerEntry(in: model))
        #expect(model.selectedSubtitleToken == nil)
        #expect(!model.isDownloadingSubtitle)

        model.selectSubtitle(token: row.token)
        #expect(model.isDownloadingSubtitle)
        #expect(model.selectedSubtitleToken == nil, "a metadata row is not selected before download")

        await model.awaitSubtitleDownload()

        #expect(!model.isDownloadingSubtitle)
        #expect(model.selectedSubtitleToken == row.token)
        #expect(model.subtitleMenu.flatMap(\.entries).contains { $0.token == row.token })
        #expect(session.drawnSubtitles.last == .document(row.token))
        #expect(http.downloadRequests == 2, "metadata POST and subtitle GET")
    }

    @Test("a failed provider download preserves the previous subtitle and reports a transient message")
    func failedDownloadDoesNotMoveSelection() async throws {
        let (model, fixture, http, _) = makeModel()
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let sidecar = fixture.write("Film.en.srt", TempFixture.validSrt)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        model.loadSubtitleFile(at: sidecar)
        let localToken = try #require(
            model.subtitleMenu.flatMap(\.entries).first { $0.kind == .user }?.token
        )
        model.selectSubtitle(token: localToken)
        http.failDownloads = true

        let row = try #require(providerEntry(in: model))
        model.selectSubtitle(token: row.token)
        await model.awaitSubtitleDownload()

        #expect(model.selectedSubtitleToken == localToken)
        #expect(model.transientMessage == "OpenSubtitles altyazısı indirilemedi.")
        #expect(!model.isDownloadingSubtitle)
    }

    @Test("a late provider result cannot populate the next medium")
    func lateDownloadResultIsDiscardedAfterMediaChange() async throws {
        let gate = DownloadGate()
        let (model, fixture, _, _) = makeModel(downloadGate: gate)
        defer { fixture.remove() }
        let first = fixture.write("First.mkv", "first")
        let second = fixture.write("Second.mkv", "second")
        model.openMedia(at: first)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        let row = try #require(providerEntry(in: model))

        model.selectSubtitle(token: row.token)
        await gate.waitForStart()

        model.openMedia(at: second)
        gate.proceed.signal()
        try await Task.sleep(nanoseconds: 100_000_000)

        #expect(model.selectedSubtitleToken == nil)
        #expect(!model.subtitleMenu.flatMap(\.entries).contains { $0.kind == .openSubtitles })
        #expect(!model.isDownloadingSubtitle)
    }

    @Test("an uncatalogued provider row is never highlighted as selected")
    func providerRowWithoutDocumentIsNotSelected() async throws {
        let (model, fixture, _, _) = makeModel()
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()

        let row = try #require(providerEntry(in: model))
        model.selectSubtitle(token: row.token)
        #expect(model.selectedSubtitleToken == nil)
        #expect(model.isDownloadingSubtitle)
        await model.awaitSubtitleDownload()
    }

    private func providerEntry(in model: PlayerModel) -> FfiMenuEntry? {
        model.subtitleMenu
            .flatMap(\.entries)
            .first { $0.kind == .openSubtitles }
    }

    private func makeModel(
        downloadGate: DownloadGate? = nil,
        primary: String? = "en",
        secondary: String? = nil,
        automaticDownloadEnabled: Bool = false,
        identityStatus: FfiIdentityLookupStatus = .match,
        noCandidates: Bool = false,
        automaticDownloadAttemptDefaults: UserDefaults = .standard
    ) -> (PlayerModel, TempFixture, FixtureHTTPClient, FakeSession) {
        let fixture = TempFixture("opensubtitles-selection")
        let http = FixtureHTTPClient()
        let credentialStore = http.credentialStore
        let identity = FfiVerifiedMediaIdentity(
            title: "Fixture Film",
            year: 2020,
            season: nil,
            episode: nil
        )
        let session = FakeSession()
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(
                primary: primary,
                secondary: secondary,
                automaticOpenSubtitlesDownloadEnabled: automaticDownloadEnabled
            ),
            automaticDownloadAttemptDefaults: automaticDownloadAttemptDefaults,
            identityLookupRunner: { _ in
                FfiIdentityLookupResult(
                    status: identityStatus,
                    identity: identityStatus == .match ? identity : nil
                )
            },
            subtitleCandidateSearchRunner: { url, verifiedIdentity, languages in
                // The second medium in the stale-result test intentionally
                // has no provider candidates of its own; any row observed
                // there would therefore be the old worker's mutation.
                if noCandidates || url.lastPathComponent == "Second.mkv" {
                    return SubtitleCandidateSearchResult(
                        library: FfiSubtitleLibrary(),
                        report: FfiCandidateSearchReport(
                            status: .cataloged, candidateCount: 0, attempted: [], foundBy: nil
                        )
                    )
                }
                let library = FfiSubtitleLibrary()
                let report = try searchOpensubtitlesCandidates(
                    mediaHash: nil,
                    identity: verifiedIdentity ?? identity,
                    languages: languages,
                    credentialStore: credentialStore,
                    httpClient: http,
                    library: library
                )
                return SubtitleCandidateSearchResult(library: library, report: report)
            },
            subtitleDownloadRunner: { library, token in
                if let downloadGate {
                    downloadGate.started.signal()
                    downloadGate.proceed.wait()
                }
                try library.downloadOpensubtitles(
                    token: token,
                    credentialStore: credentialStore,
                    httpClient: http
                )
            },
            credentialStore: credentialStore,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return (model, fixture, http, session)
    }
}

private func isolatedDefaults() -> (UserDefaults, () -> Void) {
    let suiteName = "player.nen.tests.automatic-subtitles.\(UUID().uuidString)"
    let defaults = UserDefaults(suiteName: suiteName)!
    return (defaults, { defaults.removePersistentDomain(forName: suiteName) })
}

private final class FixtureHTTPClient: ForeignHttpClient, @unchecked Sendable {
    let credentialStore: FfiSecureCredentialStore
    private let foreignStore = FixtureCredentialStore()
    private let lock = NSLock()
    private(set) var downloadRequests = 0
    var failDownloads = false

    init() {
        credentialStore = FfiSecureCredentialStore(store: foreignStore)
    }

    func send(request: FfiHttpRequest) throws -> FfiHttpResponse {
        if request.url.contains("/subtitles?") {
            return response(
                statusCode: 200,
                contentType: "application/json",
                body: fixture("candidates.json")
            )
        }
        if request.url.hasSuffix("/download") {
            lock.lock()
            downloadRequests += 1
            let failed = failDownloads
            lock.unlock()
            if failed {
                return response(statusCode: 503, contentType: "application/json", body: Data())
            }
            return response(
                statusCode: 200,
                contentType: "application/json",
                body: fixture("download-link.json")
            )
        }
        lock.lock()
        downloadRequests += 1
        lock.unlock()
        return response(
            statusCode: 200,
            contentType: "text/plain; charset=utf-8",
            body: Data("1\n00:00:00,000 --> 00:00:01,000\nFixture subtitle\n".utf8)
        )
    }

    private func response(statusCode: UInt16, contentType: String, body: Data) -> FfiHttpResponse {
        FfiHttpResponse(
            statusCode: statusCode,
            headers: [FfiHttpHeader(name: "Content-Type", value: contentType)],
            body: body
        )
    }

    private func fixture(_ name: String) -> Data {
        let root = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        return (try? Data(contentsOf: root.appendingPathComponent("fixtures/providers/opensubtitles/\(name)"))) ?? Data()
    }
}

private final class FixtureCredentialStore: ForeignSecureCredentialStore, @unchecked Sendable {
    func get(kind: FfiCredentialKind) throws -> String? {
        kind == .openSubtitles ? "fixture-key" : nil
    }

    func contains(kind: FfiCredentialKind) throws -> Bool {
        kind == .openSubtitles
    }

    func set(kind: FfiCredentialKind, value: String) throws {}
    func delete(kind: FfiCredentialKind) throws {}
}

private final class DownloadGate: @unchecked Sendable {
    let started = DispatchSemaphore(value: 0)
    let proceed = DispatchSemaphore(value: 0)

    func waitForStart() async {
        await withCheckedContinuation { continuation in
            DispatchQueue.global().async {
                self.started.wait()
                continuation.resume()
            }
        }
    }
}
