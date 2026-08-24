// swift-tools-version: 6.0
//
// Apple hedeflerinin ortak Swift paketi (ADR-0006).
//
// `Sources/` YOKTUR: core binding'i üretilen koddur ve commit edilmez.
// Bu paket derlenmeden önce `bash scripts/build-apple.sh` çalıştırılmalıdır;
// script `generated/` altındaki üç hedefi de üretir.

import Foundation
import PackageDescription

// Manifest'in kendi konumundan mutlak yol türetilir; linker'ın çalışma
// dizinine bağlı göreli yol kullanılmaz.
let packageDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let staticLibDir = packageDir.appendingPathComponent("generated/lib").path

let package = Package(
    name: "NenCore",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "NenCore", targets: ["NenCore"])
    ],
    targets: [
        // UniFFI'ın ürettiği C yüzeyi. Modül adı `nen_ffiFFI` olmak zorunda:
        // üretilen Swift dosyası `#if canImport(nen_ffiFFI)` ile arıyor.
        .systemLibrary(
            name: "nen_ffiFFI",
            path: "generated/nen_ffiFFI"
        ),
        // Üretilen Swift binding + Rust statik kütüphanesinin linklenmesi.
        .target(
            name: "NenCore",
            dependencies: ["nen_ffiFFI"],
            path: "generated/NenCore",
            linkerSettings: [
                .unsafeFlags(["-L\(staticLibDir)", "-lnen_ffi"])
            ]
        ),
        .testTarget(
            name: "NenCoreTests",
            dependencies: ["NenCore"],
            path: "Tests/NenCoreTests"
        )
    ]
)
