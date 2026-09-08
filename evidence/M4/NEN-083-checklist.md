# NEN-083 — Handoff log yüzeyleri

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0

## Uygulama

- Core ve FFI handoff guard’ları argv sonucu, yerel path, token-bearing uzak
  URL, başlangıç pozisyonu ve tüm hata varyantlarını `Debug` + `Display` ile
  tarıyor.
- `HandoffMetadata` doğrudan redaction guard’ına eklendi; yeni alan/varyant
  eklenirse exhaustive eşleşmeler ve eksiksiz struct kurulumları test kapısını
  kıracak.
- macOS `HandoffOutcome.rejected` artık payload taşımıyor. `HandoffOutcome`,
  `FfiHandoffLocator` ve `FfiHandoffRequest` güvenli string/debug/reflection
  gösterimleri kullanıyor: URL/path `<redacted>`, yalnız güvenli türevler
  görünür.
- Handoff intake’in ham `[String]`/`String` girdileri loglanmıyor; ADR-0043’teki
  locator metadata’sı dışında yeni metadata kanalı açılmadı.

## Guard kanıtı

- `cargo test -p nen-app --test guard_handoff_debug` — **6 passed, 0 failed**.
- `cargo test -p nen-ffi --test guard_ffi_handoff_debug` — **5 passed, 0 failed**.
- `cargo test -p nen-identity --test guard_evidence_debug` — **8 passed, 0 failed**.
- macOS `handoff log surfaces` suite’i — **4 passed, 0 failed**:
  `String(describing:)`, `String(reflecting:)` ve `dump` çıktıları yerel path,
  uzak URL, host, filename ve token içermiyor; tür/şema/uzantı/süre/durum
  gibi güvenli alanlar görünür.

## Negatif kontroller

- Türetilmiş Rust ve Swift taşıyıcılar aynı gizli fixture’ları gerçekten
  yazdırdı; guard’ların needlesız veya boş çıktıyla geçmediği doğrulandı.
- Rust `HandoffLocator` redaksiyonu geçici olarak geri alındığında hedef guard
  beklenen şekilde **exit 101**, 4 failing test ile kırıldı; güvenli kod geri
  yüklendi.
- Swift `HandoffOutcome` redaksiyonu geçici olarak geri alındığında hedef suite
  beklenen şekilde **exit 1**, 8 failing assertion ile kırıldı; güvenli kod geri
  yüklendi.

## Tam doğrulama

- `cargo test --workspace` — exit 0, tüm workspace suite’leri yeşil.
- `cargo fmt --check` — exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — exit 0.
- `cargo deny check` — `advisories ok, bans ok, licenses ok, sources ok`.
- `swift test --no-parallel --package-path platforms/macos` — **239 test / 0
  failure**, 28 suite.
- `bash scripts/test-macos.sh` paralel koşusu handoff suite’leri dahil 238/239
  testte yeşil kaldı; tek kırmızı, önceden belgelenmiş NEN-049 sınıfı
  `PicturelessSurfaceTests` main-actor contention’ıydı. Aynı paket serial
  kapıda 239/239 yeşildir.
- `git diff --check` — exit 0.
