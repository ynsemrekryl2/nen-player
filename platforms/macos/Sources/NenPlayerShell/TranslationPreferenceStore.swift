import Foundation

public protocol TranslationPreferenceStoring: AnyObject {
    var targetLanguage: String? { get }
    func save(_ targetLanguage: String?)
}

/// Persists the AI translation target language (§9, `NEN-101`), on the same
/// terms as `UserDefaultsSubtitlePreferenceStore`.
///
/// Seeds the target from the system locale, but **once** — the same reason
/// `SubtitlePreferenceStore` seeds only once: a tohumlama that ran on every
/// launch would put the language back the moment a user cleared it. No
/// second language table opens here either: the Settings picker keys on
/// `LanguageCatalog.allCodes` (`ADR-0030`'s primary-subtag granularity), and
/// this store carries the same plain primary-subtag string, not a parsed
/// `LanguageTag` — the core already turns an unparseable string into "no
/// preference" at the FFI gate (`nen-ffi/src/translation.rs`'s
/// `InvalidTargetLanguage`), so a second parser here would only be a second
/// place for that leniency to drift.
public final class UserDefaultsTranslationPreferenceStore: TranslationPreferenceStoring {
    private enum Key {
        static let target = "translationTargetLanguage"
        static let seeded = "translationTargetSeeded"
    }

    private let defaults: UserDefaults
    /// How the seed reads the system language — injected on the same terms
    /// as `SubtitlePreferenceStore`'s `systemLanguage`, so a test for the
    /// seed's own logic does not depend on the language of the machine
    /// running it.
    private let systemLanguage: () -> String?

    public init(
        defaults: UserDefaults = .standard,
        systemLanguage: @escaping () -> String? = { Locale.preferredLanguages.first }
    ) {
        self.defaults = defaults
        self.systemLanguage = systemLanguage
        seedIfNeeded()
    }

    public var targetLanguage: String? {
        defaults.string(forKey: Key.target)
    }

    public func save(_ targetLanguage: String?) {
        defaults.set(targetLanguage, forKey: Key.target)
    }

    /// Writes the system language as the target, but only the first time
    /// this store is ever opened on a machine.
    private func seedIfNeeded() {
        guard !defaults.bool(forKey: Key.seeded) else { return }
        defaults.set(true, forKey: Key.seeded)
        guard let language = systemLanguage() else { return }
        // `Locale.preferredLanguages` hands out things like `tr-TR`; the
        // catalog and the settings picker both key on the primary subtag
        // (ADR-0030), so the seed is reduced to it before it is stored.
        let primarySubtag = language
            .split(separator: "-", maxSplits: 1)
            .first
            .map(String.init)?
            .lowercased()
        defaults.set(primarySubtag, forKey: Key.target)
    }
}
