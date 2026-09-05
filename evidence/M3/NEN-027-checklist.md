# NEN-027 — altyazının ekrana çıkışı, kanıt kaydı

Koşum: **2026-08-29** (ölçüm ve otomatik kanıt) · **2026-09-05** (görsel
kabul koşusu) · macOS 27.0 · libmpv 2.5.0 (Homebrew, dinamik) · Swift paketi ve
`.app` ad-hoc imzalı.

Kanıt medyası yalnız depodaki sentetik fixture'lardır:

- otomatik testler: `fixtures/media/contract-clip.mkv`
- çizilen kare kanıtı ve görsel kabul: `fixtures/media/menu-clip.mkv`'nin
  geçici bir kopyası (`Nen Demo.mkv`), yanında elle yazılmış üç cue'luk bir
  `.srt` (depoya girmedi)

Aşağıdaki hiçbir kayıtta tam dosya yolu, query, motor adı veya özel medya
metadata'sı yok. Kare görüntülerindeki replikler uydurmadır.

## DoD karşılıkları

| DoD maddesi | Durum | Kanıt |
|---|---|---|
| Seçim anında altyazı görünüyor | ✅ | 2026-09-05 koşusu, `NEN-027-selection.png` |
| Seek sonrası cue = `CueIndex` sonucu | ✅ | Rust: 4 000 moment · Swift: gerçek libmpv, 28 moment · görsel: `NEN-027-after-seek.png` |
| Cue'suz ana seek → altyazı yok | ✅ | Aynı iki sweep'in boş yarısı · `NEN-027-frame-10s-gap.jpg` · `NEN-027-gap.png` |
| UI'da renderer implementasyon adı yok | ✅ | grep, aşağıda |

## Otomatik kanıt

**Rust — `cargo test --manifest-path core/Cargo.toml`, 72 hedef yeşil.**

- `nen_app::subtitle_rendering::a_user_file_reaches_the_screen_and_the_cue_matches_the_document_after_every_seek`
  — 4 000 sözde-rastgele moment (sabit tohum). Her birinde `CueIndex`'in
  cevabı ile motorun çizdiğini bildirdiği metin **birebir aynı**; referans
  motor cevabı bütün cue listesini tarayarak buluyor, yani hızlı yol yavaş
  yolla karşılaştırılıyor. Sweep'in 2 266 momenti cue içinde, 1 734'ü
  boşlukta geçti — testin kendi çıktısı, `--nocapture` ile okunur; ikisi de
  > 500 eşiğiyle korunuyor.
- `…::a_moment_past_the_last_cue_draws_nothing` ·
  `…::turning_subtitles_off_takes_the_document_off_screen` ·
  `…::an_embedded_row_is_selected_on_the_engine_rather_than_drawn` ·
  `…::a_row_that_cannot_be_shown_changes_nothing` ·
  `…::loading_another_medium_forgets_the_document` ·
  `…::an_engine_that_cannot_draw_an_external_document_refuses_and_says_so`
  (negatif kontrol: capability yokken tipli refüze, sessiz no-op değil)
- `…::the_engine_native_renderer_passes_the_renderer_contract_kit` — port
  kitinin fake dışındaki ilk gerçek geçişi.
- `nen_ports::contract_renderer_fake` (4) ve
  `nen_ports::contract_renderer_is_not_vacuous` (4) — kit hem referansta
  geçiyor hem de kasten bozulmuş dört renderer'ı kırmızıya çeviriyor.
- K23: `nen_ports::guard_renderer_debug` (3) ve
  `nen_app::guard_renderer_state_debug` (3) — çizilen belge hiçbir `Debug`
  çıktısında görünmüyor; her ikisinde de kasten türetilmiş ikiz gerçekten
  sızdırıyor, yani guard boş değil.

**Swift — `bash scripts/test-macos.sh`, 97/97 yeşil.**

- `ContractTests.theRealAdapterPassesTheSharedContractKit` — paylaşılan kit
  artık enjeksiyon ve çizilen-metin senaryolarını da uyguluyor: gerçek
  libmpv'de belge kabul ediliyor, kendi anında çiziliyor ve `Kapalı`
  sonrasında ekran boşalıyor.
- `SubtitleRenderingTests.aUserFileIsDrawnAndTheCueMatchesTheCoreAfterEverySeek`
  — gerçek motorda 28 moment (14'ü cue içinde, 14'ü boşlukta); her birinde
  ekrandaki metin ile çekirdeğin o an için verdiği cevap birebir aynı.
- `…anInjectedDocumentIsNotOneOfTheMediumsTracks` — enjekte edilen belge
  `tracks(kind:.subtitle)`'da görünmüyor (ADR-0013 Karar 5).
- `…showingASecondDocumentReplacesTheFirstRatherThanStacking` — üç kez
  gösterildikten sonra da tek belge yüklü.
