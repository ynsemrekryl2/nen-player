import Foundation

/// Every language Foundation can name, for the two preference pickers
/// (NEN-037).
///
/// No second table is shipped here — the same reasoning ADR-0010 Karar 7
/// gives for the subtitle menu's group headings applies to this list:
/// `SubtitleMenuPresentation.endonym(for:)` is the one place a language code
/// becomes a name, and this catalog reuses it rather than curating its own.
/// A code Foundation cannot name is dropped rather than shown as itself
/// upper-cased — a picker row a user cannot read is worse than a shorter
/// list.
public enum LanguageCatalog {
    /// Primary-subtag codes, sorted by their own endonym.
    public static let allCodes: [String] = {
        Locale.LanguageCode.isoLanguageCodes
            .map(\.identifier)
            .filter { !$0.isEmpty }
            .filter { isNamed($0) }
            .sorted { SubtitleMenuPresentation.endonym(for: $0) < SubtitleMenuPresentation.endonym(for: $1) }
    }()

    /// Whether `SubtitleMenuPresentation.endonym(for:)` actually named this
    /// code, rather than falling back to the code itself.
    private static func isNamed(_ code: String) -> Bool {
        SubtitleMenuPresentation.endonym(for: code) != code.uppercased()
    }
}
