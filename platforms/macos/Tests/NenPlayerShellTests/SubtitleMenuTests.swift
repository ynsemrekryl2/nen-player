import Foundation
import NenCore
import NenPlaybackMPV
import Testing
@testable import NenPlayerShell

/// §8's menu as the user meets it (NEN-026).
///
/// The core already proves *which* headings exist and in what order, against
/// four golden files. Nothing here re-checks that. What only exists on this
/// side is the menu's **visible text** — a language's own name, the Turkish
/// chrome — and the three rules ADR-0031 Karar 4 and 5 put on the shell:
/// a growing list must not move the user, `Kapalı` is an action, and a broken
/// row stays in the list and out of reach.
@Suite("Subtitle menu")
@MainActor
struct SubtitleMenuTests {
    // MARK: - The visible text (§8, ADR-0010 Karar 7)

    @Test("a language is named in its own language, whatever the UI language is")
    func headingsUseEndonyms() {
        #expect(SubtitleMenuPresentation.groupTitle(.language(tag: "en")) == "English")
        #expect(SubtitleMenuPresentation.groupTitle(.language(tag: "fr")) == "Français")
        #expect(SubtitleMenuPresentation.groupTitle(.language(tag: "tr")) == "Türkçe")
        #expect(SubtitleMenuPresentation.groupTitle(.language(tag: "de")) == "Deutsch")
    }

    @Test("the chrome around the languages is Turkish")
    func chromeIsTurkish() {
        #expect(SubtitleMenuPresentation.groupTitle(.closed) == "Kapalı")
        #expect(SubtitleMenuPresentation.groupTitle(.userSubtitles) == "Kullanıcı Altyazıları")
        #expect(SubtitleMenuPresentation.groupTitle(.unknownLanguage) == "Dil Belirsiz")
    }

    @Test("a language the system cannot name falls back to its tag, never to an invention")
    func anUnnameableLanguageShowsItsTag() {
        #expect(SubtitleMenuPresentation.groupTitle(.language(tag: "zz")) == "ZZ")
    }

