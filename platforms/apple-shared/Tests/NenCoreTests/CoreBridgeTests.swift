import Testing

@testable import NenCore

// NEN-007 iskelet doğrulaması: Swift'ten çağrılan fonksiyon gerçekten cihazda
// çalışan Rust core'a iniyor mu?
//
// XCTest DEĞİL swift-testing kullanılıyor: bu makinede tam Xcode yok, yalnız
// CommandLineTools var ve XCTest.framework onunla gelmiyor.
@Suite("Core FFI bridge")
struct CoreBridgeTests {

    /// `version()` zinciri: nen-domain → nen-app → nen-ffi → Swift.
    /// Sabit string dönseydi bu zincir kanıtlanmış olmazdı; değer
    /// `nen-domain`'deki `CORE_NAME` ile `nen-app`'in sürümünden birleşiyor.
    @Test("version() carries a value across the whole Rust chain")
    func versionCrossesTheFfiBoundary() {
        let reported = version()

        #expect(reported == "nen-core 0.1.0")
    }

    /// FFI çağrısı yan etkisiz ve tekrarlanabilir olmalı — ilk çağrının
    /// kurulum yapıp sonrakilerin bozulduğu bir durum sessizce geçmesin.
    @Test("version() is stable across repeated calls")
    func versionIsStableAcrossCalls() {
        let first = version()
        let calls = (0..<64).map { _ in version() }

        #expect(calls.allSatisfy { $0 == first })
    }
}
