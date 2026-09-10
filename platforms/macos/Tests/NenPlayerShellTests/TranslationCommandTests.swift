import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

@Suite("AI ile çevir command (NEN-101)")
@MainActor
struct TranslationCommandTests {
    @Test("the command is enabled only while a source is selected")
    func enabledOnlyWhileSelected() throws {
        let store = TempFixture("translate-enablement")
        defer { store.remove() }
        let dir = TempFixture("translate-enablement-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)

        #expect(!model.canTranslateSelectedSubtitle, "no source is selected yet")

        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        #expect(model.canTranslateSelectedSubtitle)

        model.browseSubtitleGroup(.closed)
        #expect(!model.canTranslateSelectedSubtitle, "Kapalı leaves nothing selected")
    }

    @Test("a source already in the target language disables the command and refuses to start")
    func alreadyTargetLanguageDisablesAndRefuses() async throws {
        let store = TempFixture("translate-already-target")
        defer { store.remove() }
        let dir = TempFixture("translate-already-target-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        // The filename hint is authoritative (ADR-0029 Karar 5), so the
        // content itself does not have to be Turkish for this to prove the
        // language-equality gate rather than language detection.
        let subtitle = dir.write("Source.tr.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        #expect(!model.canTranslateSelectedSubtitle)

        let sourceCountBefore = model.subtitleSourceCount
        model.translateSelectedSubtitle()
        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(model.subtitleSourceCount == sourceCountBefore, "no AI row was added")
        #expect(!FileManager.default.fileExists(atPath: store.url.appendingPathComponent("artifacts").path))
    }

    @Test("a region-qualified source in the target language's primary subtag also refuses")
    func regionQualifiedTargetLanguageAlsoRefuses() throws {
        let store = TempFixture("translate-region-variant")
        defer { store.remove() }
        let dir = TempFixture("translate-region-variant-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.tr-TR.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        #expect(!model.canTranslateSelectedSubtitle, "tr-TR and tr share a primary subtag")
    }

    @Test("selecting a subtitle alone never starts a translation")
    func selectingAloneNeverTranslates() throws {
        let store = TempFixture("translate-select-only")
        defer { store.remove() }
        let dir = TempFixture("translate-select-only-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let sourceCountBefore = model.subtitleSourceCount
        let token = try #require(userSubtitleToken(in: model))

        model.selectSubtitle(token: token)

        #expect(!model.isTranslating)
        #expect(model.subtitleSourceCount == sourceCountBefore)
        #expect(!FileManager.default.fileExists(atPath: store.url.appendingPathComponent("artifacts").path))
    }

    @Test("the command starts, finishes and catalogues without moving the selection")
    func happyPathCataloguesWithoutRetargeting() async throws {
        let store = TempFixture("translate-happy-path")
        defer { store.remove() }
        let dir = TempFixture("translate-happy-path-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession(), targetLanguage: "tr", storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        let sourceCountBefore = model.subtitleSourceCount

        #expect(model.canTranslateSelectedSubtitle)
        model.translateSelectedSubtitle()

        // Set synchronously by the command, before the background task ever
        // runs — the command's own gate against a second concurrent start.
        #expect(model.isTranslating)
        #expect(!model.canTranslateSelectedSubtitle, "a running job disables the command")

        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(model.subtitleSourceCount == sourceCountBefore + 1, "one AI row was catalogued")
        #expect(model.selectedSubtitleToken == token, "the screen never jumps to the AI output (§9)")
        #expect(model.transientMessage == nil)

        let aiEntry = model.subtitleMenu
            .flatMap(\.entries)
            .first { $0.kind == .ai }
        #expect(aiEntry != nil)
    }

    @Test("changing the target language does not reorder the menu or move the selection")
    func changingTargetLanguageLeavesMenuAndSelectionAlone() throws {
        let store = TempFixture("translate-target-change")
        defer { store.remove() }
        let dir = TempFixture("translate-target-change-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let translationStore = MemoryTranslationPreferenceStore(targetLanguage: "tr")
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            translationPreferenceStore: translationStore,
            translationStoreRoot: store.url,
            sessionFactory: { _ in FakeSession() }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        let menuBefore = model.subtitleMenu

        model.updateTranslationTargetLanguage("de")

        #expect(translationStore.targetLanguage == "de")
        #expect(model.translationTargetLanguage == "de")
        #expect(model.subtitleMenu == menuBefore)
        #expect(model.selectedSubtitleToken == token)
    }

    // MARK: - Helpers

    private func makeModel(
        session: FakeSession,
        targetLanguage: String?,
        storeRoot: URL
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            translationPreferenceStore: MemoryTranslationPreferenceStore(targetLanguage: targetLanguage),
            translationStoreRoot: storeRoot,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }

    private func userSubtitleToken(in model: PlayerModel) -> UInt32? {
        model.subtitleMenu
            .first { SubtitleMenuGroupID($0.group) == .userSubtitles }?
            .entries.first?.token
    }
}
