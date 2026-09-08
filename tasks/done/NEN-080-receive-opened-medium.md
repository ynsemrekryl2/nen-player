---
id: NEN-080
title: Receive a medium opened by another application
milestone: M4
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-079]
blocks: [NEN-081, NEN-082, NEN-083]
adr: [43]
---

# NEN-080 — Receive a medium opened by another application

## Sonuç

Başka bir uygulamanın Nen Player'a verdiği `file`/`http`/`https` medyası
açılıyor ve `⌘O` ile açılmış gibi aynı yoldan oynuyor — uygulama kapalıyken de,
açıkken de.

## Bağlam

Bugün alıcı yüzey **hiç yok**: `platforms/macos/Resources/Info.plist` ne
`CFBundleDocumentTypes` ne `CFBundleURLTypes` taşıyor,
`platforms/macos/Sources/NenPlayerApp/NenPlayerApp.swift`'in `AppDelegate`'i
yalnız `applicationShouldTerminateAfterLastWindowClosed` ve
`applicationShouldHandleReopen` uyguluyor, `CommandLine` hiç okunmuyor.

Yüzeyin **hangisi** olduğu `NEN-079`'un ADR'sinde kararlaştırılır; bu task onu
uygular. Medya bir kez çözüldükten sonra yeni bir yol açılmaz: mevcut
`PlaybackSession::load(locator)` (`core/crates/nen-app/src/session.rs:150`)
çağrılır, uzak URL'ler `remote_evidence::validate_url`'ün şema kapısından
geçer, açılan medya `RecentMediaStore`'a normal kaydıyla girer.

## Kapsam

- ADR'nin seçtiği alıcı yüzey(ler): `Info.plist` tipleri ve/veya
  `application(_:open:)` ve/veya argv okuması
- Gelen locator'ın ayrıştırılması ve tipli hataya bağlanması — `unwrap`/`expect`
  yok (`security-policy.md` §2)
- Reddedilenler: desteklenmeyen şema, boş/bozuk argüman, medya olmayan girdi —
  hepsi sessizce değil, kullanıcının gördüğü mevcut hata yüzeyiyle
- Uygulama **kapalıyken** açılış ve **zaten açıkken** ikinci medya, aynı davranış
- Açılan medyanın son açılanlar listesine normal kaydı

## YAPILMAYACAK

- Başlangıç pozisyonu → `NEN-081`
- Handoff metadata'sının kimliğe bağlanması → `NEN-082`
- Log denetimi → `NEN-083` (K23 kapalı-küme testi orada)
- Yeni bir medya açma yolu kurmak — mevcut `load` yolu kullanılır
- Yeni HTTP politikası — şema kapısı `validate_url`'de, genişletilmez
- Android tarafı → M10

## Kanıt (DoD)

- [x] Platform testi: verilen `file://` locator'ı medyayı yüklüyor
- [x] Platform testi: `http`/`https` locator'ı aynı yoldan geçiyor
- [x] Negatif: desteklenmeyen şema (ör. `ftp://`, `javascript:`) ve bozuk
      argüman tipli hatayla reddediliyor, panik yok
- [x] Uygulama kapalıyken açılan medya ve açıkken açılan ikinci medya için
      ayrı testler
- [x] Gerçek `.app` kabulü: `open -a` ile verilen fixture oynuyor, medya adı
      kromda doğru, son açılanlar listesine giriyor
- [x] Regresyon: `⌘O` yolu ve `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

ADR-0043'ün üç yüzeyi (`argv` + open-with + `nenplayer://` scheme) tek bir iç
yapıya (`HandoffRequest`) indirgendi. Core'da yeni `nen_app::handoff` (`parse_argv`,
`parse_url`) — bayrak adları, `#t=` fragment'i, `file://`/`http`/`https` çözümü
(uzak şema kapısı `remote_evidence::validate_url`'in **aynısı**, ikinci kopya
açılmadı), pozisyonun saniye→ms dönüşümü, geçersiz değerin sessiz düşmesi.
FFI'da `nen_ffi::handoff` aynı ayrımı gate'e taşıyor — locator gate'ten **geri**
çıktığı için `Debug` elle yazıldı (subtitle path'lerin tersine). macOS'ta yeni
`HandoffIntake` (ham girdi → `HandoffOutcome`) ve `HandoffCoordinator`
(`PlayerModel`'in yaşam döngüsüyle uzlaştırma); `PlayerModel.openMedia(at:)`
artık `http`/`https`'i de kabul ediyor; `Info.plist`'e `CFBundleDocumentTypes`
ve `CFBundleURLTypes` eklendi.

**Yol üstünde bulunan ve düzeltilen kusur:** gerçek `.app` kabulünde ilk
denemede medya **açılmadı** — ölçüm, `Window(id:)` + `.windowResizability`
sahnesinin her başlatmada pencereyi görünür olmadan **önce** bir kez tam söküp
(`onDisappear` → `model.shutdown()`) ~100 ms içinde yeniden kurduğunu, ve video
yüzeyinin ikinci kuruluşta bir daha hiç `attach` edilmediğini gösterdi —
handoff'tan bağımsız, önceden var olan bir pencere yaşam döngüsü kırılganlığı
(Info.plist'in yeni girdileri kaldırılınca da aynen gözlendi). Bugüne kadar
görünmemesinin nedeni `⌘O`'nun bu ~100 ms'lik pencereden çok sonra tetiklenmesi;
handoff bunu ilk kez açığa çıkardı. Düzeltme tek satır: `PlayerRootView`'in
`.onAppear`'ına `model.resume()` — `NEN-046`'nın Dock-yeniden-açma için zaten
kullandığı aynı kendini-onarma çağrısı. Regresyon testi (`HandoffCoordinatorTests
.realHandoffSurvivesALaterNone`) düzeltme geri alınınca tek başına kırmızı
dönüyor.

Rust workspace **646 passed / 1 ignored** (bu task'ın **39** yeni testi: core'da
25 unit + 5 K23 guard, FFI'da 4 unit + 5 K23 guard), `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings` ve `cargo deny check`
(yeni dış bağımlılık yok) temiz. macOS Swift paketi **221/221**
(`HandoffTests.swift`'in **19** yeni testi), iki ardışık temiz koşuda 0
kırmızı, gerçek libmpv `ContractTests`/`RedactionTests` dahil. Negatif kontrol
üç ayrı düzeltmede ayrık ölçüldü (bayrak tanıma, şema kapısı,
`HandoffCoordinator`'ın `.none`-atlama kuralı) — her biri yalnız kendi testini
kırmızıya döndürdü.

Gerçek `.app` kabulü dört senaryoda: (1) uygulama kapalıyken `open -a` ile
`contract-clip.mkv` — oynuyor, pencere başlığı doğru, `recentMediaEntries`'e
giriyor; (2) uygulama açıkken ikinci `open -a` — anında değişiyor; (3) argv
yüzeyi, gerçek Stremio mekanizması (`--start=5 --no-terminal` ile doğrudan
exec) — bilinmeyen bayrak locator'ı yemiyor; (4) negatif — `ftp://` argv'si
panik yaratmadan reddediliyor, süreç canlı kalıyor. `bash scripts/test-macos.sh`
iki ardışık koşuda yeşil (⌘O testleri dahil regresyon yok). Tam kanıt:
`evidence/M4/NEN-080-checklist.md`.
