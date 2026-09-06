import Foundation

/// The user's first and second preferred subtitle languages, as the shell
/// holds them — plain primary-subtag strings, the same shape the FFI gate
/// accepts (`nen-ffi/src/subtitles.rs`'s `preferences(primary:secondary:)`).
///
/// A parsed `LanguageTag` is not used here on purpose: the core already turns
/// an unparseable string into "no preference" (see the comment on
/// `preferences` in `nen-ffi`), so a second parser in the shell would only be
/// a second place for that same leniency to drift.
public struct SubtitleLanguagePreferences: Equatable, Sendable {
    public var primary: String?
    public var secondary: String?

    public init(primary: String? = nil, secondary: String? = nil) {
        self.primary = primary
        self.secondary = secondary
    }

    /// A secondary equal to the primary is dropped.
    ///
    /// This is the shell's face of a rule the core already enforces
    /// (`SubtitlePreferences::new`, `nen-domain/src/source.rs`) — it does not
    /// duplicate the effect, it only keeps the settings panel itself from
    /// ever showing a redundant second pick next to the first.
    public func normalized() -> SubtitleLanguagePreferences {
        guard let primary, secondary == primary else { return self }
        return SubtitleLanguagePreferences(primary: primary, secondary: nil)
    }
}

public protocol SubtitlePreferenceStoring: AnyObject {
    var preferences: SubtitleLanguagePreferences { get }
    func save(_ preferences: SubtitleLanguagePreferences)
}

/// Persists the two preferred languages (NEN-037), on the same terms as
/// `UserDefaultsRecentMediaStore`.
///
/// Seeds the primary language from the system locale, but **once**: a
/// tohumlama that ran on every launch would put the language back the moment
/// a user cleared it, and DoD #1 (kept across a restart) and DoD #4
/// (both empty) could then never both hold. The seed itself keeps today's
/// behaviour — a Turkish machine auto-selects a Turkish track — as something
/// the user now sees and can change, rather than a silent read of `Locale`.
public final class UserDefaultsSubtitlePreferenceStore: SubtitlePreferenceStoring {
    private enum Key {
        static let primary = "subtitlePrimaryLanguage"
        static let secondary = "subtitleSecondaryLanguage"
        static let seeded = "subtitlePreferencesSeeded"
    }

    private let defaults: UserDefaults
    /// How the seed reads the system language.
    ///
    /// Injected on the same terms as `PlayerModel`'s `now` — read straight
    /// from `Locale` at the point of use, a test for the seed's own logic
    /// would depend on the language of the machine running it.
    private let systemLanguage: () -> String?

    public init(
        defaults: UserDefaults = .standard,
        systemLanguage: @escaping () -> String? = { Locale.preferredLanguages.first }
    ) {
        self.defaults = defaults
        self.systemLanguage = systemLanguage
        seedIfNeeded()
    }

    public var preferences: SubtitleLanguagePreferences {
        SubtitleLanguagePreferences(
            primary: defaults.string(forKey: Key.primary),
            secondary: defaults.string(forKey: Key.secondary)
        ).normalized()
    }

    public func save(_ preferences: SubtitleLanguagePreferences) {
        let normalized = preferences.normalized()
        defaults.set(normalized.primary, forKey: Key.primary)
        defaults.set(normalized.secondary, forKey: Key.secondary)
    }

    /// Writes the system language as the primary preference, but only the
    /// first time this store is ever opened on a machine.
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
        defaults.set(primarySubtag, forKey: Key.primary)
    }
}
