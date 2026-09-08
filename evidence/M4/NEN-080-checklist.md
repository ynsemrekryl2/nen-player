# NEN-080 — Başka bir uygulamanın verdiği medyayı almak

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 (26A5425a) · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0 (mpv 0.41.0_8, Homebrew)

## Neyin değiştiği

ADR-0043'ün üç yüzeyi (`argv` + open-with + `nenplayer://` scheme) tek bir iç
yapıya (`HandoffRequest`) indirgendi.

- **Core** (`core/crates/nen-app/src/handoff.rs`, yeni): `parse_argv` ve
  `parse_url`. Bayrak adları (`--start`/`--start-time`/`--start-position`,
  hem `=değer` hem ayrı token), tanınmayan bayrağın sessizce atlanması,
  `nenplayer://` önekinin soyulması, `#t=<sn>` fragment'i, `file://` →
  yerel yol (host `localhost`/boş, percent-decode), `http`/`https` →
  `nen_app::remote_evidence::validate_url`'in **aynı** şema kapısından geçiş
  (ikinci bir kopya açılmadı — fonksiyon `pub(crate)` yapıldı), pozisyonun
  saniye→milisaniye dönüşümü ve geçersiz değerin sessizce düşmesi.
- **FFI gate** (`core/crates/nen-ffi/src/handoff.rs`, yeni):
  `parse_handoff_argv`/`parse_handoff_url`, `FfiHandoffLocator`/
  `FfiHandoffRequest`/`FfiHandoffRejection`. Locator gate'ten **geri** çıkıyor
  (subtitle path'lerin tersine) — bu yüzden `Debug` elle yazıldı.
- **macOS kabuğu**: `HandoffIntake` (yeni, `NenPlayerShell`) ham argv/URL'yi
  core'a verip `HandoffOutcome`'a çeviriyor; `HandoffCoordinator` (yeni, aynı
  dosya) bunu `PlayerModel`'in yaşam döngüsüyle uzlaştırıyor.
  `PlayerModel.openMedia(at:)` artık `http`/`https` de kabul ediyor (dosya
  güvenlik kapsamı ve sidecar taraması yalnız gerçek dosya URL'lerinde
  çalışıyor); yeni `handleHandoff(_:)` `.medium`/`.none`/`.rejected`'i
  yorumluyor. `NenPlayerApp`'in `AppDelegate`'i `applicationDidFinishLaunching`
  (argv) ve `application(_:open:)` (open-with + scheme) çağrılarını
  `HandoffCoordinator`'a iletiyor. `Info.plist`'e `CFBundleDocumentTypes`
  (`public.movie`/`public.audiovisual-content`, `LSHandlerRank: Alternate`) ve
  `CFBundleURLTypes` (`nenplayer`) eklendi.

## Yol üstünde bulunan ve düzeltilen kusur

Gerçek `.app` kabulü ilk denemede **medyayı açmadı** — pencere boş durumda
kaldı, `defaults`'ta yeni kayıt yok. Geçici `NSLog`/`FileHandle.standardError`
izleriyle ölçüldü: `Window(id:)` + `.windowResizability` sahneli her
başlatmada, pencere görünür hâle gelmeden **önce**, AppKit/SwiftUI
`PlayerRootView`'i bir kez tam söküp (`onDisappear` → `model.shutdown()` →
oturum kapanıyor) ~100 ms içinde yeniden kuruyor (`onAppear`); video
yüzeyinin kendi `NSView`'ı bu sarsıntıyı sağ çıkıyor ama ikinci kuruluşta
`VideoSurface.makeNSView`/`updateNSView` bir daha hiç çağrılmıyor — yani
oturum kalıcı olarak `nil` kalıyor. Bu, **handoff'tan bağımsız**, önceden var
olan bir pencere yaşam döngüsü kırılganlığı (Info.plist'in yeni
`CFBundleDocumentTypes`/`CFBundleURLTypes` girdileri kaldırılınca da aynen
gözlendi — sebep onlar değil); bugüne kadar görünmemesinin nedeni `⌘O`'nun her
zaman bu ~100 ms'lik pencereden çok sonra, kullanıcı eliyle tetiklenmesi.
Handoff bunu **ilk kez** görünür kıldı, çünkü açılışın tam o anında medya
verilebiliyor.

Düzeltme `PlayerRootView`'in mevcut `.onAppear`'ına tek satır: `model.resume()`
— `NEN-046`'nın Dock'tan yeniden açma için zaten kullandığı aynı kendini-onarma
çağrısı (oturum zaten varsa no-op, yoksa hâlâ canlı olan `videoView` ile
`attach(to:)`). Kapsam dışına taşınmadı: tek satır, mevcut mekanizmanın yeniden
kullanımı, yeni bir yaşam döngüsü kavramı icat edilmedi.

## Otomatik kanıt

**Rust** (core, yeni): `handoff.rs` içi **25 unit test** — her iki giriş yolu,
üç bayrak adının iki yazımı da, bilinmeyen bayrağın locator'ı yemediği,
`#t=` fragment'i, `file://` host kapısı, percent-decode hatası, `0` = pozisyon
yok, taşma/ondalık/`NaN`/`1e9` gibi geçersiz değerlerin sessiz düşmesi, colon
içeren dosya adının şema sanılmaması. `tests/guard_handoff_debug.rs` (yeni,
**5 test**, K23): `HandoffLocator`/`HandoffRequest`'in `Debug`'ı ne yolu ne
URL'yi basıyor, türetilmiş ikiz kontrol sızdırıyor.

