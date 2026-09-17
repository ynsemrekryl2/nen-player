import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

// NEN-131: the `Olaylar` window's log. Three groups — the log's own
// mechanics, the words it uses, and what `PlayerModel` records along the
// background chain. The last test is the K23 negative: the whole chain run
// against a path, a remote URL and a key that each carry a sentinel, and
// not one sentinel reaches a row.

@Suite("Pipeline event log (NEN-131)")
@MainActor
struct PipelineEventLogTests {
    // MARK: - Log mechanics

    @Test("rows keep arrival order and monotonically increasing ids")
    func rowsKeepArrivalOrder() {
        let log = PipelineEventLog()
        let first = log.record(.playbackReady)
        let second = log.record(.subtitlesTurnedOff)

        #expect(log.events.map(\.id) == [first, second])
        #expect(second > first)
        #expect(log.events.map(\.kind) == [.playbackReady, .subtitlesTurnedOff])
    }

    @Test("the oldest rows fall off once the capacity is exceeded")
    func capacityDropsOldestRows() {
        let log = PipelineEventLog(capacity: 3)
        for count in 1 ... 5 {
            log.record(.embeddedTracksCataloged(count: count))
        }

        #expect(log.events.count == 3)
        #expect(log.events.map(\.kind) == [
            .embeddedTracksCataloged(count: 3),
            .embeddedTracksCataloged(count: 4),
            .embeddedTracksCataloged(count: 5),
        ])
    }

    @Test("a live row is updated in place, keeping its id and timestamp")
    func updateReplacesInPlace() {
        var tick = Date(timeIntervalSince1970: 0)
        let log = PipelineEventLog(now: { tick })
        let id = log.record(.translationPhase(phase: .preparing, done: 0, total: 10))
        tick = Date(timeIntervalSince1970: 60)
        log.record(.playbackReady)
        log.update(id, .translationPhase(phase: .translating, done: 4, total: 10))

        #expect(log.events.count == 2)
        #expect(log.events[0].id == id)
        #expect(log.events[0].timestamp == Date(timeIntervalSince1970: 0))
        #expect(log.events[0].kind == .translationPhase(phase: .translating, done: 4, total: 10))
    }

    @Test("updating a row that was trimmed away is a no-op")
    func updateOfTrimmedRowIsIgnored() {
        let log = PipelineEventLog(capacity: 1)
        let gone = log.record(.playbackReady)
        log.record(.subtitlesTurnedOff)
        log.update(gone, .translationFinished)

        #expect(log.events.map(\.kind) == [.subtitlesTurnedOff])
    }

    @Test("clear empties the log and the menu command follows it")
    func clearEmptiesTheLog() {
        let log = PipelineEventLog()
        #expect(log.isEmpty, "\"Günlüğü Temizle\" is disabled on a fresh log")
        log.record(.playbackReady)
        #expect(!log.isEmpty)
        log.clear()
        #expect(log.isEmpty)
        #expect(log.events.isEmpty)
    }

    // MARK: - Presentation

    @Test("every summary is a single line and every detail value is non-empty")
    func summariesAreSingleLines() {
        for kind in Self.allKinds {
            let summary = PipelineEventPresentation.summary(for: kind)
            #expect(!summary.isEmpty)
            #expect(!summary.contains("\n"), "multi-line summary for \(kind)")
            for detail in PipelineEventPresentation.details(for: kind) {
                #expect(!detail.label.isEmpty)
                #expect(!detail.value.isEmpty, "empty detail value for \(kind)")
            }
        }
    }

    @Test("the candidate report says what was tried and what found the rows")
    func candidateReportWording() {
        let fallback = FfiCandidateSearchReport(
            status: .cataloged,
            candidateCount: 3,
            attempted: [.hash, .verifiedIdentity],
            foundBy: .verifiedIdentity
        )
        #expect(
            PipelineEventPresentation.summary(for: .candidateSearchFinished(report: fallback))
                == "OpenSubtitles: 3 aday — doğrulanmış kimlik ile bulundu"
        )
        let details = PipelineEventPresentation.details(for: .candidateSearchFinished(report: fallback))
        #expect(details.contains(.init("Denenen", "hash → doğrulanmış kimlik")))
        #expect(details.contains(.init("Bulan", "doğrulanmış kimlik")))

        let miss = FfiCandidateSearchReport(
            status: .cataloged, candidateCount: 0, attempted: [.hash], foundBy: nil
        )
        #expect(
            PipelineEventPresentation.summary(for: .candidateSearchFinished(report: miss))
                == "OpenSubtitles: aday yok — denenen: hash"
        )
        #expect(PipelineEventPresentation.tone(for: .candidateSearchFinished(report: miss)) == .warning)
    }

    @Test("the identity outcome names the method, and a missing hash is not a provider miss")
    func identityWording() {
        #expect(
            PipelineEventPresentation.summary(
                for: .identityLookupFinished(status: .match, label: "Fixture Film (2020)")
            ) == "Kimlik bulundu: Fixture Film (2020) — OpenSubtitles hash eşleşmesi"
        )
        #expect(
            PipelineEventPresentation.summary(for: .identityLookupFinished(status: .noHash, label: nil))
                == "Kimlik araması yapılmadı — medya hash'i hesaplanamadı"
        )
        #expect(
            PipelineEventPresentation.summary(for: .identityLookupFinished(status: .noMatch, label: nil))
                == "Kimlik bulunamadı — OpenSubtitles hash eşleşmesi yok"
        )
    }

    @Test("an error reaches a row as its case name only")
    func errorNameIsTheCaseOnly() {
        #expect(PipelineEventLog.errorName(FfiIdentityLookupError.ProviderTransport) == "ProviderTransport")
        #expect(PipelineEventLog.errorName(FfiDownloadError.HttpStatus) == "HttpStatus")
        #expect(PipelineEventLog.errorName(PayloadError.detail("/Users/secret/Film.mkv")) == "detail")
        let foundation = NSError(
            domain: "fixture", code: 7, userInfo: [NSFilePathErrorKey: "/Users/secret/Film.mkv"]
        )
        let name = PipelineEventLog.errorName(foundation)
        #expect(!name.contains("secret"))
        #expect(!name.contains("Film.mkv"))
    }

    // MARK: - What the model records

    @Test("opening a medium records the chain: identity by hash, then the candidate search")
    func openRecordsTheChainInOrder() async throws {
        let fixture = TempFixture("event-log-chain")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        _ = fixture.write("Film.en.srt", TempFixture.validSrt)
        let report = FfiCandidateSearchReport(
            status: .cataloged, candidateCount: 2, attempted: [.hash, .verifiedIdentity], foundBy: .verifiedIdentity
        )
        let model = makeModel(
            identity: { _ in FfiIdentityLookupResult(status: .match, identity: fixtureIdentity) },
            candidates: { _, _, _ in SubtitleCandidateSearchResult(library: FfiSubtitleLibrary(), report: report) }
        )

        model.openMedia(at: media)
        await model.awaitIdentityLookup()
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        model.consume([.stateChanged(state: .ready)])
        // A resync reaches `ready` again on the real .app; the row is per medium.
        model.consume([.stateChanged(state: .ready)])

        let kinds = model.events.events.map(\.kind)
        #expect(kinds.first == .mediaOpened(name: "Film.mkv", source: .file))
        #expect(kinds.filter { $0 == .embeddedTracksCataloged(count: 0) }.count == 1)
        #expect(kinds.contains(.identityLookupStarted(method: .localHash)))
        #expect(kinds.contains(.identityLookupFinished(status: .match, label: "Fixture Film (2020)")))
        #expect(kinds.contains(.candidateSearchStarted(languages: ["en"], hasIdentity: true)))
        #expect(kinds.contains(.candidateSearchFinished(report: report)))
        #expect(kinds.contains(.sidecarScanFinished(found: 1)))
        #expect(kinds.contains(.playbackReady))
        #expect(kinds.contains(.embeddedTracksCataloged(count: 0)))
        #expect(kinds.contains(.autoSelection(decision: .localSelected(label: "Film.en.srt"))))
        #expect(kinds.contains(.subtitleSelected(label: "Film.en.srt")))

        let started = try #require(kinds.firstIndex(of: .identityLookupStarted(method: .localHash)))
        let finished = try #require(kinds.firstIndex(of: .identityLookupFinished(status: .match, label: "Fixture Film (2020)")))
        let search = try #require(kinds.firstIndex(of: .candidateSearchStarted(languages: ["en"], hasIdentity: true)))
        #expect(started < finished && finished < search, "the search waits for the identity answer")
    }

    @Test("a keyless install records the silent outcomes and why nothing was downloaded")
    func keylessInstallRecordsSilentOutcomes() async throws {
        let fixture = TempFixture("event-log-keyless")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let report = FfiCandidateSearchReport(status: .noCredential, candidateCount: 0, attempted: [], foundBy: nil)
        let model = makeModel(
            identity: { _ in FfiIdentityLookupResult(status: .noCredential, identity: nil) },
            candidates: { _, _, _ in SubtitleCandidateSearchResult(library: FfiSubtitleLibrary(), report: report) }
        )

        model.openMedia(at: media)
        await model.awaitIdentityLookup()
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        model.consume([.stateChanged(state: .ready)])

        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.identityLookupFinished(status: .noCredential, label: nil)))
        #expect(kinds.contains(.candidateSearchStarted(languages: ["en"], hasIdentity: false)))
        #expect(kinds.contains(.candidateSearchFinished(report: report)))
        #expect(kinds.contains(.autoSelection(decision: .skipped(reason: .automaticDownloadDisabled))))
        #expect(!kinds.contains { if case .downloadStarted = $0 { true } else { false } })
    }

    @Test("typed failures of the evidence workers are rows, not playback errors")
    func workerFailuresAreRows() async throws {
        let fixture = TempFixture("event-log-failures")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let model = makeModel(
            identity: { _ in throw FfiIdentityLookupError.ProviderTransport },
            candidates: { _, _, _ in throw FfiCandidateSearchError.HttpStatus }
        )

        model.openMedia(at: media)
        await model.awaitIdentityLookup()
        await model.awaitSubtitleCandidateSearch()

        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.identityLookupFailed(error: "ProviderTransport")))
        #expect(kinds.contains(.candidateSearchFailed(error: "HttpStatus")))
        #expect(model.fatalMessage == nil)
        #expect(model.transientMessage == nil)
    }

    @Test("a provider download records its start and its typed failure")
    func downloadFailureIsRecorded() async throws {
        let fixture = TempFixture("event-log-download-failure")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let http = EventFixtureHTTPClient()
        let model = makeModel(
            identity: { _ in FfiIdentityLookupResult(status: .match, identity: fixtureIdentity) },
            candidates: { _, identity, languages in
                let library = FfiSubtitleLibrary()
                let report = try searchOpensubtitlesCandidates(
                    mediaHash: nil, identity: identity, languages: languages,
                    credentialStore: http.credentialStore, httpClient: http, library: library
                )
                return SubtitleCandidateSearchResult(library: library, report: report)
            },
            download: { _, _ in throw FfiDownloadError.HttpStatus }
        )

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        let row = try #require(model.subtitleMenu.flatMap(\.entries).first { $0.kind == .openSubtitles })
        model.selectSubtitle(token: row.token)
        await model.awaitSubtitleDownload()

        let label = SubtitleMenuPresentation.entryTitle(row)
        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.downloadStarted(label: label, automatic: false)))
        #expect(kinds.contains(.downloadFailed(label: label, error: "HttpStatus")))
        #expect(!kinds.contains(.downloadFinished(label: label)))
    }

    @Test("a successful download records completion and the selection it produced")
    func downloadSuccessIsRecorded() async throws {
        let fixture = TempFixture("event-log-download-success")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let http = EventFixtureHTTPClient()
        let model = makeModel(
            identity: { _ in FfiIdentityLookupResult(status: .match, identity: fixtureIdentity) },
            candidates: { _, identity, languages in
                let library = FfiSubtitleLibrary()
                let report = try searchOpensubtitlesCandidates(
                    mediaHash: nil, identity: identity, languages: languages,
                    credentialStore: http.credentialStore, httpClient: http, library: library
                )
                return SubtitleCandidateSearchResult(library: library, report: report)
            },
            download: { library, token in
                try library.downloadOpensubtitles(
                    token: token, credentialStore: http.credentialStore, httpClient: http
                )
            }
        )

        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSubtitleCandidateSearch()
        let row = try #require(model.subtitleMenu.flatMap(\.entries).first { $0.kind == .openSubtitles })
        model.selectSubtitle(token: row.token)
        await model.awaitSubtitleDownload()

        let label = SubtitleMenuPresentation.entryTitle(row)
        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.downloadStarted(label: label, automatic: false)))
        #expect(kinds.contains(.downloadFinished(label: label)))
        #expect(kinds.contains(.subtitleSelected(label: label)))
        #expect(model.selectedSubtitleToken == row.token)
    }

    @Test("a translation is one start row, one live phase row and one outcome row")
    func translationRowsAreStartPhaseOutcome() async throws {
        let store = TempFixture("event-log-translate-store")
        defer { store.remove() }
        let dir = TempFixture("event-log-translate-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)
        let model = makeModel(targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(
            model.subtitleMenu.first { SubtitleMenuGroupID($0.group) == .userSubtitles }?.entries.first?.token
        )
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        await model.awaitTranslation()

        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.translationStarted(
            provider: "OpenRouter",
            model: UserDefaultsTranslationPreferenceStore.defaultModel,
            source: "en",
            target: "tr",
            label: "Source.en.srt"
        )))
        let phaseRows = kinds.filter { if case .translationPhase = $0 { true } else { false } }
        #expect(phaseRows.count <= 1, "progress is one row updated in place, not a row per callback")
        #expect(kinds.contains(.translationFinished))
    }

    @Test("a translation cancelled before it starts records the cancellation, not a failure")
    func translationCancelledBeforeStart() async throws {
        let store = TempFixture("event-log-translate-cancel-store")
        defer { store.remove() }
        let dir = TempFixture("event-log-translate-cancel-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)
        let model = makeModel(targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(
            model.subtitleMenu.first { SubtitleMenuGroupID($0.group) == .userSubtitles }?.entries.first?.token
        )
        model.selectSubtitle(token: token)

        model.translateSelectedSubtitle()
        model.cancelTranslation()
        await model.awaitTranslation()

        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.translationCancelled))
        #expect(!kinds.contains { if case .translationFailed = $0 { true } else { false } })
    }

    // MARK: - K23 negative

    @Test("no row ever carries the path, the remote URL, its token, or the key (K23)")
    func noRowCarriesForbiddenData() async throws {
        let fixture = TempFixture("event-log-SECRETDIR")
        defer { fixture.remove() }
        let media = fixture.write("Film.mkv", "not really a video")
        let path = media.path
        let remote = URL(string: "https://media.example.invalid/library/Film.mkv?token=SECRETTOKEN")!
        let model = makeModel(
            identity: { url in
                // A worker failing with a Foundation error that quotes the
                // path and the URL — the worst thing a row could echo.
                throw NSError(domain: "fixture", code: 1, userInfo: [
                    NSFilePathErrorKey: url.absoluteString,
                    NSLocalizedDescriptionKey: "failed for \(url.absoluteString) with key fixture-key",
                ])
            },
            candidates: { url, _, _ in throw PayloadError.detail(url.absoluteString) },
            download: { _, _ in throw PayloadError.detail("fixture-key") }
        )

        model.openMedia(at: media)
        await model.awaitIdentityLookup()
        await model.awaitSubtitleCandidateSearch()
        await model.awaitSidecarScan()
        model.consume([.stateChanged(state: .ready)])
        model.openMedia(at: remote)
        await model.awaitIdentityLookup()
        await model.awaitSubtitleCandidateSearch()
        model.consume([.stateChanged(state: .ready)])

        #expect(model.events.events.count >= 8, "the chain ran for both media")
        let forbidden = [path, fixture.url.path, "SECRETDIR", "SECRETTOKEN", "media.example.invalid", "token=", "fixture-key", "https://"]
        for event in model.events.events {
            let text = ([PipelineEventPresentation.summary(for: event.kind)]
                + PipelineEventPresentation.details(for: event.kind).flatMap { [$0.label, $0.value] })
                .joined(separator: "\n")
            for needle in forbidden {
                #expect(!text.contains(needle), "row leaked \(needle): \(text)")
            }
        }
        let kinds = model.events.events.map(\.kind)
        #expect(kinds.contains(.mediaOpened(name: "Film.mkv", source: .file)))
        #expect(kinds.contains(.mediaOpened(name: "Film.mkv", source: .remote)))
        #expect(kinds.contains(.identityLookupStarted(method: .remoteEvidenceThenHash)))
        #expect(kinds.contains(.candidateSearchFailed(error: "detail")))
    }

    // MARK: - Fixtures


    static let allKinds: [PipelineEventKind] = [
        .mediaOpened(name: "Film.mkv", source: .file),
        .mediaOpened(name: "Film.mkv", source: .remote),
        .playbackReady,
        .playbackFailed(error: "Unusable"),
        .identityLookupStarted(method: .localHash),
        .identityLookupStarted(method: .remoteEvidenceThenHash),
        .identityLookupFinished(status: .match, label: "Fixture Film (2020)"),
        .identityLookupFinished(status: .noMatch, label: nil),
        .identityLookupFinished(status: .ambiguous, label: nil),
        .identityLookupFinished(status: .noCredential, label: nil),
        .identityLookupFinished(status: .noHash, label: nil),
        .identityLookupFailed(error: "ProviderTransport"),
        .sidecarScanFinished(found: 0),
        .sidecarScanFinished(found: 2),
        .embeddedTracksCataloged(count: 0),
        .embeddedTracksCataloged(count: 3),
        .candidateSearchStarted(languages: [], hasIdentity: false),
        .candidateSearchStarted(languages: ["en", "tr"], hasIdentity: true),
        .candidateSearchFinished(report: FfiCandidateSearchReport(
            status: .cataloged, candidateCount: 0, attempted: [.hash], foundBy: nil
        )),
        .candidateSearchFinished(report: FfiCandidateSearchReport(
            status: .cataloged, candidateCount: 4, attempted: [.hash], foundBy: .hash
        )),
        .candidateSearchFinished(report: FfiCandidateSearchReport(
            status: .noCredential, candidateCount: 0, attempted: [], foundBy: nil
        )),
        .candidateSearchFinished(report: FfiCandidateSearchReport(
            status: .noIdentity, candidateCount: 0, attempted: [], foundBy: nil
        )),
        .candidateSearchFailed(error: "Transport"),
        .autoSelection(decision: .localSelected(label: "Film.en.srt")),
        .autoSelection(decision: .providerDownloadStarted(label: "Release")),
        .autoSelection(decision: .skipped(reason: .alreadySelected)),
        .autoSelection(decision: .skipped(reason: .localSourceArrivedLate)),
        .autoSelection(decision: .skipped(reason: .automaticDownloadDisabled)),
        .autoSelection(decision: .skipped(reason: .identityNotVerified)),
        .autoSelection(decision: .skipped(reason: .alreadyAttemptedToday)),
        .autoSelection(decision: .skipped(reason: .noMatchingCandidate)),
        .subtitleSelected(label: "Release"),
        .subtitlesTurnedOff,
        .downloadStarted(label: "Release", automatic: true),
        .downloadStarted(label: "Release", automatic: false),
        .downloadFinished(label: "Release"),
        .downloadFailed(label: "Release", error: "HttpStatus"),
        .downloadUnavailable(label: "Release"),
        .translationStarted(provider: "OpenRouter", model: "m", source: "en", target: "tr", label: "Release"),
        .translationPhase(phase: .preparing, done: 0, total: 0),
        .translationPhase(phase: .translating, done: 4, total: 10),
        .translationPhase(phase: .finalizing, done: 10, total: 10),
        .translationFinished,
        .translationCancelled,
        .translationFailed(error: "Failed"),
    ]

    private func makeModel(
        identity: PlayerModel.IdentityLookupRunner? = nil,
        candidates: PlayerModel.SubtitleCandidateSearchRunner? = nil,
        download: PlayerModel.SubtitleDownloadRunner? = nil,
        targetLanguage: String? = nil,
        storeRoot: URL? = nil
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(primary: "en", secondary: nil),
            translationPreferenceStore: MemoryTranslationPreferenceStore(targetLanguage: targetLanguage),
            translationStoreRoot: storeRoot ?? PlayerModel.defaultTranslationStoreRoot(),
            identityLookupRunner: identity,
            subtitleCandidateSearchRunner: candidates,
            subtitleDownloadRunner: download,
            handoffEvidenceCollector: { _ in },
            sessionFactory: { _ in FakeSession() }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }
}

