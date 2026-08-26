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
        case .RateOutOfRange:
            return "Bu oynatma hızı kullanılamıyor."
        case .Unsupported:
            return "Bu işlem desteklenmiyor."
        case .ReentrantCall:
            return "İşlem şu anda tamamlanamadı."
        case .EngineFailure:
            return "Medya oynatılamadı."
        }
    }
}
