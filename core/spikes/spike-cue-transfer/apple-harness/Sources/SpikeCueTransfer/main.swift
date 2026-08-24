// NEN-008 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
//   SpikeCueTransfer full     <count>
//   SpikeCueTransfer windowed <count>
//
// Tek çağrıda bir yaklaşım ölçülür: peak RSS'i tek process'te iki yaklaşım
// birbirini kirletir. Bu dosya `main.swift` olduğu için üst seviye kodu ana
// thread'de koşar — "UI thread ne kadar bloklandı" sorusunun ölçülebilmesi
// tam olarak buna dayanıyor.

import Darwin
import Foundation
import SpikeCore

// MARK: - Ölçüm araçları

/// Monotonik saat; NTP düzeltmesinden ve uykudan etkilenmez.
@inline(__always)
func nowNanos() -> UInt64 {
    clock_gettime_nsec_np(CLOCK_UPTIME_RAW)
}

/// Nearest-rank yüzdelik. p95, "en kötü %5'in eşiği" olarak okunur.
func percentile(_ sorted: [UInt64], _ p: Int) -> UInt64 {
    guard !sorted.isEmpty else { return 0 }
    let rank = max(1, (p * sorted.count + 99) / 100)
    return sorted[min(rank, sorted.count) - 1]
}

func ms(_ nanos: UInt64) -> String {
    String(format: "%.3f ms", Double(nanos) / 1_000_000)
}

func us(_ nanos: UInt64) -> String {
    String(format: "%.2f µs", Double(nanos) / 1_000)
}

func mib(_ bytes: UInt64) -> String {
    String(format: "%.1f MiB", Double(bytes) / (1024 * 1024))
}

/// Process'in şu anki ve **tepe** resident set size'ı.
///
/// `resident_size_max` çekirdeğin tuttuğu tepe değerdir — biz örnekleme
/// yapmadığımız için iki ölçüm arasındaki zirveyi kaçırmayız.
func residentSize() -> (current: UInt64, peak: UInt64) {
    var info = mach_task_basic_info()
    var count = mach_msg_type_number_t(
        MemoryLayout<mach_task_basic_info>.size / MemoryLayout<natural_t>.size)

    let result = withUnsafeMutablePointer(to: &info) { pointer in
        pointer.withMemoryRebound(to: integer_t.self, capacity: Int(count)) { rebound in
            task_info(mach_task_self_, task_flavor_t(MACH_TASK_BASIC_INFO), rebound, &count)
        }
    }

    guard result == KERN_SUCCESS else { return (0, 0) }
    return (info.resident_size, info.resident_size_max)
}

/// Deterministik ama sıralı olmayan konumlar: gerçek bir kullanıcı listede
/// zıplar. Sıralı erişim ölçseydik yalnız cache davranışını ölçerdik.
/// Tohum sabit — ölçüm tekrarlanabilir kalsın.
struct Positions {
    private var state: UInt64 = 0x2545_F491_4F6C_DD1D

    mutating func next(below bound: UInt64) -> UInt64 {
        state = state &* 6_364_136_223_846_793_005 &+ 1_442_695_040_888_963_407
        return bound == 0 ? 0 : (state >> 33) % bound
    }
}

/// Optimizer'ın ölçülen çağrıyı silmesini engelleyen ve verinin gerçekten
/// geldiğini kanıtlayan toplam.
var checksum: UInt64 = 0

// `main.swift`'in üst seviye değişkenleri Swift 6'da @MainActor izole; onlara
// dokunan yardımcı da öyle olmak zorunda. Ölçüm zaten ana thread'de koşuyor.
@MainActor
@inline(__always)
func absorb(_ cues: [SpikeCue]) {
    for cue in cues {
        checksum &+= cue.endMs &+ UInt64(cue.text.utf8.count)
    }
}

// MARK: - Argümanlar

let arguments = CommandLine.arguments
guard arguments.count == 3, let count = UInt32(arguments[2]) else {
    FileHandle.standardError.write(
        Data("kullanım: SpikeCueTransfer <full|windowed> <count>\n".utf8))
    exit(2)
}
let approach = arguments[1]

/// Pencere boyu: bir altyazı listesi ekranında aynı anda görünen cue sayısı
/// mertebesinde. Menü/liste UI'ı bundan fazlasını zaten çizmiyor.
let windowSize: UInt32 = 40
/// Rastgele erişim örneklem sayısı — p95'in gürültüden ayrılabilmesi için.
let randomAccessSamples = 10_000
/// Tam liste tekrar sayısı: tek ölçüm p50/p95 vermez, ama her tekrar
/// 50k cue'luk bir tahsis demek, o yüzden az tutuluyor.
let fullListRepeats = 20

let payload = payloadBytes(count: count)
let baselineRss = residentSize().current

// MARK: - Yaklaşım A — tam listeyi FFI'dan geçir

