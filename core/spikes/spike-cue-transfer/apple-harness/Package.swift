// swift-tools-version: 6.0
//
// NEN-008 ölçüm harness'ı — ÜRÜN KODU DEĞİLDİR (CLAUDE.md kural 7).
//
// Ürünün Swift paketi platforms/apple-shared/'dır ve bu paketi tanımaz;
// ayrı durmasının sebebi ADR-0028'in birinci sınırı: spike binding'i ürün
// paketine linklenmez.
//
// `generated/` YOKTUR ve commit EDİLMEZ: derlemeden önce
// `bash scripts/spike-cues.sh` çalıştırılmalıdır.
//
// Ölçtüğü Rust crate'inin İÇİNDE duruyor, core/spikes/ altında kardeşi olarak
// değil: core/Cargo.toml `members = [..., "spikes/*"]` diyor, yani spikes/
// altındaki her dizin bir Cargo paketi sanılır. Böylece spike'ın tamamı tek
// dizin — ADR-0028'in "geri dönüş: dizini sil" maliyeti aynen geçerli.

import Foundation
import PackageDescription

// Manifest'in kendi konumundan mutlak yol — linker'ın çalışma dizinine bağlı
// göreli yol kullanılmaz (platforms/apple-shared/Package.swift ile aynı desen).
let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let staticLibDir = packageDir.appendingPathComponent("generated/lib").path

let package = Package(
    name: "SpikeHarness",
    platforms: [.macOS(.v14)],
    targets: [
        // UniFFI'ın ürettiği C yüzeyi. Modül adı `spike_cue_transferFFI` olmak
        // zorunda: üretilen Swift dosyası onu `#if canImport(...)` ile arıyor.
        .systemLibrary(
            name: "spike_cue_transferFFI",
            path: "generated/spike_cue_transferFFI"
        ),
        .target(
            name: "SpikeCore",
            dependencies: ["spike_cue_transferFFI"],
            path: "generated/SpikeCore",
            linkerSettings: [
                .unsafeFlags(["-L\(staticLibDir)", "-lspike_cue_transfer"])
            ]
        ),
        .executableTarget(
            name: "SpikeCueTransfer",
            dependencies: ["SpikeCore"],
            path: "Sources/SpikeCueTransfer"
        )
    ]
)
