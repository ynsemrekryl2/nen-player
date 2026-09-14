import Foundation

/// The provider choices exposed by the M6 settings surface. The model stays
/// separate because the user may keep the same provider and change only its
/// model identity.
public enum TranslationProviderKind: String, CaseIterable, Hashable, Identifiable, Sendable {
    case openRouter
    case openAi

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .openRouter: "OpenRouter"
        case .openAi: "OpenAI"
        }
    }
}

public protocol TranslationPreferenceStoring: AnyObject {
    var targetLanguage: String? { get }
    var providerKind: TranslationProviderKind { get }
    var providerModel: String { get }
    func save(_ targetLanguage: String?)
    func save(provider: TranslationProviderKind, model: String)
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
        static let provider = "translationProvider"
        static let model = "translationModel"
        static let seeded = "translationTargetSeeded"
    }

    public static let defaultProvider: TranslationProviderKind = .openRouter
    public static let defaultModel = "openai/gpt-5.6-luna"

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

    public var providerKind: TranslationProviderKind {
        guard let raw = defaults.string(forKey: Key.provider),
              let provider = TranslationProviderKind(rawValue: raw)
        else { return Self.defaultProvider }
        return provider
    }

    public var providerModel: String {
        guard let model = defaults.string(forKey: Key.model),
              !model.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
        else { return Self.defaultModel }
        return model
    }

    public func save(_ targetLanguage: String?) {
        defaults.set(targetLanguage, forKey: Key.target)
    }

    public func save(provider: TranslationProviderKind, model: String) {
        defaults.set(provider.rawValue, forKey: Key.provider)
        defaults.set(model.trimmingCharacters(in: .whitespacesAndNewlines), forKey: Key.model)
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
