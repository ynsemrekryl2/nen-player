import Foundation
import NenCore

/// What the shell records about the background chain a medium sets off
/// (NEN-131): hash → identity lookup → local scans → OpenSubtitles candidate
/// search → automatic selection / download → translation.
///
/// **Typed on purpose.** Every case carries only bounded fields — a basename
/// the window title already shows, status enums, counts, language tags, a
/// provider/model name, a phase, an error *variant name*. There is no case
/// that could hold a path, a URL, a hash, a private file id or a key, which
/// is what keeps `docs/security-policy.md` §1 (K23) a property of the type
/// rather than of every call site. Presentation derives its text from these
/// fields and nothing else.
public enum PipelineEventKind: Equatable, Sendable {
    public enum MediaSource: Equatable, Sendable {
        case file
        case remote
    }

    public enum IdentityMethod: Equatable, Sendable {
        /// Local file: the provider-compatible hash computed from the file's
        /// two bounded windows, then an exact OpenSubtitles hash lookup.
        case localHash
        /// Remote URL: bounded remote evidence first, then the same lookup.
        case remoteEvidenceThenHash
    }

    public enum AutoSelectionDecision: Equatable, Sendable {
        case localSelected(label: String)
        case providerDownloadStarted(label: String)
        case skipped(reason: AutoSelectionSkipReason)
    }

    public enum AutoSelectionSkipReason: Equatable, Sendable {
        case alreadySelected
        case localSourceArrivedLate
        case automaticDownloadDisabled
        case identityNotVerified
        case alreadyAttemptedToday
        case noMatchingCandidate
    }

    case mediaOpened(name: String, source: MediaSource)
    case playbackReady
    case playbackFailed(error: String)

    case identityLookupStarted(method: IdentityMethod)
    case identityLookupFinished(status: FfiIdentityLookupStatus, label: String?)
    case identityLookupFailed(error: String)

    case sidecarScanFinished(found: Int)
    case embeddedTracksCataloged(count: Int)

    case candidateSearchStarted(languages: [String], hasIdentity: Bool)
    case candidateSearchFinished(report: FfiCandidateSearchReport)
    case candidateSearchFailed(error: String)

    case autoSelection(decision: AutoSelectionDecision)
    case subtitleSelected(label: String)
    case subtitlesTurnedOff

    case downloadStarted(label: String, automatic: Bool)
    case downloadFinished(label: String)
    case downloadFailed(label: String, error: String)
    case downloadUnavailable(label: String)

    case translationStarted(provider: String, model: String, source: String, target: String, label: String)
    case translationPhase(phase: FfiTranslationPhase, done: UInt32, total: UInt32)
    /// `usage` is `FfiTranslationJob.totalUsage()` at the moment each of
    /// these landed (`NEN-138`/`NEN-139`) — the tokens (and, when the
    /// provider itself reports one, the billed cost) that job's provider
    /// calls consumed. `.zero` on `translationFailed`/`translationCancelled`
    /// means no provider call was ever made, not that one was free.
    ///
    /// `duration` (`NEN-140`) is the wall-clock time since the matching
    /// `translationStarted` row — checkpoint/resume and retry waits
    /// included, since that is the time the user actually waited, not a
    /// provider's own server-side processing time.
    case translationFinished(usage: FfiTokenUsage, duration: TimeInterval)
    case translationCancelled(usage: FfiTokenUsage, duration: TimeInterval)
    case translationFailed(error: String, usage: FfiTokenUsage, duration: TimeInterval)

    /// The usage a terminal translation event carries, for
    /// `PipelineEventLog.sessionTokenUsage` (`NEN-139`) to sum without
    /// repeating this switch — `nil` for every other kind, including the
    /// live `translationPhase` row, which never carries usage.
    var terminalTranslationUsage: FfiTokenUsage? {
        switch self {
        case let .translationFinished(usage, _): usage
        case let .translationCancelled(usage, _): usage
        case let .translationFailed(_, usage, _): usage
        default: nil
        }
    }
}

/// One row of the event window. `id` is assigned by the log, monotonically,
/// so a row keeps its identity (and its expanded/collapsed state) when the
/// log is trimmed in front of it or a live row is updated in place.
public struct PipelineEvent: Identifiable, Equatable, Sendable {
    public let id: UInt64
    public let timestamp: Date
    public let kind: PipelineEventKind
}

/// The session-long, in-memory event list behind the `Olaylar` window.
///
/// Never persisted, never mirrored to `os_log`: the window is the only
/// surface (task YAPILMAYACAK), so the K23 argument above stays about one
/// type in one place. Bounded so a long session cannot grow without limit —
/// the oldest rows fall off the front.
@MainActor
public final class PipelineEventLog: ObservableObject {
    public static let defaultCapacity = 500

