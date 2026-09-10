import Foundation
import Testing
@testable import NenPlayerShell

@Suite("translation target language storage")
struct TranslationPreferenceStoreTests {
    /// Same isolated-suite pattern as `SubtitlePreferenceStoreTests`.
    private func freshDefaults() -> (UserDefaults, () -> Void) {
        let suiteName = "player.nen.tests.translation-target.\(UUID().uuidString)"
        let defaults = UserDefaults(suiteName: suiteName)!
        return (defaults, { defaults.removePersistentDomain(forName: suiteName) })
    }

    @Test("the target language survives a fresh store instance")
    func targetSurvivesRestart() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let writer = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { nil })
        writer.save("tr")

        // A second instance models the app relaunching — the store itself
        // holds nothing in memory that a restart would lose.
        let reader = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { nil })
        #expect(reader.targetLanguage == "tr")
    }

    @Test("the system language seeds the target exactly once")
    func systemLanguageSeedsOnlyOnce() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let first = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { "tr-TR" })
        #expect(first.targetLanguage == "tr", "the region is dropped — ADR-0030's granularity")

        first.save(nil)
        // A relaunch that still reports the same system language must not
        // put the seed back — the user's empty choice is the one that counts.
        let second = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { "tr-TR" })
        #expect(second.targetLanguage == nil, "a cleared target stays cleared")
    }

    @Test("no system language leaves the seed empty")
    func noSystemLanguageSeedsNothing() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let store = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { nil })
        #expect(store.targetLanguage == nil)
    }

    @Test("a region-qualified system language seeds only its primary subtag")
    func regionQualifiedSystemLanguageSeedsPrimarySubtag() throws {
        let (defaults, cleanup) = freshDefaults()
        defer { cleanup() }

        let store = UserDefaultsTranslationPreferenceStore(defaults: defaults, systemLanguage: { "en-US" })
        #expect(store.targetLanguage == "en")
    }
}
