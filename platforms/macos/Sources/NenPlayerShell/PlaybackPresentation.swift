import Foundation
import NenCore

public enum PlaybackPresentation {
    public static func time(milliseconds: UInt64) -> String {
        let totalSeconds = milliseconds / 1_000
        let hours = totalSeconds / 3_600
        let minutes = (totalSeconds % 3_600) / 60
        let seconds = totalSeconds % 60
        if hours > 0 {
            return String(format: "%02llu:%02llu:%02llu", hours, minutes, seconds)
        }
        return String(format: "%02llu:%02llu", minutes, seconds)
    }

    public static func duration(
        position: UInt64,
        total: UInt64?,
        showsRemaining: Bool
    ) -> String {
        guard let total else {
            return "\(time(milliseconds: position)) / --:--"
        }
        if showsRemaining {
            let remaining = total > position ? total - position : 0
            return "−\(time(milliseconds: remaining)) / \(time(milliseconds: total))"
        }
        return "\(time(milliseconds: position)) / \(time(milliseconds: total))"
    }

    /// The left side of the single-row transport: always the current moment.
    public static func elapsed(position: UInt64) -> String {
        time(milliseconds: position)
    }

    /// The right side of the single-row transport: remaining or total time.
    public static func trailingDuration(
        position: UInt64,
        total: UInt64?,
        showsRemaining: Bool
    ) -> String {
        guard let total else { return "--:--" }
        if showsRemaining {
            let remaining = total > position ? total - position : 0
            return "−\(time(milliseconds: remaining))"
        }
        return time(milliseconds: total)
    }

    /// Closed Turkish copy: payloads, paths, engine names, and opaque codes
    /// never enter user-facing text (ADR-0031 Karar 1–2).
    public static func errorMessage(for error: Error) -> String {
        guard let playbackError = error as? FfiPlaybackError else {
            return "İşlem tamamlanamadı."
        }
        switch playbackError {
        case let .LoadFailed(reason):
            switch reason {
            case .notFound: return "Dosya bulunamadı."
            case .unreadable: return "Dosya okunamadı."
            case .unsupportedFormat: return "Bu medya biçimi desteklenmiyor."
            case .networkUnavailable: return "Ağ bağlantısı kullanılamıyor."
            }
        case .NotLoaded:
            return "Önce bir medya açın."
        case .ShutDown:
            return "Oynatma oturumu kapandı."
        case .UnknownTrack:
            return "Seçilen parça kullanılamıyor."
        case .TrackCarriesNoText:
            return "Bu parça metin taşımıyor."
        case .RateOutOfRange:
            return "Bu oynatma hızı kullanılamıyor."
        case .Unsupported:
            return "Bu işlem desteklenmiyor."
        case .ReentrantCall:
            return "İşlem şu anda tamamlanamadı."
        case .InsetOutOfRange:
            // Nothing the user did and nothing they can fix: the value came
            // from this shell's own layout (ADR-0037). It gets a sentence
            // because the set is closed, not because it is meant to be read —
            // the same one a caller-side contract violation gets.
            return "İşlem şu anda tamamlanamadı."
        case .EngineFailure:
            return "Medya oynatılamadı."
        }
    }

    /// What the user is told when a subtitle file they picked was refused at a
    /// security gate (ADR-0031 Karar 1, *geçici* sınıf; NEN-025).
    ///
    /// One sentence per variant, in the user's words. No path, no gate name, no
    /// errno: ADR-0031 Karar 2 keeps the path off every surface, and the reason
    /// a file was refused is not made clearer by naming the rule it broke.
    ///
    /// Only a **rejection** reaches this function. A file that was catalogued
    /// and marked broken says so in the menu instead (Karar 5) and never
    /// interrupts playback.
    public static func subtitleRejectionMessage(for rejection: FfiFileRejection) -> String {
        switch rejection {
        case .symlink: return "Bu bir kısayol; altyazı olarak açılamıyor."
        case .traversal: return "Bu altyazı dosyasının konumu kullanılamıyor."
        case .notRegularFile: return "Bu bir altyazı dosyası değil."
        case .tooLarge: return "Bu altyazı dosyası çok büyük."
        }
    }