    @Published public private(set) var events: [PipelineEvent] = []

    public let capacity: Int
    private let now: () -> Date
    private var nextID: UInt64 = 1

    public init(capacity: Int = PipelineEventLog.defaultCapacity, now: @escaping () -> Date = Date.init) {
        self.capacity = max(1, capacity)
        self.now = now
    }

    public var isEmpty: Bool { events.isEmpty }

    /// Sum of every terminal translation event's usage currently in
    /// `events` (`NEN-139`) — a job's usage is recorded exactly once, on
    /// the row reporting it finished, failed or was cancelled, so this
    /// never double-counts a live `translationPhase` row updated in place.
    /// Follows `events`' own lifetime: `clear()` and capacity trimming
    /// reduce it the same way they reduce the window, since this log is
    /// session-only to begin with (see the type's own doc comment).
    public var sessionTokenUsage: FfiTokenUsage {
        events.reduce(FfiTokenUsage.zero) { total, event in
            guard let usage = event.kind.terminalTranslationUsage else { return total }
            return total.adding(usage)
        }
    }

    /// Appends a row and returns its id, for callers that will `update` it.
    @discardableResult
    public func record(_ kind: PipelineEventKind) -> PipelineEvent.ID {
        let id = nextID
        nextID &+= 1
        events.append(PipelineEvent(id: id, timestamp: now(), kind: kind))
        if events.count > capacity {
            events.removeFirst(events.count - capacity)
        }
        return id
    }

    /// Replaces a live row's content in place — translation progress is one
    /// row that moves, not a row per callback. A row that has already been
    /// trimmed away is left alone: the window would not show it anyway.
    public func update(_ id: PipelineEvent.ID, _ kind: PipelineEventKind) {
        guard let index = events.firstIndex(where: { $0.id == id }) else { return }
        events[index] = PipelineEvent(id: id, timestamp: events[index].timestamp, kind: kind)
    }

    public func clear() {
        events.removeAll()
    }

    /// The only way an error reaches a row: its variant name.
    ///
    /// UniFFI's flat error enums print as their bare case (`ProviderTransport`)
    /// under `String(describing:)`, which is exactly the "hata varyantı,
    /// payload'sız" K23 allows. `localizedDescription` is never used — for a
    /// Foundation error it can carry a path or a URL.
    public static func errorName(_ error: any Error) -> String {
        let mirror = Mirror(reflecting: error)
        guard mirror.displayStyle == .enum else {
            // Not an enum — an `NSError` or a struct, whose description can
            // quote a path or a URL. Its type name is all the window gets.
            return String(describing: type(of: error))
        }
        let name = String(describing: error)
        // A payload-carrying case prints as `case(...)`; keep the case name
        // only, so nothing a payload holds can reach the window.
        if let open = name.firstIndex(of: "(") {
            return String(name[..<open])
        }
        return name
    }
}

/// The words the event window uses. Pure, so a test can read every summary
/// and detail without drawing a view.
public enum PipelineEventPresentation {
    public enum Tone: Equatable, Sendable {
        case info
        case success
        case warning
        case failure
    }

    public struct Detail: Equatable, Sendable {
        public let label: String
        public let value: String

        public init(_ label: String, _ value: String) {
            self.label = label
            self.value = value
        }
    }

    public static let emptyMessage = "Henüz olay yok — bir medya açın."