private let fixtureIdentity = FfiVerifiedMediaIdentity(
    title: "Fixture Film", year: 2020, season: nil, episode: nil
)

private enum PayloadError: Error {
    case detail(String)
}

/// The OpenSubtitles fixture answers, so a provider row can exist in the
/// menu and be downloaded without a network (Kural 8).
private final class EventFixtureHTTPClient: ForeignHttpClient, @unchecked Sendable {
    let credentialStore: FfiSecureCredentialStore
    private let foreignStore = EventFixtureCredentialStore()

    init() {
        credentialStore = FfiSecureCredentialStore(store: foreignStore)
    }

    func send(request: FfiHttpRequest) throws -> FfiHttpResponse {
        if request.url.contains("/subtitles?") {
            return response(contentType: "application/json", body: fixture("candidates.json"))
        }
        if request.url.hasSuffix("/download") {
            return response(contentType: "application/json", body: fixture("download-link.json"))
        }
        return response(
            contentType: "text/plain; charset=utf-8",
            body: Data("1\n00:00:00,000 --> 00:00:01,000\nFixture subtitle\n".utf8)
        )
    }

    private func response(contentType: String, body: Data) -> FfiHttpResponse {
        FfiHttpResponse(
            statusCode: 200,
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

private final class EventFixtureCredentialStore: ForeignSecureCredentialStore, @unchecked Sendable {
    func get(kind: FfiCredentialKind) throws -> String? {
        kind == .openSubtitles ? "fixture-key" : nil
    }

    func contains(kind: FfiCredentialKind) throws -> Bool {
        kind == .openSubtitles
    }

    func set(kind: FfiCredentialKind, value: String) throws {}
    func delete(kind: FfiCredentialKind) throws {}
}
