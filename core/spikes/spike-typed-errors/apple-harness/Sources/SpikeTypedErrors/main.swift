// NEN-010 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
//   SpikeTypedErrors
//
// M1-core-spike.md'nin istediği baseline'ı raporlar: varyant sayısı ve
// "eşleme maliyeti" — bir hata Rust tarafında fırlatılıp Swift tarafında
// yakalanıp `switch` ile ayrıştırılana kadarki round-trip. Invariant'ların
// (I3: string parse gerektirmiyor) pass/fail kanıtı burada DEĞİL,
// Tests/SpikeTypedErrorsTests/TypedErrorTests.swift'te — bu dosya yalnız
// baseline sayıları raporlar.

import Darwin
import Foundation
import SpikeCore

@inline(__always)
func nowNanos() -> UInt64 {
    clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
}

func percentile(_ sorted: [UInt64], _ p: Int) -> UInt64 {
    guard !sorted.isEmpty else { return 0 }
    let rank = max(1, (p * sorted.count + 99) / 100)
    return sorted[min(rank, sorted.count) - 1]
}

func us(_ nanos: UInt64) -> String {
    String(format: "%.2f µs", Double(nanos) / 1_000)
}

print("## Bağlam")
print("")
print("- cihaz: \(ProcessInfo.processInfo.processorCount) çekirdek")
print("- OS: \(ProcessInfo.processInfo.operatingSystemVersionString)")
print("- tarih: \(Date())")
print("")

let variantCount = 5
print("## Varyant sayısı")
print("")
print("`AppError`: \(variantCount) varyant (Parse, Network, Cancelled, CapabilityUnavailable, Validation).")
print("")

// Eşleme maliyeti: throw → catch → switch round-trip. `triggerParseError` en
// pahalı varyant (iki alan + Optional<String>); tek bir örnek kullanılıyor
// çünkü ölçülen şey "hangi varyant" değil "FFI + switch mekanizmasının
// sabit maliyeti" — beşi de aynı `RustBuffer` decode + enum-discriminant
// switch yolundan geçiyor (bkz. `spike_typed_errors.swift`'in üretilen
// `FfiConverterTypeAppError`'ı).
func measureMappingCost(repeats: Int) -> [UInt64] {
    var samples: [UInt64] = []
    samples.reserveCapacity(repeats)
    for i in 0..<repeats {
        let start = nowNanos()
        do {
            try triggerParseError(path: "/dev/null", line: UInt32(i))
        } catch let error as AppError {
            switch error {
            case .Parse(let ext, let line):
                _ = (ext, line)
            default:
                break
            }
        } catch {
            break
        }
        samples.append(nowNanos() - start)
    }
    return samples
}

let repeats = 10_000
let samples = measureMappingCost(repeats: repeats).sorted()

print("## Eşleme maliyeti — throw → catch → switch round-trip")
print("")
print("- repeats: \(repeats)")
print("")
print("| p50 | p95 | max |")
print("|---|---|---|")
print("| \(us(percentile(samples, 50))) | \(us(percentile(samples, 95))) | \(us(samples.last ?? 0)) |")