    /// The single line a row shows. Never contains a line break.
    public static func summary(for kind: PipelineEventKind) -> String {
        switch kind {
        case let .mediaOpened(name, source):
            return "Medya açıldı: \(name) (\(sourceLabel(source)))"
        case .playbackReady:
            return "Oynatma hazır"
        case let .playbackFailed(error):
            return "Oynatma başarısız: \(error)"

        case let .identityLookupStarted(method):
            return "Kimlik araması başladı — \(methodLabel(method))"
        case let .identityLookupFinished(status, label):
            switch status {
            case .match:
                return "Hash kimliği bulundu: \(label ?? "?") — OpenSubtitles eşleşmesi"
            case .parsedMatch:
                return "Dosya adıyla medya kimliği bulundu: \(label ?? "?") — otomatik indirme kapalı"
            case .noMatch:
                return "Hash kimliği eşleşmedi — aday araması diğer kanıtlarla sürecek"
            case .ambiguous:
                return "Hash kimliği belirsiz — OpenSubtitles birden fazla eşleşme döndü"
            case .noCredential:
                return "Kimlik araması yapılmadı — OpenSubtitles anahtarı yok"
            case .noHash:
                return "Kimlik araması yapılmadı — medya hash'i hesaplanamadı"
            }
        case let .identityLookupFailed(error):
            return "Kimlik araması başarısız: \(error)"

        case let .sidecarScanFinished(found):
            return found == 0
                ? "Yan dosya taraması bitti — altyazı dosyası yok"
                : "Yan dosya taraması bitti — \(found) altyazı dosyası"
        case let .embeddedTracksCataloged(count):
            return count == 0
                ? "Gömülü altyazı izi yok"
                : "Gömülü altyazı izleri: \(count)"

        case let .candidateSearchStarted(languages, hasIdentity):
            let langs = languages.isEmpty ? "tüm diller" : languages.joined(separator: ", ")
            return "OpenSubtitles araması başladı — \(langs)" + (hasIdentity ? " · doğrulanmış kimlikle" : "")
        case let .candidateSearchFinished(report):
            switch report.status {
            case .cataloged:
                if report.candidateCount == 0 {
                    return "OpenSubtitles: aday yok — denenen: \(methodsLabel(report.attempted))"
                }
                let by = report.foundBy.map(methodLabel) ?? "?"
                return "OpenSubtitles: \(report.candidateCount) aday — \(by) ile bulundu"
            case .noCredential:
                return "OpenSubtitles araması yapılmadı — anahtar yok"
            case .noIdentity:
                return "OpenSubtitles araması yapılmadı — ne hash ne kimlik var"
            }
        case let .candidateSearchFailed(error):
            return "OpenSubtitles araması başarısız: \(error)"

        case let .autoSelection(decision):
            switch decision {
            case let .localSelected(label):
                return "Otomatik seçim: yerel kaynak açıldı — \(label)"
            case let .providerDownloadStarted(label):
                return "Otomatik seçim: OpenSubtitles indirmesi başlatıldı — \(label)"
            case let .skipped(reason):
                return "Otomatik seçim yapılmadı — \(skipReasonLabel(reason))"
            }
        case let .subtitleSelected(label):
            return "Altyazı seçildi: \(label)"
        case .subtitlesTurnedOff:
            return "Altyazı kapatıldı"

        case let .downloadStarted(label, automatic):
            return "İndirme başladı (\(automatic ? "otomatik" : "elle")): \(label)"
        case let .downloadFinished(label):
            return "İndirildi: \(label)"
        case let .downloadFailed(label, error):
            return "İndirme başarısız: \(label) — \(error)"
        case let .downloadUnavailable(label):
            return "İndirme yapılamadı: \(label) — OpenSubtitles anahtarı yok"

        case let .translationStarted(provider, model, source, target, label):
            return "Çeviri başladı: \(source) → \(target) · \(provider)/\(model) · \(label)"
        case let .translationPhase(phase, done, total):
            return "Çeviri — \(phaseLabel(phase)) · \(done)/\(total)"
        case .translationFinished(_, _):
            return "Çeviri tamamlandı"
        case .translationCancelled(_, _):
            return "Çeviri iptal edildi"
        case let .translationFailed(error, _, _):
            return "Çeviri başarısız: \(error)"
        }
    }

