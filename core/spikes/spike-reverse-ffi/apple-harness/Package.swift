// swift-tools-version: 6.0
//
// NEN-029 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Swift paketi platforms/apple-shared/'dır ve bu paketi tanımaz;
// ayrı durmasının sebebi ADR-0028'in birinci sınırı: spike binding'i ürün
// paketine linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-reverse-ffi.sh` çalıştırılmalıdır.
//
// spike-async-cancel/apple-harness ile aynı desen — bkz. oradaki yorum.

import Foundation
import PackageDescription

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let staticLibDir = packageDir.appendingPathComponent("generated/lib").path

let package = Package(
    name: "SpikeReverseFFIHarness",
    platforms: [.macOS(.v14)],
    targets: [
        // UniFFI'ın ürettiği C yüzeyi. Modül adı `spike_reverse_ffiFFI` olmak
        // zorunda: üretilen Swift dosyası onu `#if canImport(...)` ile arıyor.
        .systemLibrary(
            name: "spike_reverse_ffiFFI",
            path: "generated/spike_reverse_ffiFFI"
        ),
        .target(
            name: "SpikeCore",
            dependencies: ["spike_reverse_ffiFFI"],
            path: "generated/SpikeCore",
            linkerSettings: [
                .unsafeFlags(["-L\(staticLibDir)", "-lspike_reverse_ffi"])
            ]
        ),
        .executableTarget(
            name: "SpikeReverseFFI",
            dependencies: ["SpikeCore"],
            path: "Sources/SpikeReverseFFI"
        ),
        .testTarget(
            name: "SpikeReverseFFITests",
            dependencies: ["SpikeCore"],
            path: "Tests/SpikeReverseFFITests"
        )
    ]
)