    @Test("a titleless track is labelled with its own language")
    func aTitlelessTrackBorrowsItsLanguage() {
        // The core deliberately leaves this empty rather than inventing a name
        // in one language and showing it in every other one (NEN-023).
        #expect(SubtitleMenuPresentation.entryTitle(entry(label: "", language: "fr")) == "Français")
        #expect(SubtitleMenuPresentation.entryTitle(entry(label: "", language: nil)) == "Adsız parça")
        #expect(
            SubtitleMenuPresentation.entryTitle(entry(label: "English (SDH)", language: "en"))
                == "English (SDH)"
        )
    }

    @Test("the reason a row cannot be used wins over where it came from")
    func aReasonReplacesTheOriginBadge() {
        #expect(SubtitleMenuPresentation.entrySubtitle(entry(kind: .embedded)) == "Gömülü")
        #expect(SubtitleMenuPresentation.entrySubtitle(entry(kind: .openSubtitles)) == "OpenSubtitles")
        #expect(SubtitleMenuPresentation.entrySubtitle(entry(kind: .ai)) == "AI")
        #expect(
            SubtitleMenuPresentation.entrySubtitle(entry(kind: .user, defect: .malformed))
                == "biçim hatalı"
        )
        #expect(
            SubtitleMenuPresentation.entrySubtitle(entry(kind: .user, defect: .unreadable))
                == "okunamadı"
        )
    }

    @Test("the transport label says Kapalı or the selected entry title")
    func transportSelectionLabel() {
        let selected = entry(label: "English (SDH)", language: "en")
        let menu = [FfiMenuSection(group: .language(tag: "en"), entries: [selected])]

        #expect(SubtitleMenuPresentation.selectionLabel(selectedToken: nil, menu: menu) == "Kapalı")
        #expect(
            SubtitleMenuPresentation.selectionLabel(selectedToken: selected.token, menu: menu)
                == "English (SDH)"
        )
        #expect(SubtitleMenuPresentation.selectionLabel(selectedToken: 99, menu: menu) == "Altyazı")
    }

    // MARK: - Column two with nothing in it

    @Test("the three empty states say three different things, in priority order")
    func emptyStatesAreDistinct() {
        // "Off" is only useful news when there was something to turn on, so a
        // medium with nothing to offer says that instead.
        #expect(
            SubtitleMenuPresentation.emptyMessage(hasAnySource: true, isScanning: false)
                == "Altyazılar kapalı."
        )
        #expect(
            SubtitleMenuPresentation.emptyMessage(hasAnySource: true, isScanning: true)
                == "Altyazılar kapalı."
        )
        #expect(
            SubtitleMenuPresentation.emptyMessage(hasAnySource: false, isScanning: true)
                == "Altyazılar aranıyor…"
        )
        #expect(
            SubtitleMenuPresentation.emptyMessage(hasAnySource: false, isScanning: false)
                == "Bu medya için altyazı bulunamadı."
        )
    }

    // MARK: - What the menu holds

    @Test("a medium with no subtitles at all offers Kapalı and nothing else")
    func anEmptyMenuStillOffersClosed() async {
        let dir = TempFixture("menu-empty")
        defer { dir.remove() }
        let media = dir.write("Bare.mkv", "not really a video")

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        await model.awaitSidecarScan()

        #expect(model.subtitleMenu.map { SubtitleMenuGroupID($0.group) } == [.closed])
        #expect(model.browsedSubtitleEntries.isEmpty)
        #expect(!model.hasAnySubtitleSource)
    }

    @Test("an empty heading is never drawn")
    func headingsAppearOnlyOnceTheyHaveSomething() async {
        // The rule column one draws: `Kullanıcı Altyazıları` and `Dil Belirsiz`
        // exist only when something is in them (§8).
        let dir = TempFixture("menu-headings")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")

        let fixture = FakeSession()
        fixture.subtitleTracks = [subtitleTrack(id: 2, language: "en", title: "English")]
        let model = makeModel(session: fixture)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSidecarScan()

        #expect(
            model.subtitleMenu.map { SubtitleMenuGroupID($0.group) } == [.closed, .language("en")]
        )
    }

    @Test("a titleless track opens Dil Belirsiz and a sidecar opens Kullanıcı Altyazıları")
    func bothOptionalHeadingsAppearWhenEarned() async {
        let dir = TempFixture("menu-optional")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        dir.write("Film.srt", TempFixture.validSrt)

        let fixture = FakeSession()
        fixture.subtitleTracks = [
            subtitleTrack(id: 2, language: "en", title: "English"),
            subtitleTrack(id: 3, language: nil, title: nil)
        ]
        let model = makeModel(session: fixture)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSidecarScan()

        let headings = model.subtitleMenu.map { SubtitleMenuGroupID($0.group) }
        #expect(headings.contains(.userSubtitles))
        #expect(headings.contains(.unknownLanguage))
        #expect(headings.last == .unknownLanguage, "Dil Belirsiz stays behind every language")
    }

    // MARK: - Kapalı is an action (2. tur, 2026-08-27)

    @Test("Kapalı turns the subtitle off and takes over column two")
    func closedTurnsOffAndOwnsTheSecondColumn() async throws {
        let (model, fixture) = await playingModelWithTracks()
        let english = try #require(token(in: model, group: .language("en")))

        // Browse somewhere *else* first, or pressing `Kapalı` would prove
        // nothing about column two: it starts there.
        model.browseSubtitleGroup(.language("en"))
        model.selectSubtitle(token: english)
        #expect(model.selectedSubtitleToken == english)
        #expect(model.browsedSubtitleGroup == .language("en"))
        #expect(!model.browsedSubtitleEntries.isEmpty)

        model.browseSubtitleGroup(.closed)

        #expect(model.selectedSubtitleToken == nil)
        #expect(model.browsedSubtitleGroup == .closed, "column two came back to Kapalı")
        #expect(model.browsedSubtitleEntries.isEmpty)
        #expect(fixture.drawnSubtitles.last == .off, "the engine was told to stop")
    }

    @Test("looking at a language while subtitles are off does not turn them on")
    func browsingIsNotSelecting() async {
        let (model, fixture) = await playingModelWithTracks()
        let before = fixture.drawnSubtitles.count

        model.browseSubtitleGroup(.language("en"))

        #expect(model.browsedSubtitleGroup == .language("en"))
        #expect(!model.browsedSubtitleEntries.isEmpty, "the rows are there to look at")
        #expect(model.selectedSubtitleToken == nil, "nothing is showing")
        #expect(fixture.drawnSubtitles.count == before, "the engine was not touched")
    }

    // MARK: - Selection

    @Test("selecting an embedded row reaches the engine as its own track")
    func anEmbeddedRowSelectsItsTrack() async throws {
        let (model, fixture) = await playingModelWithTracks()
        let turkish = try #require(token(in: model, group: .language("tr")))

        model.selectSubtitle(token: turkish)

        #expect(model.selectedSubtitleToken == turkish)
        #expect(fixture.drawnSubtitles.last == .track(3))
    }

    @Test("selecting a user file draws the file instead of an embedded track")
    func aUserFileReplacesTheEmbeddedTrack() async throws {
        // What must not happen is two subtitles on screen at once: the file
        // replaces the track rather than joining it (NEN-027).
        let (model, fixture) = await playingModelWithTracks(withSidecar: true)
        model.selectSubtitle(token: try #require(token(in: model, group: .language("en"))))

        let sidecar = try #require(token(in: model, group: .userSubtitles))
        model.selectSubtitle(token: sidecar)

        #expect(model.selectedSubtitleToken == sidecar)
        #expect(fixture.drawnSubtitles.last == .document(sidecar))
        #expect(fixture.drawnSubtitles.count == 2, "one replaced the other, nothing stacked")
    }

    @Test("a broken row is in the list and out of reach")
    func aDefectiveRowCannotBeSelected() async throws {
        let dir = TempFixture("menu-broken")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        // Times that do not advance: read fine, then refused by the strict
        // parser (NEN-013) — `biçim hatalı`.
        dir.write("Film.srt", "1\n00:00:05,000 --> 00:00:02,000\nBackwards.\n")

        let fixture = FakeSession()
        let model = makeModel(session: fixture)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSidecarScan()

        model.browseSubtitleGroup(.userSubtitles)
        let row = try #require(model.browsedSubtitleEntries.first)
        #expect(row.defect == .malformed)
        #expect(!SubtitleMenuPresentation.isSelectable(row))
        #expect(SubtitleMenuPresentation.entrySubtitle(row) == "biçim hatalı")

        model.selectSubtitle(token: row.token)
        #expect(model.selectedSubtitleToken == nil, "the model refuses it too, not just the view")
    }

    @Test("a file refused at a gate leaves no trace in the menu")
    func aRefusedFileIsNotInTheMenu() async {
        // ADR-0031 Karar 5's other half: it never became a source, so there is
        // nothing to list. A symlink is the gate that a real filesystem can
        // actually answer.
        let dir = TempFixture("menu-refused")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let real = dir.write("real.srt", TempFixture.validSrt)
        _ = dir.symlink("Film.srt", to: real)

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        await model.awaitSidecarScan()

        #expect(model.subtitleMenu.map { SubtitleMenuGroupID($0.group) } == [.closed])
        #expect(model.subtitleSourceCount == 0)
        #expect(model.transientMessage == nil, "a scan says nothing, whatever it finds")
    }

    // MARK: - A file the user picks himself (NEN-028)

    @Test("a file the user loads is in the menu as soon as it is loaded")
    func aLoadedFileIsInTheMenuAtOnce() {
        // The menu is the only surface a user file has (ADR-0031 Karar 5), so
        // a counter moving is not the same thing as the list the user opens
        // having the row in it. Nothing is awaited here on purpose: the row
        // has to be there on the load's own turn, not on the scan's.
        let dir = TempFixture("menu-loaded")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let subtitle = dir.write("Chosen.srt", TempFixture.validSrt)

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: subtitle)

        #expect(model.subtitleSourceCount == 1)
        let user = model.subtitleMenu.first { SubtitleMenuGroupID($0.group) == .userSubtitles }
        #expect(user?.entries.map(\.label) == ["Chosen.srt"])
    }

    @Test("a broken file the user loads is in the menu, marked and out of reach")
    func aBrokenLoadedFileIsMarkedInTheMenu() {
        // The M3 exit criterion "bozuk bir .srt playback'i durdurmuyor" has a
        // second half: the source is *marked*, and the mark is only visible if
        // the row reaches the menu.
        let dir = TempFixture("menu-loaded-broken")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let broken = dir.write("Broken.srt", "this is not a timecode")

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: broken)

        let user = model.subtitleMenu.first { SubtitleMenuGroupID($0.group) == .userSubtitles }
        let row = user?.entries.first
        #expect(row?.label == "Broken.srt")
        #expect(row?.defect == .malformed)
        #expect(row.map(SubtitleMenuPresentation.entrySubtitle) == "biçim hatalı")
        #expect(model.transientMessage == nil, "the menu carries it, not the transport")
    }

    @Test("a file the user loads and has refused leaves no row behind")
    func aRefusedLoadedFileIsNotInTheMenu() {
        // The control for the two tests above: if the menu were refreshed on
        // any load whatsoever, this heading would appear empty-handed.
        let dir = TempFixture("menu-loaded-refused")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        let real = dir.write("real.srt", TempFixture.validSrt)
        let link = dir.symlink("Chosen.srt", to: real)

        let model = makeModel(session: FakeSession())
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        model.loadSubtitleFile(at: link)

        #expect(model.subtitleMenu.map { SubtitleMenuGroupID($0.group) } == [.closed])
        #expect(model.transientMessage == "Bu bir kısayol; altyazı olarak açılamıyor.")
    }

    // MARK: - The list grows while the menu is open (ADR-0031 Karar 4)

    @Test("the menu is usable before the scan finishes")
    func theMenuDoesNotWaitForTheScan() async {
        let dir = TempFixture("menu-scan")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        dir.write("Film.srt", TempFixture.validSrt)

        let fixture = FakeSession()
        fixture.subtitleTracks = [subtitleTrack(id: 2, language: "en", title: "English")]
        let model = makeModel(session: fixture)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])

        // Still scanning, and the embedded track is already selectable.
        #expect(model.isScanningSubtitles)
        #expect(model.subtitleMenu.map { SubtitleMenuGroupID($0.group) } == [.closed, .language("en")])
        model.browseSubtitleGroup(.language("en"))
        #expect(model.browsedSubtitleEntries.count == 1)

        await model.awaitSidecarScan()
        #expect(!model.isScanningSubtitles)
    }

    @Test("the sidecar landing moves nothing the user was holding")
    func aLandingScanKeepsTheSelectionFocusAndColumn() async throws {
        // This is the concrete break ADR-0031 Karar 4.2 protects against:
        // `Kullanıcı Altyazıları` is born at position two and pushes every
        // language heading down a row. Bound to a row index, the browsed
        // column and the selection would both land on a different subtitle.
        let dir = TempFixture("menu-growth")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        dir.write("Film.srt", TempFixture.validSrt)

        let fixture = FakeSession()
        fixture.subtitleTracks = [
            subtitleTrack(id: 2, language: "en", title: "English"),
            subtitleTrack(id: 3, language: "tr", title: "Türkçe")
        ]
        let model = makeModel(session: fixture)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])

        model.browseSubtitleGroup(.language("tr"))
        let turkish = try #require(token(in: model, group: .language("tr")))
        model.selectSubtitle(token: turkish)
        let headingsBefore = model.subtitleMenu.map { SubtitleMenuGroupID($0.group) }

        await model.awaitSidecarScan()

        let headingsAfter = model.subtitleMenu.map { SubtitleMenuGroupID($0.group) }
        #expect(headingsAfter.count == headingsBefore.count + 1, "the list really did grow")
        #expect(headingsAfter.contains(.userSubtitles))
        #expect(model.browsedSubtitleGroup == .language("tr"), "column two did not move")
        #expect(model.selectedSubtitleToken == turkish, "the selection did not move")
        #expect(model.browsedSubtitleEntries.first?.token == turkish)
    }

    @Test("the preferred language's embedded track is opened when playback starts")
    func automaticSelectionOpensThePreferredTrack() async throws {
        // §8, as extended by ADR-0010 Karar 9: a source that is already
        // playable in the preferred language opens by itself, and opening it
        // downloads nothing and translates nothing.
        let (model, fixture) = await playingModelWithTracks(preferredLanguage: "tr-TR")

        let turkish = try #require(token(in: model, group: .language("tr")))
        #expect(model.selectedSubtitleToken == turkish)
        #expect(fixture.drawnSubtitles == [.track(3)], "the engine drew it, once")
        #expect(
            SubtitleMenuGroupID(model.subtitleMenu[1].group) == .language("tr"),
            "a regional preference still hoists its language (ADR-0030)"
        )
    }

    @Test("no preference opens nothing at all")
    func withoutAPreferenceNothingIsOpened() async {
        let (model, fixture) = await playingModelWithTracks()
        #expect(model.selectedSubtitleToken == nil)
        #expect(fixture.drawnSubtitles.isEmpty)
    }

    @Test("a preferred sidecar found after playback started is not opened by itself")
    func aLateSidecarNeverRetriggersAutomaticSelection() async {
        // ADR-0031 Karar 4.3, and the same family as §9's rule that nobody is
        // switched to AI output without asking.
        let dir = TempFixture("menu-late")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        // Long enough for NEN-020's detector to clear its confidence
        // threshold: the sidecar has to land in the *preferred* language, or
        // the test would pass for the boring reason that nothing matched.
        dir.write("Film.srt", TempFixture.turkishSrt)

        let fixture = FakeSession()
        let model = makeModel(session: fixture, preferredLanguage: "tr")
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        #expect(model.selectedSubtitleToken == nil, "nothing was catalogued yet")

        await model.awaitSidecarScan()

        #expect(model.subtitleSourceCount == 1, "it is in the menu")
        #expect(
            model.subtitleMenu.contains { SubtitleMenuGroupID($0.group) == .userSubtitles },
            "and it really is the kind of source that would have been picked"
        )
        #expect(model.selectedSubtitleToken == nil, "and it is not showing")
        #expect(fixture.drawnSubtitles.isEmpty)
    }

    @Test("the same sidecar found before playback starts is opened")
    func aSidecarThatLandedInTimeIsOpened() async throws {
        // The other side of the one-shot rule, and what makes the test above
        // mean something: nothing is wrong with the sidecar itself. Arriving
        // before the medium is ready is the whole difference.
        let dir = TempFixture("menu-intime")
        defer { dir.remove() }
        let media = dir.write("Film.mkv", "not really a video")
        dir.write("Film.srt", TempFixture.turkishSrt)

        let fixture = FakeSession()
        let model = makeModel(session: fixture, preferredLanguage: "tr")
        model.openMedia(at: media)
        await model.awaitSidecarScan()
        model.consume([.stateChanged(state: .ready)])

        #expect(model.selectedSubtitleToken != nil, "it was there when the shot was taken")
    }

    // MARK: - Helpers

    private func entry(
        label: String = "Movie.srt",
        language: String? = "tr",
        kind: FfiSubtitleSourceKind = .user,
        defect: FfiSourceDefect? = nil
    ) -> FfiMenuEntry {
        FfiMenuEntry(
            token: 1,
            kind: kind,
            language: language,
            label: label,
            defect: defect,
            translatable: true
        )
    }

    private func subtitleTrack(id: UInt32, language: String?, title: String?) -> FfiTrackDescriptor {
        FfiTrackDescriptor(
            id: id,
            kind: .subtitle,
            language: language,
            codec: "subrip",
            isDefault: false,
            title: title
        )
    }

    private func token(in model: PlayerModel, group: SubtitleMenuGroupID) -> UInt32? {
        model.subtitleMenu
            .first { SubtitleMenuGroupID($0.group) == group }?
            .entries.first?.token
    }

    /// A model on a medium with an English and a Turkish embedded track, past
    /// the scan, with nothing selected.
    private func playingModelWithTracks(
        withSidecar: Bool = false,
        preferredLanguage: String? = nil
    ) async -> (PlayerModel, FakeSession) {
        let dir = TempFixture("menu-tracks")
        let media = dir.write("Film.mkv", "not really a video")
        if withSidecar {
            dir.write("Film.srt", TempFixture.validSrt)
        }

        let fixture = FakeSession()
        fixture.subtitleTracks = [
            subtitleTrack(id: 2, language: "en", title: "English"),
            subtitleTrack(id: 3, language: "tr", title: "Türkçe")
        ]
        let model = makeModel(session: fixture, preferredLanguage: preferredLanguage)
        model.openMedia(at: media)
        model.consume([.stateChanged(state: .ready)])
        await model.awaitSidecarScan()
        // The fixture directory outlives the scan; the catalog holds no path.
        dir.remove()
        return (model, fixture)
    }

    /// Every model here states its preferred language.
    ///
    /// `nil` — no preference — is the default on purpose: read from `Locale`,
    /// automatic selection would fire or not fire depending on the language of
    /// the machine running the suite, and half these tests would be green on a
    /// Turkish laptop and red on an English one.
    private func makeModel(
        session: FakeSession,
        preferredLanguage: String? = nil
    ) -> PlayerModel {
        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferredSubtitleLanguage: preferredLanguage,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        return model
    }
}
