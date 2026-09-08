# NEN-081 — Handoff'un taşıdığı başlangıç pozisyonunu uygulamak

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 (26A5425a) · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0 (mpv 0.41.0_8, Homebrew)

## Neyin değiştiği

`NEN-080` `start_position_ms`'i zaten ayrıştırıp `HandoffOutcome.medium(URL,
startPositionMs:)` ile kabuğa taşıyordu; `PlayerModel.handleHandoff` onu
bilerek kullanmıyordu. Bu task son adımı attı — kapsam yalnız macOS kabuğu
(`platforms/macos/Sources/NenPlayerShell/PlayerModel.swift`), Rust çekirdeğinde
kod değişmedi (yalnız "NEN-081 uygulayacak" diyen yorumlar gerçeğe çevrildi:
`core/crates/nen-app/src/handoff.rs`, `core/crates/nen-ffi/src/handoff.rs`).

**Planlama sırasında bulunan tutarsızlık ve kullanıcı kararı** (bkz.
ADR-0043'ün Notlar bölümü, 2026-09-08 eklentisi): Karar 2 hem pozisyonun
ADR-0042'nin erteleme mekanizmasından (`load` ile birlikte, `FILE_LOADED`
anında) geçmesini hem de süreyi aşan bir değerin sessizce düşüp medyanın
**baştan** açılmasını istiyordu — ama `load` anında süre bilinmiyor (mpv onu
ancak `FILE_LOADED`'da biliyor), dolayısıyla ertelenmiş, süreyi aşan bir seek
medyayı sonda açardı, baştan değil. Kullanıcı kararıyla uygulama yolu netleşti:
pozisyon kabukta (`pendingHandoffStartPositionMs`) tutulur ve `apply(_:)`'ın
`.ready` dalında, mpv'nin süreyi artık okuyabildiği an, **`play()`'den önce**
uygulanır. Süresi okunamayan medya (canlı yayın) için pozisyon yine uygulanır
— kapı yalnız "süre biliniyor **ve** pozisyon ≥ süre" olduğunda kapanır.

- `PlayerModel.openMedia(at:startPositionMs:)`: yeni, varsayılan `nil` —
  `⌘O`, son açılanlar ve sürükle-bırak çağrı yerleri değişmedi.
- `pendingURL` → `pendingOpen: (url:, startPositionMs:)?`: soğuk açılışta
  (`attach(to:)`'tan önce gelen handoff) pozisyon da URL ile birlikte taşınır.
- Yeni `pendingHandoffStartPositionMs`: yüklenmekte olan medyanın bekleyen
  pozisyonu. Her `openMedia` çağrısı öncekini düşürür (aynı anda yalnız bir
  medya yüklenir).
- Yeni `applyHandoffStartPosition()`: `apply(_:)`'ın `.ready` dalında,
  `playWhenReady → session.play()`'den önce çağrılır. Süre okunabiliyorsa ve
  pozisyon ona eşit/aşıyorsa seek **düşürülür** (medya baştan oynar, hata
  yüzeyi yok); süre okunamıyorsa (canlı yayın) pozisyon uygulanır.
- `seek(to:)` iki yeni parametre aldı: `drainingEvents` (handoff yolu `false`
  — `apply(_:)` zaten `consume(_:)`'ın olay döngüsü içinden çalışıyor,
  yeniden drenaj iç içe geçmiş bir ikinci olay grubu besler) ve
  `presentsErrorOnFailure` (handoff yolu `false` — reddedilen bir handoff
  pozisyonu kullanıcının kendi eylemi değil, ADR-0031 Karar 1'in *geçici*
  sınıfı burada yok).
- ADR-0042 Karar 4'ün karşılığı: `apply(.failed)`, `apply(.idle)` ve
  `shutdown()` bekleyen pozisyonu düşürür — yüklemeyi hiç cevaplamayacak bir
  medyaya sonradan gelen bir `.ready` onu tekrar oynatmaz.

## Otomatik kanıt

**macOS Swift, `FakeSession` üzerinden** (`HandoffTests.swift`, yeni **10
test**, `HandoffStartPositionTests`): pozisyonlu handoff `play`'den **önce**
seek ediyor (çağrı sırası ayrıca kaydedilip doğrulandı); pozisyonsuz handoff
hiç seek etmiyor; süreyi aşan ve süreye eşit pozisyon sessizce düşüyor
(`transientMessage == nil`, medya yine açılıyor); süresi okunamayan medyada
(canlı yayın) pozisyon yine uygulanıyor; `⌘O` gibi sıradan açma hiçbir zaman
seek üretmiyor; başarısız yükleme bekleyen pozisyonu düşürüyor (sonraki
ilgisiz bir `.ready` onu tekrar oynatmıyor); ikinci bir handoff birincinin
bekleyen pozisyonunu değiştiriyor; soğuk açılışta (`attach`'tan önce gelen
handoff) pozisyon `attach` sonrası uygulanıyor; motorun reddettiği bir
pozisyon hata göstermiyor ve playback'i bloklamıyor.

`FakeSession`'a (`ShellTestSupport.swift`, paylaşılan tek fake) `callOrder`
eklendi — `seek` ve `play`'in gerçek çağrı sırasını kaydediyor; iki ayrı
sayaç "seek `play`'den önce" iddiasını kanıtlayamazdı.

**macOS Swift, gerçek libmpv üzerinden** (`HandoffTests.swift`, yeni **1
test**, `HandoffStartPositionRealEngineTests`): DoD'un kendi maddesi —
`FakeSession` bir yarışın gerçekten kazanıldığını kanıtlayamaz, `NEN-052`'nin
ölçtüğü yükleme penceresi (2,5–12 ms) tam olarak handoff pozisyonunun
düştüğü pencere. `PlayerModel`'in gerçek `sessionFactory` varsayılanıyla
(`MPVPlaybackEngine` + gerçek, pencereli `MPVVideoView` —
`PicturelessSurfaceTests.windowedSurface()`'in kurduğu düzenin aynısı, daha
önce hiçbir kabuk testi bu ikisini `PlayerModel` üzerinden birlikte
sürmemişti) `contract-clip.mkv` (ölçülen süre **30.008 ms**) 12.000 ms
pozisyonla açılıyor, `model.positionMilliseconds` hedefe **inene kadar
beklenerek** okunuyor (`NEN-052`/`NEN-049`'un öğrettiği desen, tek seferlik
okuma değil) ve 11.900–12.100 ms aralığında iniyor; aynı medya 999.000 ms
pozisyonla yeniden açıldığında (süreyi kat kat aşıyor) motor yine oynuyor,
`transientMessage == nil`, pozisyon 5.000 ms'in altında kalıyor — yani gerçek
başlangıçtan. İki senaryo tek test fonksiyonunda, aynı motor örneğinde
art arda çalışıyor: `swift test` suite'ler arası paralel çalıştığı için her
ek gerçek mpv + pencere örneği bu paketin komşu testlerinin zaten belgelediği
(`NEN-049`, `NEN-066`) main-actor zamanlama contention'ına ekleniyor; iki ayrı
gerçek motor yerine bir tanesi yeterliydi.

Negatif kontrol dört ayrı düzeltmede geri alınıp ayrık ölçüldü:

1. `applyHandoffStartPosition()`'ın seek çağrısı kaldırıldığında yalnız bu
   task'ın pozisyon testleri (`FakeSession` tarafında 5 test:
   seeksBeforePlay, appliesWhenDurationIsUnreadable, secondHandoffReplaces,
   survivesColdLaunchQueue, refusedShowsNoError'ın `callOrder` iddiası;
   gerçek motor tarafında `startPositionAgainstTheRealEngine`'in iniş
   iddiası) kırmızı.
2. Süre kapısı (`target >= duration` kontrolü) kaldırıldığında yalnız
   `startPositionPastDurationIsDropped`, `startPositionAtDurationIsDropped`
   ve gerçek motor tarafında `startPositionAgainstTheRealEngine`'in ikinci
   yarısı (999 s'nin düşmesi) kırmızı.
3. Seek çağrısı `play()`'den **sonraya** taşındığında yalnız
   `startPositionSeeksBeforePlay`'in `callOrder` iddiası kırmızı.

Rust workspace **646 passed / 1 ignored** — bu task Rust'a kod olarak
dokunmadı (yalnız iki yorum güncellendi), sayı değişmedi. `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings` ve `cargo deny check`
(`advisories ok, bans ok, licenses ok, sources ok` — yeni dış bağımlılık yok)
temiz. macOS Swift paketi **221 → 232** (bu task'ın 11 yeni testi, başka
hiçbir suite'e dokunulmadı).

**Paralel koşuda ölçülen, bu task'tan bağımsız bir contention bulgusu.**
Bu oturumun kendisi (masaüstü uygulamasının GPU/renderer süreçleri,
`uptime`'ın gösterdiği yük ortalaması geçici olarak ~4'ten ~9'a çıktı) bu
makinede `bash scripts/test-macos.sh`'ın varsayılan paralel koşusunu, bu
paketin zaten belgelenmiş main-actor zamanlama hassasiyeti olan testlerinde
(`resumeRestartsPolling`, `PicturelessSurfaceTests`'in piksel testi,
`controlsVisible`/`transientMessage` bekleyen birkaç test — hepsi `NEN-049`,
`NEN-066`'nın kaydettiği aynı sınıf) ara sıra kırmızıya döndürdü. Bu task'ın
kendi değişikliğinin bu testlere karışmadığı **izole ölçüldü**:
`swift test --skip HandoffStartPositionRealEngineTests` ile yeni suite
tamamen devre dışı bırakıldığında **aynı** test kümesi, **aynı** oranda
kırmızı kaldı — sebep bu task değil, oturumun kendi yük profili. Kesin kanıt
`swift test --no-parallel` ile: **iki ardışık koşu, 232/232, 0 kırmızı**
(42 sn/koşu). `bash scripts/test-macos.sh`'ın kendisi de (paralel, varsayılan)
ayrı denemelerde temiz çıktı; DoD'un "yeşil" ölçütü seri koşuyla kesin
sağlanmış durumda.

## Gerçek `.app` kabulü

`bash scripts/build-macos-app.sh` ile ad-hoc imzalı `.app`,
`fixtures/media/contract-clip.mkv` (ContractTests'in ölçtüğü süre: **30.008 s**)
ile, `NEN-078` Bulgu 4'ün ölçtüğü gerçek Stremio mekanizmasıyla — doğrudan argv:

1. **Pozisyonlu, geçerli:** `.app` kapalıyken
   `Contents/MacOS/NenPlayer contract-clip.mkv --start=5 --no-terminal`.
   Ekran görüntüsü: transport bar'ın geçen süre etiketi **`00:05`**, duraklat
   ikonu (▐▐) görünür — yani medya zaten oynuyor, 0'dan başlayıp sonra 5'e
   sıçramadı. Kaydırıcı topuzu bar'ın en başında (30 s'lik medyada 5 s'nin
   beklenen konumu).
2. **Pozisyonlu, süreyi aşan:** aynı `.app`, `--start=999` (medyanın 30,008 s
   süresini kat kat aşıyor). Ekran görüntüsü: geçen süre **`00:02`** — medya
   **baştan** açılıp normal oynamaya devam etti (0'dan itibaren birkaç
   saniye geçmiş), hiçbir hata diyaloğu yok, ekranda gömülü Türkçe altyazının
   ilk satırı ("Birinci Türkçe satır") görünüyor — DoD'un "medya yine açılır,
   hata yüzeyi tetiklenmez" maddesi birebir.
3. **Süreç sağlığı:** her iki koşuda da `ps aux` ile süreç canlı kaldı, panik
   yok; `⌘Q` ile temiz kapanış.

Her koşu sonrası `defaults delete player.nen.macos` ve
`~/Library/Saved Application State/player.nen.macos.savedState` temizlendi.

**Kapsam dışı (görüldü, dokunulmadı):** `nenplayer://` scheme'inin bu
makinede uçtan uca doğrulanması hâlâ ADR-0043 Bulgu 9'un notarization
boşluğuna bağlı (roadmap **S11**) — bu task scheme yüzeyine dokunmadı, kod
yolu zaten `NEN-080`'de birim testleriyle kanıtlıydı ve bu task onu
değiştirmedi.

## Doğrulama

```bash
bash scripts/test-macos.sh                                   # 232/232 (bkz. yukarıdaki paralel not)
swift test --package-path platforms/macos --no-parallel       # kesin kanıt: iki ardışık koşu, 232/232
cd core && cargo test --workspace && cargo fmt --check \
  && cargo clippy --workspace --all-targets -- -D warnings && cargo deny check
bash scripts/test.sh && bash scripts/check-docs.sh
```
