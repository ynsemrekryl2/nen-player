// swift-tools-version: 6.0
//
// NEN-010 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Swift paketi platforms/apple-shared/'dır ve bu paketi tanımaz;
// ayrı durmasının sebebi ADR-0028'in birinci sınırı: spike binding'i ürün
// paketine linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-typed-errors.sh` çalıştırılmalıdır.
//
// spike-async-cancel/apple-harness ile aynı desen — bkz. oradaki yorum.
// Executable target M1-core-spike.md'nin istediği baseline'ı raporlar
// (varyant sayısı · eşleme maliyeti); asıl switch/redaction kanıtı test
// target'ında.

import Foundation
import PackageDescription

let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let staticLibDir = packageDir.appendingPathComponent("generated/lib").path

let package = Package(
    name: "SpikeTypedErrorsHarness",
    platforms: [.macOS(.v14)],
    targets: [
        // UniFFI'ın ürettiği C yüzeyi. Modül adı `spike_typed_errorsFFI` olmak
        // zorunda: üretilen Swift dosyası onu `#if canImport(...)` ile arıyor.
        .systemLibrary(
            name: "spike_typed_errorsFFI",
            path: "generated/spike_typed_errorsFFI"
        ),
        .target(
            name: "SpikeCore",
            dependencies: ["spike_typed_errorsFFI"],
            path: "generated/SpikeCore",
            linkerSettings: [
                .unsafeFlags(["-L\(staticLibDir)", "-lspike_typed_errors"])
            ]
        ),
        .executableTarget(
            name: "SpikeTypedErrors",
            dependencies: ["SpikeCore"],
            path: "Sources/SpikeTypedErrors"
        ),
        .testTarget(
            name: "SpikeTypedErrorsTests",
            dependencies: ["SpikeCore"],
            path: "Tests/SpikeTypedErrorsTests"
        )
    ]
)