- `…anInjectedDocumentSurvivesNothingOfThePreviousMedium` — yeni medyadan
  sonra eski id'ye dokunulmuyor.
- `…aUserFileReplacesAnEmbeddedTrackThatIsAlreadyDrawing` — kabuğun gerçek
  sırası: önce otomatik seçim gömülü track'i açıyor, sonra kullanıcı kendi
  dosyasını seçiyor. Gerçek motorda dosyanın cue'su ekrana geçiyor.

## Çizilen kareler (ekran kontrolü olmadan)

Adapter'ın kendi başlangıç ayarlarıyla kurulan bir motora belge `memory://`
ile verildi ve mpv'nin çizdiği kareler doğrudan dosyaya alındı. Bunlar
uygulamanın ekran görüntüsü **değil** — motorun çizdiği karenin kendisi:

| Kare | An | Ekranda |
|---|---|---|
| `NEN-027-frame-02s-cue1.jpg` | 2,0 s | 1. replik, Türkçe diyakritikleriyle |
| `NEN-027-frame-07s-cue2.jpg` | 7,0 s | 2. replik |
| `NEN-027-frame-10s-gap.jpg` | 10,0 s | **boş** — cue'suz an |
| `NEN-027-frame-12s-cue3.jpg` | 12,5 s | 3. replik |

Bunun `sub-text` okumasına eklediği şey piksel: metin gerçekten çiziliyor ve
`memory://` üzerinden geçen UTF-8 bozulmadan varıyor.

## grep — UI çizim yolunu bilmiyor

```
$ grep -rniE "sub-add|memory://|injectSubtitle|sub-text|renderedSubtitleText|\
SubtitleRenderer|EngineNativeRenderer" platforms/macos/Sources/NenPlayerShell/
(eşleşme yok)

$ grep -rn "embeddedTrackOf\|selectTrack" platforms/macos/Sources/NenPlayerShell/
(eşleşme yok)
```

İkincisi bu task'ın taşıdığı şeyi gösteriyor: "gömülü track mi, dosya mı"
ayrımı kabuktan tümüyle kalktı (ADR-0013 Karar 3). Kabuğun motora dair tek
teması `MPVPlaybackEngine(videoView:)` kurulumu ve `MPVVideoView` yüzeyi —
ikisi de NEN-022/NEN-024'ten geliyor, bu task'ta değişmedi ve hiçbiri
**çizim** implementasyonunu adlandırmıyor.

## 2026-08-29 — kusur ölçümü (tarihsel kayıt)

Uygulama derlendi, `Nen Demo.mkv` menüden açıldı ve altyazı yolu ekranda
izlendi. Menü, seçim ve etiket doğru çalışıyordu; **çizim çalışmıyordu.**

