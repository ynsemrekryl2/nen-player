---
id: NEN-045
title: Core playback session across the FFI boundary
milestone: M3
size: M
state: done
depends_on: [NEN-021, NEN-022]
blocks: [NEN-024]
adr: [33]
---

# NEN-045 — Core playback session across the FFI boundary

## Sonuç

Platform kabuğu, çekirdek üzerinden medya yükleyip oynatabilir: tek bir
oturum nesnesi komutları alır, olayları ADR-0011 Karar 1'in kurallarıyla
biriktirir ve kabuk hazır olduğunda teslim eder.

## Kapsam

- `nen-app`'te oturum gövdesi: `ShellEngineBridge`'i saran, kendi pump'ına
  sahip bir `PlaybackSession` (ADR-0033 Karar 1 ve 2)
- Pump adapter'dan **sürekli** çeker; birikme her zaman bounded kuyrukta olur,
  adapter'ın kendi dizisinde değil
- Teslimat **pull**: `drain_events()` kuyrukta ne varsa döner (ADR-0033 Karar 3)
- Yüzey ADR-0011 Karar 3'ün **taban** kümesi: load · play · pause · stop ·
  seek · position · duration · state · track enumeration/selection · olay
  akışı · shutdown; artı capability'ye bağlı `set_rate` / `set_volume`
- `nen-ffi`'de ince geçit: `uniffi::Object`, mevcut tip çevirilerini kullanır
- Yaşam döngüsü: `shutdown()` pump'ı durdurup **join eder**, sonra motoru
  kapatır; `Drop` aynısını idempotent yapar

## YAPILMAYACAK

- **Video yüzeyi, SwiftUI kabuk, transport, `.app` paketleme** → `NEN-024`
- Katalog / altyazı kaynağı yüzeyi → `NEN-026`
- `inject_subtitle` yüzeyi → `NEN-027`
- `extract_text` yüzeyi → `NEN-044`
- Foreign `EventSink` yolunun FFI'da açılması — ADR-0033 Karar 3 bunu portta
  bıraktı
- Pump aralığının platform başına ayarlanabilir yapılması

## Kanıt (DoD)

- [x] Pump çalışırken tüketici beklerken gelen kritik olay seli sınırlı bir
      liste + sonunda `EventsLost` olarak teslim ediliyor
- [x] **Negatif kontrol:** pump devre dışıyken aynı sel sınırsız listeye
      dönüşüyor — garantiyi sağlayan şey pump'ın kendisi
- [x] `PositionChanged` coalescing'i sınırı geçiyor: yüzlerce pozisyon
      kuyrukta tek slot tutuyor
- [x] Kritik olayların sırası korunuyor
- [x] `shutdown()` sonrası komutlar typed error dönüyor — panik yok, sessiz
      no-op yok
- [x] `shutdown()` pump'ı gerçekten durduruyor: join sonrası motora çağrı
      gelmiyor
- [x] Gerçek libmpv, gerçek fixture: oturum üzerinden yüklenen klip oynuyor ve
      `drainEvents()` `StateChanged` + ilerleyen `PositionChanged` veriyor
- [x] K23: oturum locator saklamıyor ve `Debug` ile hiçbir şey sızdırmıyor

## Kanıt kaydı

**Ölçüm tarihi:** 2026-08-26 · Apple M5 · Rust 1.9x (`rust-toolchain.toml`) ·
libmpv 2.5.0 · Xcode 26.6

### Testler

| | Önce | Sonra |
|---|---|---|
| Rust (`cargo test --workspace --no-fail-fast`) | 438 | **452** |
| Swift (`bash scripts/test-macos.sh`) | 16 | **21** |

`cargo clippy --workspace --all-targets` → 0 uyarı · `cargo fmt --all --check`
→ temiz · `bash scripts/test.sh` → 2 test dosyası da geçti.
**Yeni bağımlılık yok:** `core/Cargo.lock` ve `core/deny.toml` değişmedi.

`cargo test` yerine `--no-fail-fast` kullanıldı — `NEN-023`'te ölçüldüğü gibi
`cargo test` ilk kırmızı hedefte duruyor ve kırmızı sayımını olduğundan küçük
gösteriyor.

### Yeni testler (14 Rust + 5 Swift)

