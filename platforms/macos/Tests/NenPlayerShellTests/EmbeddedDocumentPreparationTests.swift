import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

/// `NEN-044`: `translateSelectedSubtitle()` asks the session to prepare a
/// document before it ever touches `FfiTranslationEngine`, and a refusal
/// from that ask is its own transient message — never a job that started
/// and then failed.
///
/// `TranslationCommandTests` (`NEN-101`) proves the happy path still
/// catalogues and never retargets; this file is only about the new call and
/// what happens when it refuses. `FakeSession` cannot attach a document to
/// the real `FfiSubtitleLibrary` it is handed (that materialization is the
/// production `FfiPlaybackSession`'s job, proved at the Rust/FFI level by
/// `nen-app`'s `embedded_extraction.rs` and `nen-ffi`'s
/// `embedded_document.rs`), so every scenario here uses an ordinary user
/// file — the call happens for that row exactly as it would for an
/// embedded one, and `.success(.ready)` is what a user file's document
/// always answers with.
@Suite("Embedded document preparation (NEN-044)")
@MainActor
struct EmbeddedDocumentPreparationTests {
    @Test("translateSelectedSubtitle asks the session to prepare the selected token's document")
    func asksSessionToPrepare() async throws {
        let store = TempFixture("prepare-embedded-asks")
        defer { store.remove() }
        let dir = TempFixture("prepare-embedded-asks-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let session = FakeSession()
        let model = makeModel(session: session, storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)

        #expect(session.prepareEmbeddedDocumentCalls.isEmpty, "selecting alone must not prepare")

        model.translateSelectedSubtitle()
        await model.awaitTranslation()

        #expect(session.prepareEmbeddedDocumentCalls == [token])
    }

    @Test("a prepare refusal shows its own message and adds no source")
    func prepareRefusalShowsItsOwnMessage() async throws {
        let store = TempFixture("prepare-embedded-refuses")
        defer { store.remove() }
        let dir = TempFixture("prepare-embedded-refuses-media")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Source.en.srt", TempFixture.validSrt)

        let session = FakeSession()
        session.prepareEmbeddedDocumentResult = .failure(.TrackCarriesNoText)
        let model = makeModel(session: session, storeRoot: store.url)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)
        let token = try #require(userSubtitleToken(in: model))
        model.selectSubtitle(token: token)
        let sourceCountBefore = model.subtitleSourceCount

        model.translateSelectedSubtitle()
        await model.awaitTranslation()

        #expect(!model.isTranslating)
        #expect(session.prepareEmbeddedDocumentCalls == [token])
        #expect(
            model.transientMessage
                == PlaybackPresentation.prepareEmbeddedDocumentMessage(for: .TrackCarriesNoText)
        )
        #expect(model.subtitleSourceCount == sourceCountBefore, "no AI row was added")
        #expect(
            !FileManager.default.fileExists(atPath: store.url.appendingPathComponent("artifacts").path),
            "a refused preparation must never reach the store"
        )
    }

    // MARK: - Helpers

    private func makeModel(session: FakeSession, storeRoot: URL) -> PlayerModel {
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            translationPreferenceStore: MemoryTranslationPreferenceStore(targetLanguage: "tr"),
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