    /// The label/value lines shown under an expanded row.
    public static func details(for kind: PipelineEventKind) -> [Detail] {
        switch kind {
        case let .mediaOpened(name, source):
            return [Detail("Ad", name), Detail("Kaynak", sourceLabel(source))]
        case .playbackReady:
            return [Detail("Gömülü izler", "katalogda; menü açılabilir")]
        case let .playbackFailed(error):
            return [Detail("Hata", error)]

        case let .identityLookupStarted(method):
            return [Detail("Yöntem", methodLabel(method)), Detail("Sağlayıcı", "OpenSubtitles")]
        case let .identityLookupFinished(status, label):
            var details = [Detail("Durum", identityStatusLabel(status)), Detail("Yöntem", "OpenSubtitles hash eşleşmesi")]
            if let label {
                details.append(Detail("Kimlik", label))
            }
            details.append(Detail(
                "Otomatik indirme",
                status == .match ? "kimlik doğrulandı, kapı açık" : "kimlik doğrulanmadı, kapı kapalı"
            ))
            return details
        case let .identityLookupFailed(error):
            return [Detail("Hata", error), Detail("Etki", "oynatma sürüyor; kimlik yok")]

        case let .sidecarScanFinished(found):
            return [Detail("Bulunan", "\(found)")]
        case let .embeddedTracksCataloged(count):
            return [Detail("İz sayısı", "\(count)")]

        case let .candidateSearchStarted(languages, hasIdentity):
            return [
                Detail("Diller", languages.isEmpty ? "—" : languages.joined(separator: ", ")),
                Detail("Sıra", "önce hash, aday yoksa doğrulanmış kimlik"),
                Detail("Kimlik", hasIdentity ? "var" : "yok"),
            ]
        case let .candidateSearchFinished(report):
            var details = [Detail("Durum", candidateStatusLabel(report.status))]
            if report.status == .cataloged {
                details.append(Detail("Aday sayısı", "\(report.candidateCount)"))
                details.append(Detail("Denenen", methodsLabel(report.attempted)))
                details.append(Detail("Bulan", report.foundBy.map(methodLabel) ?? "—"))
            }
            return details
        case let .candidateSearchFailed(error):
            return [Detail("Hata", error)]

        case let .autoSelection(decision):
            switch decision {
            case let .localSelected(label):
                return [Detail("Seçilen", label), Detail("Kural", "yerel kaynak sağlayıcıyı bastırır")]
            case let .providerDownloadStarted(label):
                return [Detail("Aday", label), Detail("Kural", "yerel kaynak yok; opt-in otomatik indirme")]
            case let .skipped(reason):
                return [Detail("Sebep", skipReasonLabel(reason))]
            }
        case let .subtitleSelected(label):
            return [Detail("Kaynak", label)]
        case .subtitlesTurnedOff:
            return []

        case let .downloadStarted(label, automatic):
            return [Detail("Kaynak", label), Detail("Tetikleyen", automatic ? "otomatik seçim" : "kullanıcı")]
        case let .downloadFinished(label):
            return [Detail("Kaynak", label)]
        case let .downloadFailed(label, error):
            return [Detail("Kaynak", label), Detail("Hata", error)]
        case let .downloadUnavailable(label):
            return [Detail("Kaynak", label)]

        case let .translationStarted(provider, model, source, target, label):
            return [
                Detail("Kaynak", label),
                Detail("Dil", "\(source) → \(target)"),
                Detail("Sağlayıcı", provider),
                Detail("Model", model),
            ]
        case let .translationPhase(phase, done, total):
            let percent = total == 0 ? 0 : Int((Double(done) / Double(total) * 100).rounded())
            return [Detail("Faz", phaseLabel(phase)), Detail("İlerleme", "\(done)/\(total) · %\(percent)")]
        case let .translationFinished(usage, duration):
            return [Detail("Sonuç", "çeviri menüye eklendi"), Detail("Süre", formattedDuration(duration))]
                + usageDetails(usage)
        case let .translationCancelled(usage, duration):
            return [Detail("Sonuç", "yarım artifact bırakılmadı"), Detail("Süre", formattedDuration(duration))]
                + usageDetails(usage)
        case let .translationFailed(error, usage, duration):
            return [Detail("Hata", error), Detail("Süre", formattedDuration(duration))] + usageDetails(usage)
        }
    }

    /// "Girdi token" / "Cache'lenmiş girdi" / "Çıktı token" — and, only when
    /// the provider itself reported a billed figure, "Tahmini maliyet"
    /// (`NEN-139`). Empty when `usage` is `.zero`: that means no provider
    /// call was ever made (every early-cancellation and prepare/start
    /// failure path), not that one cost nothing — showing zeroes there
    /// would misrepresent silence as measurement.
    static func usageDetails(_ usage: FfiTokenUsage) -> [Detail] {
        guard usage.inputTokens > 0 || usage.outputTokens > 0 else { return [] }
        var details = [
            Detail("Girdi token", "\(usage.inputTokens)"),
            Detail("Cache'lenmiş girdi", "\(usage.cachedInputTokens)"),
            Detail("Çıktı token", "\(usage.outputTokens)"),
        ]
        if let costUsd = usage.costUsd {
            details.append(Detail("Tahmini maliyet", formattedCost(costUsd)))
        }
        return details
    }

    static func formattedCost(_ costUsd: Double) -> String {
        String(format: "$%.4f", costUsd)
    }

    /// `12,4s` under a minute, `1dk 03s` at or above one (`NEN-140`) — never
    /// the raw `TimeInterval`, and never negative (a `duration` this row
    /// computes from two real timestamps cannot go backwards, but clamping
    /// here keeps the presentation layer honest on its own terms).
    static func formattedDuration(_ duration: TimeInterval) -> String {
        let totalSeconds = max(0, duration)
        guard totalSeconds >= 60 else {
            let text = String(format: "%.1f", totalSeconds).replacingOccurrences(of: ".", with: ",")
            return "\(text)s"
        }
        let minutes = Int(totalSeconds) / 60
        let seconds = Int(totalSeconds.rounded()) % 60
        return String(format: "%ddk %02ds", minutes, seconds)
    }