`core/crates/nen-app/tests/playback_session.rs` (10) — pump'ın kabuk sormadan
çekmesi · gözetimsiz motorun sınırsız biriktirmesi · çekilen selin tek
`EventsLost` işaretine çökmesi · bin pozisyonun tek slot tutması · kritik olay
sırası · shutdown sonrası **on üç** komutun `ShutDown { operation }` dönmesi ·
shutdown sonrası kuyruğun hâlâ cevap vermesi · shutdown'ın pump'ı durdurması ·
idempotanlık · `Drop`'un hem thread'i hem motoru kapatması.

`core/crates/nen-ffi/src/session.rs` (4) — `EventsLost`'un sayısıyla birlikte
geçmesi · her olayın taşıdığını koruması · hatanın yalnız `operation`'ını
kaybetmesi · descriptor'ın kanonik dil etiketiyle dönmesi.

`platforms/macos/Tests/NenPlaybackMPVTests/SessionTests.swift` (5) — gerçek
libmpv oturum üzerinden oynuyor ve ilerleyen pozisyon raporluyor · seek
cevaplanıyor · track metadata gidip dönüyor · bozuk dosya çökmeden hata
oluyor · shutdown sonrası her şey reddediliyor.

### Negatif kontrol — beş yönde bozuldu, hepsi kırmızıya döndü

| Bozma | Kırmızı |
|---|---|
| Pump hiç spawn edilmiyor | **3** (nen-app) |
| `shutdown()` pump'ı durdurmadan motoru kapatıyor | **2** (nen-app) |
| Shutdown sonrası komut guard'ı kaldırıldı | **1** (nen-app) |
| `PositionChanged` coalescing yerine kritik sınıfa alındı | **7** toplam (2'si nen-app) |
| `EventsLost` FFI sınırında düşürülüyor | **1** (nen-ffi) |

Yeşil ağaçta kırmızı **0**; her bozma geri alındıktan sonra tekrar **0**.

### Ölçüm neyi düzeltti

**Pump'ın değeri "olay teslimatı" değil, `EventsLost`'un var olabilmesi.**
`drain_events()` zaten kendisi çekiyor (`ShellEngineBridge::events()` → `pull()`),
yani düzenli olarak drain eden bir kabuk pump olmadan da her olayı görür.
Pump'ın kapattığı senaryo tam olarak **kimsenin drain etmediği** an: o zaman
birikme adapter'ın sınırsız dizisinde olur ve kuyruğun 64'lük sınırı hiçbir şeyi
sınırlamaz. `an_unwatched_engine_buffers_without_limit` bu boşluğu ölçüyor,
`the_pump_pulls_without_the_shell_asking` kapandığını.

**Outbound olay tipi ayrı yazılmak zorunda kaldı.** `FfiPlaybackEvent`
`EventsLost` taşımıyor ve bu bilinçli — bir adapter onu iddia edebilseydi
teslim edemediği akışı gizleyebilirdi. Ama kabuk **öğrenmek zorunda**
(ADR-0031 Karar 3'ün resync'i). İki yön iki tip oldu: `FfiSessionEvent` yalnız
bu variant'ı fazladan taşıyor.

### Güvenlik (K23)

Track başlığı bu task'la birlikte ilk kez **geri** de geçiyor: `session.tracks()`
descriptor'ları kabuğa döndürüyor. Argüman round-trip olması — değer kabuğun
kendi ürettiği değer, değişmeden sahibine dönüyor; core kaynaklı hiçbir şey
ona eklenmiyor. `nen-ffi/src/playback.rs`'in modül dokümanı "inbound only"
ifadesinden bu kurala güncellendi. Yasak olan **Rust'ın onu basması** ve o ayrı
kural yerinde: `FfiTrackDescriptor`'ın elle yazılmış `Debug`'ı ve
`guard_ffi_track_debug.rs`'in kasıtlı sızdıran ikizi duruyor.

`FfiPlaybackSession` locator, yol veya başlık tutmuyor ve `Debug` **türetmiyor**.
Buna dair test yazılmadı: bir trait'in yokluğunu ölçen test boşta döner ve
`NEN-023`'ün dersi bu — boşta dönen bir testle sınırı örtmek, sınırı yazmaktan
kötüdür.

### Kabul edilen maliyet

`nen-app` bugüne kadar tamamen çağrı-güdümlüydü; artık bir thread'e sahip.
Yaşam döngüsü iki testle bağlandı (`shutdown_stops_the_pump`,
`dropping_the_session_stops_the_pump_and_the_engine`) ve ikisi de pump
durdurulmadığında kırmızı.