    /// What the user is told when the recent-media store's bookmark could not
    /// be saved. Playback continues regardless — this is informational, not a
    /// refusal (NEN-050).
    public static let recentMediaSaveFailedMessage = "Son açılan medya kaydedilemedi."

    /// What the user is told when the recent-media entry can no longer be
    /// resolved, whether because it resolved to nothing or because resolving
    /// it threw (NEN-050). Both cases clear the stored entry.
    public static let recentMediaUnavailableMessage = "Son açılan medya artık kullanılamıyor."

    /// What the user is told when a source `PlayerModel.openMedia(at:)`
    /// cannot open at all — a scheme with no engine support, or a locator a
    /// handoff (NEN-080) could not resolve. One sentence, no scheme name, no
    /// path: the surface is shared with `HandoffIntake` on purpose, so a
    /// refused handoff and a refused drop read identically to the user.
    public static let unsupportedMediaSourceMessage = "Bu medya kaynağı açılamıyor."

    /// What the user is told when "AI ile çevir" could not even start
    /// (`NEN-101`). One sentence per variant, ADR-0031 Karar 1 *geçici*
    /// sınıf — no path, no counts, no provider name: `FfiTranslationStartError`
    /// itself carries none of those (its own doc comment).
    public static func translationStartMessage(for error: FfiTranslationStartError) -> String {
        switch error {
        case .Unusable: return "Bu kaynak kullanılamıyor."
        case .NotTranslatable: return "Bu kaynak çevrilemiyor."
        case .UnknownSourceLanguage: return "Bu kaynağın dili belirlenemedi."
        case .AlreadyTargetLanguage: return "Bu kaynak zaten hedef dilde."
        case .NoDocument: return "Bu kaynağın içeriği henüz okunamadı."
        case .LayoutRefused: return "Bu altyazı çeviri için uygun değil."
        case .InvalidTargetLanguage: return "Hedef dil geçersiz."
        case .StoreUnavailable: return "Çeviri deposu kullanılamıyor."
        }
    }

    /// What the user is told when `translateSelectedSubtitle()` could not get
    /// an embedded row a document to translate (`NEN-044`). Same rule as
    /// `translationStartMessage`: variant only — `FfiEmbeddedDocumentError`
    /// is flat and payload-free, so there is nothing else to say.
    public static func prepareEmbeddedDocumentMessage(for error: FfiEmbeddedDocumentError) -> String {
        switch error {
        case .Unsupported: return "Bu parçadan metin çıkarılamıyor."
        case .ReentrantCall: return "İşlem şu anda tamamlanamadı."
        case .NotLoaded: return "Önce bir medya açın."
        case .ShutDown: return "Oynatma oturumu kapandı."
        case .UnknownTrack: return "Seçilen parça kullanılamıyor."
        case .TrackCarriesNoText: return "Bu parça metin taşımıyor."
        case .EngineFailure: return "Gömülü altyazı metni okunamadı."
        case .Unparseable: return "Gömülü altyazı metni okunamadı."
        }
    }

    /// What the user is told when a started translation job did not produce
    /// an outcome (`NEN-101`). Same rule: variant only, no cue text, no path.
    public static func translationJoinMessage(for error: FfiTranslationError) -> String {
        switch error {
        case .Cancelled: return "Çeviri iptal edildi."
        case .Failed: return "Çeviri tamamlanamadı."
        case .Incomplete: return "Çeviri tamamlanamadı."
        case .AssemblyRejected: return "Çeviri sonucu doğrulanamadı."
        case .StoreFailed: return "Çeviri sonucu kaydedilemedi."
        case .WorkerPanicked: return "Çeviri tamamlanamadı."
        case .AlreadyJoined: return "Çeviri tamamlanamadı."
        }
    }

    /// What the user is told while a translation job is running (`NEN-102`).
    /// Block sequence and that block's own count only — no document-wide
    /// percentage (`TranslationProgressState`'s own doc comment, `NEN-107`),
    /// no cue text, no provider name.
    public static func translationProgressMessage(for state: TranslationProgressState) -> String {
        switch state.phase {
        case .preparing: return "AI çevirisi hazırlanıyor… · \(state.block). blok"
        case .translating: return "AI çevirisi · \(state.block). blok"
        case .finalizing: return "AI çevirisi tamamlanıyor… · \(state.block). blok"
        }
    }
}
