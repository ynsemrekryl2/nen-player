---
id: NEN-068
title: Media-sized, aspect-locked player window
milestone: M3
size: L
state: done
closed: 2026-09-05
depends_on: [NEN-067]
blocks: []
adr: [26, 31, 38]
---

# NEN-068 — Medya boyutunda açılan, orana kilitli pencere

## Sonuç

Pencere, açılan medyanın display boyutunda açılır ve resize boyunca o en-boy
oranını korur. Tam ekran dışında pencerede üstte veya altta hiçbir zaman siyah
bar oluşmaz.

## Kapsam

### Core — playback portu (ADR-0038)

- `VideoGeometry { width, height }` tipi; ikisi de sıfırdan büyük.
- `PlaybackEngine::video_geometry() -> Result<Option<VideoGeometry>, PlaybackError>`.
- Payload'sız coalescing `PlaybackEvent::VideoGeometryChanged`.
- Fake engine ve contract kiti yeni davranışı zorlar; yeni `Capability` yok.

### App ve FFI

- `ShellEngine` karşılığı ve `ShellEngineBridge` geçişi —
  `set_subtitle_bottom_inset` hangi deseni izliyorsa aynısı.
- `FfiVideoGeometry`, `ForeignPlaybackEngine.videoGeometry()`,
  `FfiSessionEvent::VideoGeometryChanged`, `FfiPlaybackSession.videoGeometry()`.

### macOS adapter

- `video-out-params/dw`/`dh` okunur; yoksa `video-params/dw`/`dh` fallback.
- `MPV_EVENT_VIDEO_RECONFIG` olayı porta `VideoGeometryChanged` olarak çıkar.
- `NullPlaybackEngine` yeni metodu karşılar.

### Kabuk

- `PlaybackSessionClient` protokolüne sorgu; test fake'ine karşılığı.
- `PlayerModel.videoGeometry`: olayda ve `EventsLost` resync'inde yeniden
  okunur, medya kapanınca `nil` olur.

### Sunum (ADR-0038 Karar 4)

- Yeni `WindowGeometry` saf fonksiyonları:
  - `minimumContentSize(for:)` — krom tabanı **600×390 pt**'den orana göre
    türetir: `minW = max(600, 390 × oran)`, `minH = minW / oran`.
    16:9 → 693×390 · 2.39:1 → 932×390 · 4:3 → 600×450 · 9:16 → 600×1067.
  - `contentSize(for:visibleFrame:)` — nominal 1 pt = 1 px; ekranın
    `visibleFrame`ine sığmıyorsa oran korunarak ölçeklenir, sonra minimuma
    yükseltilir.
- `WindowTitleWriter` desenini izleyen bir pencere yazıcısı
  `contentAspectRatio`, `contentMinSize` ve yeni medyada `setContentSize`
  uygular; pencere merkezi korunur, ardından ekrana clamp edilir.
- `PlayerRootView`'daki sabit `.frame(minWidth: 720, minHeight: 450)`
  gevşetilir. SwiftUI'ın kendi minimumu pencereye yayılır; türetilmiş
  `contentMinSize` ile çatışırsa AppKit ikisini birden tutamaz ve minimum
  boyutta letterbox geri gelir. Minimum tek kaynaktan, `WindowGeometry`'den
  gelmelidir.
- Tam ekrana girerken oran kilidi bırakılır, çıkarken geri konur.
- Video yokken (audio-only, fatal, boş durum) kilit yok; serbest resize ve
  mevcut `1080×680` varsayılan boyut korunur.

## Krom tabanının kaynağı

600×390 pt ölçülmüş bir sayıdır, keyfi değil:

- **Genişlik ≈590 pt** — NEN-067 transport sırasının içsel genişliği:
  2×22 pt iç boşluk + 11×6 pt aralık + 34+30+30 düğme, ~30 geçen süre,
  76 seek minimumu, 43 kalan süre, 22 hoparlör, 72 ses minimumu, 4,5 ayraç,
  69 CC, 40 hız, 30 tam ekran. Bir saati aşan `0:00:00` biçiminde ≈606 pt.
- **Yükseklik ≈390 pt** — 326 pt altyazı paneli + 57 pt bar.

Bugünkü 720×450 bu tabanın yuvarlanmış hâlidir ve 1,60'tan geniş videolarda
oranla çelişir.

## YAPILMAYACAK

- Codec, HDR veya kalite rozeti (ADR-0036 `rejected`, `NEN-063` `canceled`).
- Kullanıcıya oran seçtirme (zoom / crop / stretch menüsü).
- Pencere boyutunu diske kalıcı yazma.
- NEN-067'nin krom yerleşimini değiştirmek.

## Kanıt (DoD)

- [x] Contract kit `None` ve pozitif geometry senaryolarını,
      `VideoGeometryChanged` sıralamasını ve `EventsLost` sonrası yeniden
      okumayı zorluyor; fake engine yeşil.
- [x] `guard_playback_debug` ve redaction kapıları yeni tiple yeşil.
- [x] `WindowGeometry` saf fonksiyon testleri: 16:9, 4:3, 2.39:1, dikey 9:16,
      anamorphic 720×576 → 1024×576, ekrandan büyük 3840×2160 ve minimum
      türetmesi.
- [x] Model testi: geometry olayında `videoGeometry` güncelleniyor, medya
      kapanınca sıfırlanıyor, audio-only'de `nil`.
- [x] Elle doğrulama: 16:9, 4:3 ve 2.39:1 dosyalarda açılış boyutu; canlı
      resize boyunca siyah bar yok; tam ekran giriş/çıkış; audio-only serbest
      resize → `evidence/M3/NEN-068-checklist.md` ve ekran görüntüleri.
- [x] `cargo test`, `bash scripts/test.sh`, `bash scripts/test-macos.sh`,
      uygulama build'i, strict codesign ve doküman kapıları geçti.

## Kanıt kaydı

2026-09-05 kapanışında Rust workspace **560/560** geçti (1 ignored benchmark);
fmt, clippy, cargo-deny, shell testleri ve doküman kapıları yeşil. macOS paketi
**150 test / 15 suite / 0 failure** verdi; debug `.app` build'i ve strict
codesign geçti.

Gerçek ad-hoc imzalı uygulamada 16:9 **693×390**, 4:3 **600×450**, 2.39:1
**931×390** ve anamorphic 720×576 kaynak **1024×576** açıldı. Tam ekran `F`,
düğme ve `Esc` yolları; oynarken/duraklatılmışken play/pause ve seek; çıkış
sonrası oran kilidi ve canlı kenar sürüklemesi doğrulandı. Audio-only ve boş
durumda oran kilidi kurulmadı, serbest resize ölçüldü.

Elle kabul iki gerçek kusur buldu ve kapattı: video yüzeyi safe area nedeniyle
kendi siyah çerçevesini çiziyordu; ayrıca sıfır oran ataması native tam ekran
çıkışını yarıda bırakıyordu. Ardışık medya kabulü de yükleme sırasında eski VO
boyutunun sızdığını gösterdi. Üçü regresyon ve negatif kontrollerle zorlandı.

Tam kayıt ve ekran kanıtları:
`evidence/M3/NEN-068-checklist.md` ·
`evidence/M3/NEN-068-measurement.md` ·
`evidence/M3/NEN-068-fullscreen.md`.
