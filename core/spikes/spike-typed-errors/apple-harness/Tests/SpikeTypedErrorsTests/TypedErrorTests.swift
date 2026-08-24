import Testing

import SpikeCore

// NEN-010 — typed error mapping across the FFI boundary (M1-core-spike.md).
// swift-testing, not XCTest: this machine has no full Xcode (aynı desen
// spike-async-cancel/apple-harness/Tests ile).
//
// Aynı 8-yasaklı-desen sözlüğü, `nen-domain/tests/guard_redaction.rs` ve
// `spike-typed-errors/src/lib.rs`'nin Rust testleriyle aynı (K23).
private let privatePath = "/Users/alice/Movies/Really Personal/S01E01.mkv"
private let mediaUrlWithToken =
    "https://media.example.com/stream/season1/ep01.mkv?token=eyJhbGciOiJIUzI1NiJ9.super-secret-payload"

@Suite("NEN-010 typed errors")
struct TypedErrorTests {

    /// DoD #1 — her varyant Swift tarafında `switch` ile, mesaj string'i
    /// parse edilmeden ayrıştırılabiliyor. `default:` YOK: derleyici
    /// exhaustiveness'i zorluyor (DoD #3'ün derleme-zamanı yüzü — ayrıca
    /// `scripts/spike-typed-errors.sh`'ın negatif-kontrol adımı çalışma
    /// zamanında değil, tam olarak bu exhaustiveness'i mekanik olarak kırıp
    /// kanıtlıyor).
    @Test("every variant switches without string parsing")
    func everyVariantSwitchesWithoutStringParsing() throws {
        // Not: uniffi 0.32'de `uniffi::Error` varyant adları PascalCase
        // (`.Parse`), sıradan `uniffi::Enum` varyantları ise camelCase
        // (`.localAsr`) — iki farklı derive makrosunun tutarsız isimlendirme
        // kuralı. Bu spike'ın kendisi bu asimetriyi ortaya çıkardı; kanıt
        // kaydında not edilecek.
        func classify(_ error: AppError) -> String {
            switch error {
            case .Parse(let `extension`, let line):
                return "parse:\(`extension` ?? "none"):\(line)"
            case .Network(let host, let status):
                return "network:\(host):\(status)"
            case .Cancelled:
                return "cancelled"
            case .CapabilityUnavailable(let capability):
                return "capability:\(capability)"
            case .Validation(let field):
                return "validation:\(field)"
            }
        }

        #expect(throws: AppError.self) { try triggerParseError(path: privatePath, line: 7) }
        #expect(throws: AppError.self) {
            try triggerNetworkError(url: mediaUrlWithToken, status: 503)
        }
        #expect(throws: AppError.self) { try triggerCancelled() }
        #expect(throws: AppError.self) {
            try triggerCapabilityUnavailable(capability: .localAsr)
        }
        #expect(throws: AppError.self) {
            try triggerValidationError(field: .subtitlePath)
        }

        do {
            try triggerParseError(path: privatePath, line: 7)
            Issue.record("expected triggerParseError to throw")
        } catch let error as AppError {
            #expect(classify(error) == "parse:mkv:7")
        }

        do {
            try triggerNetworkError(url: mediaUrlWithToken, status: 503)
            Issue.record("expected triggerNetworkError to throw")
        } catch let error as AppError {
            // media.example.com is not on the spike's allowlist.
            #expect(classify(error) == "network:<redacted-host>:503")
        }

        do {
            try triggerNetworkError(url: "https://api.opensubtitles.com/v1/x", status: 200)
            Issue.record("expected triggerNetworkError to throw")
        } catch let error as AppError {
            #expect(classify(error) == "network:api.opensubtitles.com:200")
        }

        do {
            try triggerCapabilityUnavailable(capability: .cloudTranslation)
            Issue.record("expected triggerCapabilityUnavailable to throw")
        } catch let error as AppError {
            #expect(classify(error) == "capability:cloudTranslation")
        }

        do {
            try triggerValidationError(field: .mediaDuration)
            Issue.record("expected triggerValidationError to throw")
        } catch let error as AppError {
            #expect(classify(error) == "validation:mediaDuration")
        }
    }

    /// DoD #2 — payload'lı bir varyantın çıktısı yasaklı desen içermiyor.
    /// NEN-006'nın Rust-only kanıtından farkı: burada FFI sınırını GERÇEKTEN
    /// geçmiş bir hatayı, Swift'in KENDİ varsayılan `String(describing:)`
    /// basımıyla kontrol ediyoruz — Rust'ın elle yazılmış `Debug`'ı Swift
    /// tarafında hiç devrede değil (bkz. `lib.rs` modül dokümanı). Ham değer
    /// hiç sınırı geçmediği için burada da yok.
    @Test("caught error's default Swift printing has no forbidden pattern")
    func defaultSwiftPrintingHasNoForbiddenPattern() {
        let forbidden = [privatePath, mediaUrlWithToken, "media.example.com"]

        func assertSafe(_ error: AppError, _ label: String) {
            let printed = String(describing: error)
            for pattern in forbidden {
                #expect(
                    !printed.contains(pattern),
                    "\(label)'s default Swift printing leaked \(pattern): \(printed)"
                )
            }
        }

        do {
            try triggerParseError(path: privatePath, line: 7)
        } catch let error as AppError {
            assertSafe(error, "parse")
        } catch {
            Issue.record("unexpected error type: \(error)")
        }

        do {
            try triggerNetworkError(url: mediaUrlWithToken, status: 503)
        } catch let error as AppError {
            assertSafe(error, "network")
        } catch {
            Issue.record("unexpected error type: \(error)")
        }
    }
}
