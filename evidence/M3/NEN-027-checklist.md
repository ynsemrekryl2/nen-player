# NEN-027 — altyazının ekrana çıkışı, kanıt kaydı

Koşum: **2026-08-29** · macOS 27.0 · libmpv 2.5.0 (Homebrew, dinamik) ·
Swift paketi ve `.app` ad-hoc imzalı.

Kanıt medyası yalnız depodaki sentetik fixture'lardır:

- otomatik testler: `fixtures/media/contract-clip.mkv`
- çizilen kare kanıtı: `fixtures/media/menu-clip.mkv`'nin geçici bir kopyası,
  yanında elle yazılmış üç cue'luk bir `.srt` (depoya girmedi)

Aşağıdaki hiçbir kayıtta tam dosya yolu, query, motor adı veya özel medya
metadata'sı yok. Kare görüntülerindeki replikler uydurmadır.

## DoD karşılıkları

| DoD maddesi | Durum | Kanıt |
|---|---|---|
| Seçim anında altyazı görünüyor | ❌ **karşılanmadı** | Gerçek `.app`te çizilmiyor — ölçülen kusur aşağıda |
| Seek sonrası cue = `CueIndex` sonucu | ✅ | Rust: 4 000 moment · Swift: gerçek libmpv, 28 moment |
| Cue'suz ana seek → altyazı yok | ✅ | Aynı iki sweep'in boş yarısı + `NEN-027-frame-10s-gap.jpg` |
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

## Gerçek `.app` koşusu — bir kusur ölçüldü

Uygulama derlendi, `Nen Demo.mkv` menüden açıldı ve altyazı yolu ekranda
izlendi. Menü, seçim ve etiket doğru çalışıyor; **çizim çalışmıyor.**

| Gözlem | Sonuç |
|---|---|
| Medya açıldı, otomatik seçim gömülü Türkçe track'i açtı, **oynarken** | ✅ Replik ekranda |
| CC → `Kullanıcı Altyazıları` → `Nen Demo.srt` seçildi | ✅ Aktif nokta ve CC etiketi satıra geçti |
| Aynı anda motorun durumu (geçici ölçüm satırıyla) | `sid=5` · `track-list/count=7` · `sub-text` **dolu** (an: 2,0 s, 1. cue'nun içi) |
| Ekran | ❌ Boş — replik çizilmedi |
| Pencere yeniden boyutlandırılarak tam yeniden çizim zorlandı | ❌ Hâlâ boş |
| Dosya seçiliyken 1–3. cue'ların üzerinden **oynatılarak** geçildi | ❌ Hiçbiri çizilmedi |
| Aynı oturumda gömülü Türkçe track'e dönüldü, **duraklatılmış** 16 s | ❌ O da çizilmedi — oysa aynı track koşunun başında oynarken çizilmişti |

**Ölçümün söylediği:** belge motora ulaşıyor, seçiliyor ve motor onu o an
çizdiğini söylüyor (`sub-text` dolu). Kusur enjeksiyonda değil, **video
yüzeyinin kare bileşiminde**. `vo=image` ile alınan kareler aynı belgeyi
sorunsuz çiziyor, yani mpv'nin altyazı boru hattı sağlam; sorun libmpv render
API'siyle sürülen OpenGL yüzeyinde.

İki açıklama ayakta kaldı ve bunları ayırmak için ölçülü bir koşu daha gerekiyor:

- **A —** render yolu duraklatılmışken OSD'yi kareye bileştirmiyor (o zaman
  oynarken görülen tek başarısızlığın ayrı bir nedeni var).
- **B —** dışarıdan eklenen altyazı bu yolda hiç bileştirilmiyor.

Ölçüm için üründe geçici bir satır kullanıldı; **kaldırıldı** ve `.app` temiz
kaynaktan yeniden derlendi. Kalan görsel adımlar bu kusur kapanmadan
koşulamaz:

| # | Adım | Beklenen |
|---|---|---|
| 1 | `⌘O` ile yanında `.srt` olan bir medya aç | Medya oynuyor |
| 2 | CC panelinden kullanıcı altyazısı satırını seç | Replik **anında** ekranda |
| 3 | İleri sar | Yeni andaki doğru replik anında görünüyor |
| 4 | Cue'suz bir ana sar | Ekran boş |
| 5 | Gömülü bir track'e geç | Tek altyazı görünüyor, iki değil |
| 6 | `Kapalı` | Altyazı gidiyor |

<!-- Koşum sonucu buraya yazılacak -->
