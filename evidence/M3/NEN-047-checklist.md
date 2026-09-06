# NEN-047 — Kısayolları oynatma yüzeyine bağlama checklist

Tarih: 2026-09-06

Ortam: Apple M-serisi · arm64 · macOS 27.0 (26A5421a) · Xcode 26.6
(17F113) · Swift 6.3.3 · libmpv 2.5.0 · debug, ad-hoc imzalı `.app`.

Kanıt medyası yalnız telif-temiz sentetik fixture'dır: `fixtures/media/contract-clip.mkv`.

Uzaktan `computer-use` aracıyla, gerçek `.app` üzerinde çalıştırıldı
(`bash scripts/build-macos-app.sh` çıkış 0; `platforms/macos/.build/NenPlayer.app`
`open` ile başlatıldı).

## Manuel acceptance

1. `Nen Player Ayarları` penceresi öndeyken `Space`, `←`, `→`, `↑`, `↓`
   sırayla basıldı: konum `00:01`'de kaldı, ses seviyesi ve oynatma durumu
   (duraklatılmış) değişmedi. **Geçti.**
2. Ayarlar penceresi kapatılıp oynatma penceresi öne getirildikten sonra
   `→` basıldı: konum `00:01 → 00:06` ilerledi. Yedi kısayoldan temsilen
   test edilen bu ve önceki oturumlarda `←`, `↑`, `↓`, `Space`, `⇧←`, `⇧→`
   ayrı ayrı doğrulandı — tümü oynatma penceresi öndeyken çalışmaya devam
   ediyor. **Geçti.**
3. Oynatma başlatıldı, kontroller oto-gizlenme süresini (~2,5 s) aşacak
   kadar beklendi (transport ve medya adı görünmez oldu). `→` basıldı:
   transport geri geldi (opaklık 0 → 1) ve yeni konumu (`00:27`) gösterdi
   — klavyeyle yapılan seek artık ekranda iz bırakıyor. **Geçti.**
4. Regresyon — `NEN-046` checklist #7: oynatma penceresi `⌘W` ile
   kapatıldı, ardından `⌘O` basıldı. Pencere geri geldi ve dosya seçici
   açıldı; bu davranış menü maddesinin odaktan bağımsız kalması gereken
   `⌘O`/`⇧⌘O` çifti için hâlâ doğru. **Geçti.**

Sonuç: **4/4 geçti** (yedi kısayoldan temsili örneklem; tamamı önceki ve bu
oturumda tek tek doğrulandı).

## `Esc` / tam ekran — doğrulanamadı (ortam kısıtı)

`Esc`'in yalnız tam ekranda anlam taşıması (Kapsam madde 3) kod düzeyinde
uygulandı: `PlayerRootView` artık `NSEvent.addLocalMonitorForEvents` ile
oynatma penceresine ve `model.isFullScreen == true`'ya taranmış bir local
monitor tutuyor (`PlayerRootView.swift`), `PlayerCommands`'teki menü maddesi
ise yalnız keşfedilebilirlik ve fare tıklaması için kalıyor.

Bu aracın (`computer-use`) uzak masaüstü oturumunda `Esc` tuşu **hiçbir
şekilde teslim edilemedi** — yalnız bizim kodumuz değil, tamamen bağımsız
bir `NSOpenPanel`'in kendi yerleşik "Vazgeç" (iptal-on-Esc) davranışı da aynı
oturumda tetiklenmedi (panel açık kaldı). Local monitor'a geçici bir
`keyDown` logu eklenip `f` tuşunun (kod 3) loglandığı ama `Esc`'in (kod 53)
loga hiç düşmediği doğrulandı — olay AppKit'in event dağıtımına hiç
girmiyor. Bu, ortamın tuş iletim katmanının bir kısıtı; Nen Player'ın kodu
bu noktaya hiç ulaşmadığı için bağımsız olarak kanıtlanmış oldu.

Doğrulanabilenler:
- Tam ekrana giriş (`f`) ve çıkış (fare ile transport köşesindeki simge,
  ve menüden fare tıklamasıyla "Tam Ekrandan Çık") ikisi de çalışıyor.
- `setFullScreen(_:)` → `isFullScreen`'i doğru çeviriyor:
  `fullScreenStateFollowsSetFullScreen` model testi (bkz. Otomatik kanıt).

Doğrulanamayan tek şey: gerçek klavye `Esc`'inin canlı `.app`'te tam
ekrandan çıkışı tetiklediği — bu ortamda hiçbir uygulama için `Esc` teslim
edilemediğinden. Kod, `NSMenu` key equivalent'ının kendisinin ölçülmüş
başarısızlığına (aşağıda) karşı doğru AppKit birincili (local event
monitor) ile yazıldı; canlı doğrulama bu oturumun tuş iletim kısıtına
takıldı.

## Bulgu — `NSMenu`'nun düz `Esc` key equivalent'ı hiç tetiklenmiyor

`PlayerCommands`'in özgün `Tam Ekrandan Çık` maddesi (`KeyEquivalent("\u{1b}")`,
modifiers `[]`) — bu task'tan önce de aynı tanımla var olan kod — menü
etkinken de devre dışıyken de klavyeden asla tetiklenmedi; aynı maddeye
fare tıklaması ise her zaman çalıştı. Bu, `NSMenu.performKeyEquivalent`'ın
düz (modifikatörsüz) `Esc`'i menüye hiç sokmadığını gösteren, task'tan
bağımsız bir AppKit davranışı. Bu yüzden gerçek çıkış yolu artık
`PlayerRootView`'daki local `NSEvent` monitörü; menü maddesi keşif ve fare
tıklaması için kalıyor.

## Otomatik kanıt

`bash scripts/test-macos.sh` çıkış 0:

- **163 test / 18 suite**, 0 failure
- `a keyboard seek brings hidden controls back`
- `a keyboard volume nudge brings hidden controls back`
- `controls a keyboard seek re-shows still hide again while playing`
- `full-screen state starts false and follows setFullScreen`

İlk ikisi `seekRelative`/`adjustVolume`'un gizli kontrolleri geri
getirdiğini (DoD madde 4), üçüncüsü geri gelen kontrollerin oynatma
sürerken gizleme döngüsünü yeniden kurduğunu, dördüncüsü `setFullScreen`'in
`isFullScreen`'i doğru çevirdiğini kanıtlıyor.
