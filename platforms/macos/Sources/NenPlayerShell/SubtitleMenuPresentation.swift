import Foundation
import NenCore

/// Which column-one row the menu is browsing.
///
/// The shell's own identity for a heading, not the core's. Selection is bound
/// to *this* rather than to a row index because column one grows while the menu
/// is open: the sidecar scan makes `Kullanıcı Altyazıları` appear at position
/// two and pushes every language heading down a row (ADR-0031 Karar 4.2).
public enum SubtitleMenuGroupID: Hashable, Sendable {
    case closed
    case userSubtitles
    case language(String)
    case unknownLanguage

    public init(_ group: FfiMenuGroup) {
        switch group {
        case .closed: self = .closed
        case .userSubtitles: self = .userSubtitles
        case let .language(tag): self = .language(tag)
        case .unknownLanguage: self = .unknownLanguage
        }
    }
}

/// Everything §8's menu says out loud.
///
/// This is the first place the menu's **visible text** exists. `nen-catalog`
/// returns headings and kinds and no words at all (ADR-0010 Karar 7), because a
/// language's name is its own name in every UI language while the rest of the
/// chrome is Turkish. Both halves are the platform's business, and they are
/// written here so a test can read them without drawing a view.
public enum SubtitleMenuPresentation {
    /// The compact transport label for the source currently on screen.
    ///
    /// A token is deliberately resolved through the menu rather than printed:
    /// it is an opaque lifetime-local identity (NEN-026), not user-facing text.
    public static func selectionLabel(
        selectedToken: UInt32?,
        menu: [FfiMenuSection]
    ) -> String {
        guard let selectedToken else { return "Kapalı" }
        for section in menu {
            if let entry = section.entries.first(where: { $0.token == selectedToken }) {
                return entryTitle(entry)
            }
        }
        // A stale token is not evidence that subtitles are off. This state is
        // transient during a refresh; keep the control truthful and generic.
        return "Altyazı"
    }

    // MARK: - Column one

    /// A heading, in the words §8 uses.
    public static func groupTitle(_ group: FfiMenuGroup) -> String {
        switch group {
        case .closed: "Kapalı"
        case .userSubtitles: "Kullanıcı Altyazıları"
        case let .language(tag): endonym(for: tag)
        case .unknownLanguage: "Dil Belirsiz"
        }
    }

    /// A language's name **in that language** — "English", "Français",
    /// "Türkçe" — whatever the UI language is (ADR-0010 Karar 7).
    ///
    /// Foundation already holds this table for every locale the system knows,
    /// so the alternative would be shipping a second, smaller, staler copy of
    /// it. Capitalisation is asked of the same locale on purpose: French
    /// returns "français" and it is French that decides how to raise it.
    ///
    /// A tag the system cannot name comes back as the tag itself, upper-cased.
    /// Inventing a name for a language we cannot name would be worse than
    /// showing the code the container declared.
    public static func endonym(for tag: String) -> String {
        let locale = Locale(identifier: tag)
        guard let name = locale.localizedString(forLanguageCode: tag), !name.isEmpty else {
            return tag.uppercased()
        }
        return name.capitalized(with: locale)
    }

    // MARK: - Column two

    /// The row's own line.
    ///
    /// An embedded track with no title carries an empty label on purpose — the
    /// core refuses to invent one in a single language (NEN-023). Here is where
    /// it is filled in, with the track's own language.
    public static func entryTitle(_ entry: FfiMenuEntry) -> String {
        if !entry.label.isEmpty {
            return entry.label
        }
        if let language = entry.language {
            return endonym(for: language)
        }
        return "Adsız parça"
    }

    /// The quiet second line: why the row cannot be used, or where it came
    /// from.
    ///
    /// The reason wins. A broken row's one job is to say why it is broken
    /// (ADR-0031 Karar 5), and a user reading "OpenSubtitles" under a row they
    /// cannot click learns nothing.
    public static func entrySubtitle(_ entry: FfiMenuEntry) -> String {
        if let defect = entry.defect {
            return reasonLabel(defect)
        }
        return originBadge(entry.kind)
    }

    /// §8's origin badge.
    public static func originBadge(_ kind: FfiSubtitleSourceKind) -> String {
        switch kind {
        case .embedded: "Gömülü"
        case .user: "Kullanıcı"
        case .openSubtitles: "OpenSubtitles"
        case .ai: "AI"
        }
    }

    /// The closed set of two (ADR-0035 Karar 2).
    ///
    /// Adding a third needs its own ADR **and** a test that produces it —
    /// the previous set had a label nothing could ever produce, and it took
    /// until NEN-025 to notice.
    public static func reasonLabel(_ defect: FfiSourceDefect) -> String {
        switch defect {
        case .unreadable: "okunamadı"
        case .malformed: "biçim hatalı"
        }
    }

    /// Whether the row may be clicked. A broken row stays in the list and stays
    /// out of reach (ADR-0031 Karar 5).
    public static func isSelectable(_ entry: FfiMenuEntry) -> Bool {
        entry.defect == nil
    }

    // MARK: - Column two, with nothing in it

    /// What column two says when it has no rows to show.
    ///
    /// Three states, all three reachable, and the order below is the priority.
    /// A medium with nothing to offer says so rather than reporting that
    /// subtitles are off: "off" is only useful news when there was something to
    /// turn on.
    public static func emptyMessage(
        hasAnySource: Bool,
        isScanning: Bool
    ) -> String {
        if hasAnySource {
            return "Altyazılar kapalı."
        }
        return isScanning ? "Altyazılar aranıyor…" : "Bu medya için altyazı bulunamadı."
    }

    /// Shown beside the list while the sidecar scan is still running
    /// (ADR-0031 Karar 4). The menu never waits for it.
    public static let scanningNotice = "Altyazılar aranıyor…"
}
