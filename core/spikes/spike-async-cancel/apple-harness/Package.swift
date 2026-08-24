// swift-tools-version: 6.0
//
// NEN-009 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Swift paketi platforms/apple-shared/'dır ve bu paketi tanımaz;
// ayrı durmasının sebebi ADR-0028'in birinci sınırı: spike binding'i ürün
// paketine linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-async.sh` çalıştırılmalıdır.
//
// spike-cue-transfer/apple-harness ile aynı desen — bkz. oradaki yorum.
// Buradaki fark: bir de swift-testing test target'ı var, çünkü DoD I1/I2/I4
// "Swift testi" istiyor, yalnız CLI ölçüm çıktısı değil.

import Foundation
import PackageDescription

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let staticLibDir = packageDir.appendingPathComponent("generated/lib").path

let package = Package(
    name: "SpikeAsyncCancelHarness",
    platforms: [.macOS(.v14)],
    targets: [
        // UniFFI'ın ürettiği C yüzeyi. Modül adı `spike_async_cancelFFI` olmak
        // zorunda: üretilen Swift dosyası onu `#if canImport(...)` ile arıyor.
        .systemLibrary(
            name: "spike_async_cancelFFI",
            path: "generated/spike_async_cancelFFI"
        ),
        .target(
            name: "SpikeCore",
            dependencies: ["spike_async_cancelFFI"],
            path: "generated/SpikeCore",
            linkerSettings: [
                .unsafeFlags(["-L\(staticLibDir)", "-lspike_async_cancel"])
            ]
        ),
        .executableTarget(
            name: "SpikeAsyncCancel",
            dependencies: ["SpikeCore"],
            path: "Sources/SpikeAsyncCancel"
        ),
        .testTarget(
            name: "SpikeAsyncCancelTests",
            dependencies: ["SpikeCore"],
            path: "Tests/SpikeAsyncCancelTests"
        )
    ]
)