    /// The small header line `PipelineEventLogView` shows above the list
    /// (`NEN-139`) — `nil` while the session has made no provider call yet,
    /// same "no measurement, no line" rule as `usageDetails`.
    public static func sessionUsageSummary(_ usage: FfiTokenUsage) -> String? {
        guard usage.inputTokens > 0 || usage.outputTokens > 0 else { return nil }
        var text = "Bu oturumda: \(usage.inputTokens) girdi · \(usage.outputTokens) çıktı token"
        if usage.cachedInputTokens > 0 {
            text += " (\(usage.cachedInputTokens) cache'lenmiş)"
        }
        if let costUsd = usage.costUsd {
            text += " · \(formattedCost(costUsd))"
        }
        return text
    }

    public static func tone(for kind: PipelineEventKind) -> Tone {
        switch kind {
        case .mediaOpened, .identityLookupStarted, .candidateSearchStarted, .downloadStarted,
             .translationStarted, .translationPhase, .subtitlesTurnedOff, .sidecarScanFinished,
             .embeddedTracksCataloged:
            return .info
        case .playbackReady, .subtitleSelected, .downloadFinished, .translationFinished(_, _):
            return .success
        case let .identityLookupFinished(status, _):
            return status == .match ? .success : .warning
        case let .candidateSearchFinished(report):
            return report.status == .cataloged && report.candidateCount > 0 ? .success : .warning
        case let .autoSelection(decision):
            if case .skipped = decision { return .warning }
            return .success
        case .downloadUnavailable, .translationCancelled(_, _):
            return .warning
        case .playbackFailed, .identityLookupFailed, .candidateSearchFailed, .downloadFailed,
             .translationFailed(_, _, _):
            return .failure
        }
    }

    // MARK: - Vocabulary

    static func sourceLabel(_ source: PipelineEventKind.MediaSource) -> String {
        switch source {
        case .file: "yerel dosya"
        case .remote: "uzak adres"
        }
    }

    static func methodLabel(_ method: PipelineEventKind.IdentityMethod) -> String {
        switch method {
        case .localHash: "yerel dosya hash'i → OpenSubtitles"
        case .remoteEvidenceThenHash: "uzak kanıt → OpenSubtitles hash"
        }
    }

    static func methodLabel(_ method: FfiCandidateSearchMethod) -> String {
        switch method {
        case .hash: "hash"
        case .canonicalIdentity: "kanonik kimlik"
        case .verifiedIdentity: "doğrulanmış kimlik"
        case .parsedIdentity: "ayrıştırılmış kimlik"
        }
    }

    static func methodsLabel(_ methods: [FfiCandidateSearchMethod]) -> String {
        methods.isEmpty ? "—" : methods.map(methodLabel).joined(separator: " → ")
    }

    static func identityStatusLabel(_ status: FfiIdentityLookupStatus) -> String {
        switch status {
        case .match: "eşleşme"
        case .parsedMatch: "dosya adı eşleşmesi"
        case .noMatch: "eşleşme yok"
        case .ambiguous: "belirsiz"
        case .noCredential: "anahtar yok"
        case .noHash: "hash yok"
        }
    }

    static func candidateStatusLabel(_ status: FfiCandidateSearchStatus) -> String {
        switch status {
        case .cataloged: "arama yapıldı"
        case .noCredential: "anahtar yok"
        case .noIdentity: "hash ve kimlik yok"
        }
    }

    static func skipReasonLabel(_ reason: PipelineEventKind.AutoSelectionSkipReason) -> String {
        switch reason {
        case .alreadySelected: "bir altyazı zaten seçili"
        case .localSourceArrivedLate: "yerel kaynak oynatma başladıktan sonra geldi; kendiliğinden açılmaz"
        case .automaticDownloadDisabled: "otomatik OpenSubtitles indirmesi ayarlardan kapalı"
        case .identityNotVerified: "kimlik doğrulanmadı; otomatik indirme kapısı kapalı"
        case .alreadyAttemptedToday: "bu medya için bugün zaten denendi"
        case .noMatchingCandidate: "tercih dillerinde uygun aday yok"
        }
    }

    static func phaseLabel(_ phase: FfiTranslationPhase) -> String {
        switch phase {
        case .preparing: "hazırlanıyor"
        case .translating: "çevriliyor"
        case .finalizing: "tamamlanıyor"
        }
    }
}