if approach == "full" {
    var samples: [UInt64] = []
    var firstCall: UInt64 = 0
    var received = 0

    for repetition in 0..<fullListRepeats {
        let start = nowNanos()
        let cues = allCues(count: count)
        let elapsed = nowNanos() - start

        samples.append(elapsed)
        if repetition == 0 { firstCall = elapsed }
        received = cues.count
        absorb(cues)
    }

    let rss = residentSize()
    let sorted = samples.sorted()

    print("## Yaklaşım A — tam liste (`allCues(count:)`)")
    print("")
    print("| Ölçüm | Değer |")
    print("|---|---|")
    print("| alınan cue | \(received) |")
    print("| payload (Rust tarafı) | \(mib(payload)) |")
    print("| ilk çağrı | \(ms(firstCall)) |")
    print("| p50 | \(ms(percentile(sorted, 50))) |")
    print("| p95 | \(ms(percentile(sorted, 95))) |")
    print("| min / max | \(ms(sorted.first ?? 0)) / \(ms(sorted.last ?? 0)) |")
    print("| **en uzun tek main-thread bloğu** | **\(ms(sorted.last ?? 0))** |")
    print("| toplam (\(fullListRepeats) tekrar) | \(ms(samples.reduce(0, &+))) |")
    print("| peak RSS | \(mib(rss.peak)) (başlangıç \(mib(baselineRss))) |")
    print("")
    print("checksum: \(checksum)")
    exit(0)
}

// MARK: - Yaklaşım B — pencere / handle

if approach == "windowed" {
    let handleStart = nowNanos()
    let handle = CueHandle(count: count)
    let handleElapsed = nowNanos() - handleStart

    let afterHandleRss = residentSize()
    let bound = UInt64(count > windowSize ? count - windowSize : 0)
    var positions = Positions()

    // 1. Rastgele pencere erişimi — kullanıcı listede zıplıyor.
    var windowSamples: [UInt64] = []
    windowSamples.reserveCapacity(randomAccessSamples)
    for _ in 0..<randomAccessSamples {
        let start = UInt32(positions.next(below: bound))
        let began = nowNanos()
        let window = handle.cues(start: start, len: windowSize)
        windowSamples.append(nowNanos() - began)
        absorb(window)
    }

    // 2. activeCue — playback sırasında her position güncellemesinde sorulan şey.
    let timelineMs = UInt64(count) * 3_000
    var activeSamples: [UInt64] = []
    activeSamples.reserveCapacity(randomAccessSamples)
    var found = 0
    for _ in 0..<randomAccessSamples {
        let at = positions.next(below: timelineMs)
        let began = nowNanos()
        let cue = handle.activeCue(atMs: at)
        activeSamples.append(nowNanos() - began)
        if let cue {
            found += 1
            checksum &+= cue.endMs
        }
    }

    // 3. Dokümanın TAMAMINI pencere pencere gezmek: A'nın tek çağrısının
    //    dürüst karşılığı. "Pencere ucuz" demek, 50k cue'yu görmenin de ucuz
    //    olduğu anlamına gelmiyor — ölçülmezse bu atlanır.
    let sweepStart = nowNanos()
    var swept = 0
    var cursor: UInt32 = 0
    while cursor < count {
        let window = handle.cues(start: cursor, len: windowSize)
        swept += window.count
        absorb(window)
        cursor += windowSize
    }
    let sweepElapsed = nowNanos() - sweepStart

    let rss = residentSize()
    let sortedWindows = windowSamples.sorted()
    let sortedActive = activeSamples.sorted()

    print("## Yaklaşım B — pencere / handle (`CueHandle`)")
    print("")
    print("| Ölçüm | Değer |")
    print("|---|---|")
    print("| cue sayısı (handle) | \(handle.count()) |")
    print("| payload (Rust tarafı) | \(mib(payload)) |")
    print("| handle kurulumu | \(ms(handleElapsed)) |")
    print("| pencere \(windowSize) cue — p50 | \(us(percentile(sortedWindows, 50))) |")
    print("| pencere \(windowSize) cue — p95 | \(us(percentile(sortedWindows, 95))) |")
    print("| pencere — max | \(us(sortedWindows.last ?? 0)) |")
    print("| `activeCue(atMs:)` — p50 | \(us(percentile(sortedActive, 50))) |")
    print("| `activeCue(atMs:)` — p95 | \(us(percentile(sortedActive, 95))) |")
    print("| `activeCue(atMs:)` — max | \(us(sortedActive.last ?? 0)) |")
    print("| **en uzun tek main-thread bloğu** | **\(ms(max(sortedWindows.last ?? 0, sortedActive.last ?? 0)))** |")
    print("| tüm dokümanı pencereyle gezmek | \(ms(sweepElapsed)) (\(swept) cue, \((count + windowSize - 1) / windowSize) çağrı) |")
    print("| RSS handle kurulduktan sonra | \(mib(afterHandleRss.current)) (başlangıç \(mib(baselineRss))) |")
    print("| peak RSS | \(mib(rss.peak)) |")
    print("")
    print("örneklem: \(randomAccessSamples) pencere + \(randomAccessSamples) activeCue "
        + "(\(found)'ünde cue vardı, kalanı cue'lar arası boşluk)")
    print("checksum: \(checksum)")
    exit(0)
}

FileHandle.standardError.write(Data("bilinmeyen yaklaşım: \(approach)\n".utf8))
exit(2)
