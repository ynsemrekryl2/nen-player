import Foundation
import Testing
@testable import NenPlayerShell

@Suite("subtitle language preference storage")
struct SubtitlePreferenceStoreTests {
    /// A fresh, isolated `UserDefaults` suite per test — the same pattern
    /// `RecentMediaStoreTests` uses, for the same reason: tests must not
    /// share, or leak into, the real defaults domain.
    private func freshDefaults() -> (UserDefaults, () -> Void) {
        let suiteName = "player.nen.tests.subtitle-preferences.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        return (defaults, { defaults.removePersistentDomain(forName: suiteName) })
    }

    @Test("two preferences survive a fresh store instance")
    func preferencesSurviveRestart() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let writer = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { nil })
        writer.save(SubtitleLanguagePreferences(primary: "tr", secondary: "en"))

        // A second instance models the app relaunching — the store itself
        // holds nothing in memory that a restart would lose.
        let reader = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { nil })
        #expect(reader.preferences == SubtitleLanguagePreferences(primary: "tr", secondary: "en"))
    }

    @Test("the system language seeds the primary preference exactly once")
    func systemLanguageSeedsOnlyOnce() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let first = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { "tr-TR" })
        #expect(first.preferences.primary == "tr", "the region is dropped — ADR-0030's granularity")

        first.save(SubtitleLanguagePreferences())
        // A relaunch that still reports the same system language must not
        // put the seed back — the user's empty choice is the one that counts.
        let second = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { "tr-TR" })
        #expect(second.preferences == SubtitleLanguagePreferences(), "cleared preferences stay cleared")
    }

    @Test("no system language leaves the seed empty")
    func noSystemLanguageSeedsNothing() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let store = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { nil })
        #expect(store.preferences == SubtitleLanguagePreferences())
    }

    @Test("a secondary equal to the primary is dropped on save")
    func matchingSecondaryIsDroppedOnSave() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let store = UserDefaultsSubtitlePreferenceStore(defaults: defaults, systemLanguage: { nil })
        store.save(SubtitleLanguagePreferences(primary: "tr", secondary: "tr"))

        #expect(store.preferences == SubtitleLanguagePreferences(primary: "tr", secondary: nil))
    }
}

@Suite("SubtitleLanguagePreferences normalization")
struct SubtitleLanguagePreferencesTests {
    @Test("a secondary equal to the primary normalizes away")
    func matchingSecondaryDrops() {
        let preferences = SubtitleLanguagePreferences(primary: "tr", secondary: "tr")
        #expect(preferences.normalized() == SubtitleLanguagePreferences(primary: "tr", secondary: nil))
    }

    @Test("a distinct secondary is kept")
    func distinctSecondaryStays() {
        let preferences = SubtitleLanguagePreferences(primary: "tr", secondary: "en")
        #expect(preferences.normalized() == preferences)
    }

    @Test("no primary keeps whatever secondary was given")
    func noPrimaryKeepsSecondary() {
        let preferences = SubtitleLanguagePreferences(primary: nil, secondary: "en")
        #expect(preferences.normalized() == preferences)
    }
}