**FFI** (`nen-ffi`, yeni): `handoff.rs` içi **4 unit test** (argv/URL geçişi,
tipli red). `tests/guard_ffi_handoff_debug.rs` (yeni, **5 test**): locator
gate'ten **çıkarken** de `Debug` sızdırmıyor.

**macOS Swift** (`HandoffTests.swift`, yeni, **19 test**):
- `HandoffIntakeTests` (6): argv/URL → `.medium`/`.none`/`.rejected` eşlemesi.
- `HandoffCoordinatorTests` (5) — bunlardan biri gerçek `.app`'te bulunan
  sıralama kusurunun **regresyon testi**: `application(_:open:)` `argv`'den
  önce ateşlenmiş bir gerçek handoff, sonradan gelen sıradan-başlatmanın
  `.none`'ı tarafından silinmiyor.
- `PlayerModelHandoffTests` (8): uygulama açıkken gelen handoff hemen oynuyor;
  uygulama henüz `attach` olmadan gelen handoff kuyruğa giriyor ve `attach`
  olunca oynuyor (DoD'un "kapalıyken/açıkken" maddesi); uzaktaki locator
  dosya yolu değil URL ile yükleniyor; reddedilen handoff `⌘O`'nun kullandığı
  aynı geçici mesajı gösteriyor; `.none` hiçbir şey yapmıyor; ikinci bir
  handoff öncekini değiştiriyor; ayrıca `openMedia(at:)`'in genişleyen
  kapısının kendisi — bir `http`/`https` URL'yi kabul ediyor, `ftp:` gibi bir
  şemayı yine reddediyor.

Negatif kontrol üç ayrı düzeltmede geri alınıp ayrık ölçüldü: bayrak tanıma
kaldırılınca yalnız kendi testleri kırmızı; `validate_url` çağrısı yerine
"her zaman kabul" konunca yalnız `an_unsupported_scheme_is_refused_by_name`
kırmızı; `HandoffCoordinator`'ın `.none`'ı atlama kuralı kaldırılınca **yalnız**
`realHandoffSurvivesALaterNone` kırmızı (gerçek `.app`'te gözlenen kusurun
birebir aynısı).

Rust workspace **646 passed / 1 ignored** (`handoff.rs` + iki guard dosyası bu
task'ın **39** yeni testi — 25 + 5 core, 4 + 5 FFI; workspace'in geri kalanı
dokunulmadı), `cargo fmt --check` ve `cargo clippy --workspace --all-targets
-- -D warnings` temiz, `cargo deny check` → `advisories ok, bans ok, licenses
ok, sources ok` (yeni dış bağımlılık yok). macOS Swift paketi **221/221**
(`HandoffTests.swift`'in **19** yeni testi dışında hiçbir suite'e dokunulmadı),
iki ardışık temiz koşuda **0 kırmızı**, gerçek libmpv `ContractTests`/
`RedactionTests` dahil.

## Gerçek `.app` kabulü

`bash scripts/build-macos-app.sh` ile ad-hoc imzalı `.app`, `fixtures/media/`
fixture'larıyla:

1. **Uygulama kapalıyken, open-with (Info.plist'in yüzeyi):**
   `open -a NenPlayer.app fixtures/media/contract-clip.mkv` — soğuk başlatma.
   Pencere `contract-clip.mkv` oynatıyor (ekran görüntüsü: SMPTE renk çubuğu
   test görüntüsü + gömülü Türkçe altyazı satırı ekranda), pencere başlığı
   `contract-clip.mkv`, `~/Library/Preferences/player.nen.macos.plist`'in
   `recentMediaEntries`'inin **ilk** kaydı `contract-clip.mkv` (gerçek
   security-scoped bookmark ile).
2. **Uygulama zaten açıkken, ikinci medya:** aynı `.app` çalışırken
   `open -a NenPlayer.app fixtures/media/aspect-4x3-clip.mkv` — pencere
   başlığı anında `aspect-4x3-clip.mkv`'ye döndü.
3. **argv yüzeyi, gerçek Stremio mekanizması (NEN-078 Bulgu 4):** `.app`
   kapalıyken `Contents/MacOS/NenPlayer fixtures/media/aspect-cinema-clip.mkv
   --start=5 --no-terminal` doğrudan çalıştırıldı — pencere başlığı
   `aspect-cinema-clip.mkv` (bilinmeyen `--no-terminal` bayrağı locator'ı
   yemedi; `--start` bu task'ta uygulanmıyor, `NEN-081`).
4. **Negatif:** `Contents/MacOS/NenPlayer ftp://example.test/a.mkv` — pencere
   varsayılan başlığında (`Nen Player`) kaldı, süreç **panik yapmadan** canlı
   kaldı (`ps aux` doğrulandı), hiçbir medya açılmadı.

Her koşu sonrası `defaults delete player.nen.macos` ve
`~/Library/Saved Application State/player.nen.macos.savedState` temizlendi.

## Kapsam dışı bırakılanlar (görüldü, dokunulmadı)

- `nenplayer://` scheme'inin bu makinede uçtan uca doğrulanması: ADR-0043
  Bulgu 9 gereği ad-hoc imzalı `.app` hiçbir göndericiye scheme handler olarak
  görünmüyor (`spctl -a -vv` → rejected). Kod yolu `HandoffIntakeTests` ve
  `HandoffCoordinatorTests`'te tam kanıtlı; gerçek OS yönlendirmesi roadmap
  **S11**'e (Developer ID) bağlı — ADR'nin öngördüğü sınır aynen.
- Başlangıç pozisyonunun uygulanması (`--start=5` yukarıda kabul edildi ama
  seek'e çevrilmedi) → `NEN-081`.