| Gözlem | Sonuç |
|---|---|
| Medya açıldı, otomatik seçim gömülü Türkçe track'i açtı, **oynarken** | ✅ Replik ekranda |
| CC → `Kullanıcı Altyazıları` → `Nen Demo.srt` seçildi | ✅ Aktif nokta ve CC etiketi satıra geçti |
| Aynı anda motorun durumu (geçici ölçüm satırıyla) | `sid=5` · `track-list/count=7` · `sub-text` **dolu** (an: 2,0 s, 1. cue'nun içi) |
| Ekran | ❌ Boş — replik çizilmedi |
| Pencere yeniden boyutlandırılarak tam yeniden çizim zorlandı | ❌ Hâlâ boş |
| Dosya seçiliyken 1–3. cue'ların üzerinden **oynatılarak** geçildi | ❌ Hiçbiri çizilmedi |
| Aynı oturumda gömülü Türkçe track'e dönüldü, **duraklatılmış** 16 s | ❌ O da çizilmedi — oysa aynı track koşunun başında oynarken çizilmişti |

O gün ölçümün söylediği şuydu: belge motora ulaşıyor, seçiliyor ve motor onu o
an çizdiğini söylüyor (`sub-text` dolu); kusur enjeksiyonda değil, **video
yüzeyinin kare bileşiminde**. Ölçüm için üründe kullanılan geçici satır
kaldırıldı ve `.app` temiz kaynaktan yeniden derlendi.

**Kök neden `NEN-066` içinde bulundu ve kapandı:** çizim zaten yapılıyordu,
134 pt'lik cam transport o piksel bandını örtüyordu. ADR-0037 ile kabuk görünür
kromun alt inset'ini playback session'a taşıyor, adapter bunu `sub-pos`'a
mapliyor ve krom gizlenince sıfırlıyor. `NEN-060` aynı kök nedene katlanıp
`canceled` oldu. Bu yüzden aşağıdaki görsel koşu `NEN-066` kapandıktan sonra
yürütüldü.

## 2026-09-05 — görsel kabul koşusu

`bash scripts/build-macos-app.sh` ile derlenen, `codesign --verify --deep
--strict` geçen ad-hoc imzalı `.app`. Medya `Nen Demo.mkv` (20 s, 160×90, dört
gömülü track), yanında elle yazılmış üç cue'luk `Nen Demo.srt`:

| Cue | Aralık | Kullanıcı dosyası | Gömülü Türkçe track |
|---|---|---|---|
| 1 | 2 → 6 s | dolu | 1 → 4 s dolu |
| — | 6 → 9 s | boş | boş |
| 2 | 9 → 12 s | dolu | 10 → 13 s dolu |
| — | 12 → 16 s | **boş** | boş |
| 3 | 16 → 19 s | dolu | boş |

Konum belirsizliği bırakmamak için seek, slider sürüklenerek değil **±5 sn
transport düğmeleriyle** yapıldı ve her karede geçen süre etiketi okundu.
Adımların çoğu **duraklatılmış** durumda koşuldu: 2026-08-29'da başarısız olan
tam olarak buydu.

| # | Adım | Beklenen | Sonuç |
|---|---|---|---|
| 1 | `⌘O` ile yanında `.srt` olan medyayı aç | Medya oynuyor | **Geçti.** Otomatik seçim gömülü Türkçe track'i açtı, replik oynarken ekranda. |
| 2 | 00:05'e git (gömülü track'in boşluğu), duraklat, CC → `Kullanıcı Altyazıları` → `Nen Demo.srt` | Replik **anında** ekranda | **Geçti.** Ekran seçimden önce boştu; seçimle birlikte 1. replik çizildi — oynatma gerekmedi. `NEN-027-selection.png` |
| 3 | +5 sn → 00:10 | Yeni andaki doğru replik anında | **Geçti.** 2. replik çizildi, 1. replik değil. `NEN-027-after-seek.png` |
| 4 | +5 sn → 00:15 | Ekran boş | **Geçti.** Cue'suz anda hiçbir replik kalmadı. `NEN-027-gap.png` |
| 5 | 00:11'e dön (iki kaynağın da cue'su var), CC → gömülü `Türkçe` | Tek altyazı görünüyor, iki değil | **Geçti.** Yalnız gömülü replik çizildi; kullanıcı dosyasının repliği ekrandan kalktı, üst üste binmedi. `NEN-027-embedded.png` |
| 6 | CC → `Kapalı` | Altyazı gidiyor | **Geçti.** Panel `Altyazılar kapalı.`, CC etiketi `Kapalı`; aynı an (00:11) boşaldı. |

Ek koşu — **oynarken**: kullanıcı dosyası yeniden seçildi (00:11'de 2. replik
yine anında çizildi), 00:02'ye sarılıp oynatıldı. 1. replik oynarken görünür
kaldı, 6–9 s boşluğunda ekran boşaldı, 9. saniyede 2. replik geldi ve 12–16 s
boşluğunda yine boşaldı. Krom gizlenince replik alt banda indi (ADR-0037).

Dört kare pencerenin kendi dikdörtgeninden alındı; hiçbirinde tam dosya yolu,
dosya seçim diyaloğu, query veya özel medya metadata'sı yok — yalnız basename
şeridi (`Nen Demo.mkv`) görünüyor. Replikler uydurmadır.

## Kapılar (2026-09-05)

- `cargo test --manifest-path core/Cargo.toml`: **72 hedef · 560 passed ·
  0 failed · 1 ignored** (benchmark).
- `bash scripts/test-macos.sh`: temiz koşuda **151 test / 16 suite / 0 failure**.
  İlk tam koşu `ContractTests` içinde bir kez kırmızı verdi; `NEN-070`
  koşusunda da kaydedilen, gerçek libmpv suite'lerinin paralel koşumundaki
  aynı kararsızlık. `NEN-049` bu gözlemi bekleyen açık task'tır — o koşunun
  hata metni saklanmadı, ikinci tam koşu baştan sona yeşil.
- `bash scripts/test.sh`: iki shell test dosyası yeşil.
- `bash scripts/build-macos-app.sh`: exit 0 ·
  `codesign --verify --deep --strict`: exit 0.
- `bash scripts/check-docs.sh`: exit 0.
- Yukarıdaki iki grep bugünün ağacında yeniden koşuldu: ikisi de eşleşmesiz.

## Koşum sonucu

**GEÇTİ.** DoD'nin dört maddesinin dördü de karşılandı. 2026-08-29'da
karşılanamayan tek madde — "seçim anında altyazı görünüyor" — `NEN-066`'nın
kapattığı kök nedenin ardından 2026-09-05'te duraklatılmış durumda, oynarken,
seek sonrasında ve gömülü ↔ kullanıcı geçişinde ayrı ayrı doğrulandı.
