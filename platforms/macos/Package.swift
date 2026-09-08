// swift-tools-version: 6.0
//
// macOS platform package (ADR-0006: platform kabukları ve adapter'ları
// `platforms/` altında yaşar).
//
// The libmpv adapter lives here rather than in a Rust crate — ADR-0012
// Karar 2. Build with `bash scripts/test-macos.sh`, which regenerates the
// core binding first; `generated/` is not committed.

import PackageDescription

let package = Package(
    name: "NenPlaybackMPV",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "NenPlaybackMPV", targets: ["NenPlaybackMPV"]),
        .library(name: "NenRemoteEvidenceHTTP", targets: ["NenRemoteEvidenceHTTP"]),
        .library(name: "NenPlayerShell", targets: ["NenPlayerShell"]),
        .executable(name: "NenPlayer", targets: ["NenPlayerApp"])
    ],
    dependencies: [
        .package(path: "../apple-shared")
    ],
    targets: [
        // libmpv itself. `pkgConfig` keeps the Homebrew prefix out of the
        // repository: ADR-0012 Karar 3 links dynamically during development,
        // and bundling is NEN-043's job.
        .systemLibrary(
            name: "Cmpv",
            pkgConfig: "mpv",
            providers: [.brew(["mpv"])]
        ),
        .target(
            name: "NenPlaybackMPV",
            dependencies: [
                "Cmpv",
                .product(name: "NenCore", package: "apple-shared")
            ]
        ),
        .target(
            name: "NenRemoteEvidenceHTTP",
            dependencies: [
                .product(name: "NenCore", package: "apple-shared")
            ]
        ),
        .target(
            name: "NenPlayerShell",
            dependencies: [
                "NenPlaybackMPV",
                "NenRemoteEvidenceHTTP",
                .product(name: "NenCore", package: "apple-shared")
            ]
        ),
        .executableTarget(
            name: "NenPlayerApp",
            dependencies: ["NenPlayerShell"]
        ),
        .testTarget(
            name: "NenPlaybackMPVTests",
            dependencies: ["NenPlaybackMPV"]
        ),
        .testTarget(
            name: "NenRemoteEvidenceHTTPTests",
            dependencies: [
                "NenRemoteEvidenceHTTP",
                .product(name: "NenCore", package: "apple-shared")
            ]
        ),
        .testTarget(
            name: "NenPlayerShellTests",
            dependencies: [
                "NenPlayerShell",
                .product(name: "NenCore", package: "apple-shared")
            ]
        )
    ]
)
