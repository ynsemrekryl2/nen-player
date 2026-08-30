---
id: NEN-068
title: Media-sized, aspect-locked player window
milestone: M3
size: L
state: backlog
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

- [ ] Contract kit `None` ve pozitif geometry senaryolarını,
      `VideoGeometryChanged` sıralamasını ve `EventsLost` sonrası yeniden
      okumayı zorluyor; fake engine yeşil.
- [ ] `guard_playback_debug` ve redaction kapıları yeni tiple yeşil.
- [ ] `WindowGeometry` saf fonksiyon testleri: 16:9, 4:3, 2.39:1, dikey 9:16,
      anamorphic 720×576 → 1024×576, ekrandan büyük 3840×2160 ve minimum
      türetmesi.
- [ ] Model testi: geometry olayında `videoGeometry` güncelleniyor, medya
      kapanınca sıfırlanıyor, audio-only'de `nil`.
- [ ] Elle doğrulama: 16:9, 4:3 ve 2.39:1 dosyalarda açılış boyutu; canlı
      resize boyunca siyah bar yok; tam ekran giriş/çıkış; audio-only serbest
      resize → `evidence/M3/NEN-068-checklist.md` ve ekran görüntüleri.
- [ ] `cargo test`, `bash scripts/test.sh`, `bash scripts/test-macos.sh`,
      uygulama build'i, strict codesign ve doküman kapıları geçti.

## Kanıt kaydı

<!-- done olurken doldurulacak -->
