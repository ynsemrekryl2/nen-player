# NEN-074 — Akıcı canlı resize ölçümü

Tarih: **2026-09-06**
Makine: MacBook Air (Apple M5, 10 core, 16 GB)
Yazılım: macOS 27.0 (26A5421a) · Xcode 26.6 · Swift 6.3.3 ·
libmpv 2.5.0 / mpv 0.41.0_8
Build: Debug, ad-hoc imzalı `NenPlayer.app`

## Yöntem

Telif-temiz `fixtures/media/contract-clip.mkv` oynarken pencerenin sağ-alt
köşesi CoreGraphics fare olaylarıyla 120 adımda `+500×+281`, ardından aynı
yoldan geri sürüklendi. Her adım arası 16,667 ms; büyütme/küçültme üç
kez tekrarlandı. Önce ve sonra aynı fixture, pencere durumu, hareket ve
makine kullanıldı.

Geçici prob `MPVVideoView.draw(_:)` girişinden `flushBuffer()` sonuna kadar
ana-thread süresini bellekte topladı; yalnız resize bittiğinde özet yazdı.
Prob her iki varyantta da aynıydı ve son ürün kodundan tamamen kaldırıldı.
Sayılar baseline'dır; pass/fail eşiği değildir.

## Ham sonuçlar

| Varyant | Yön/koşu | Render | Median (ms) | p95 (ms) | Max (ms) | Resize (ms) |
|---|---:|---:|---:|---:|---:|---:|
| Önce | 1 | 33 | 4,024 | 14,295 | **175,899** | 2406,2 |
| Önce | 2 | 41 | 2,211 | 7,976 | 10,404 | 2440,9 |
| Önce | 3 | 40 | 2,005 | 12,435 | 18,106 | 2416,9 |
| Önce | 4 | 41 | 1,552 | 8,260 | 16,678 | 2420,2 |
| Önce | 5 | 40 | 4,562 | 13,591 | 18,744 | 2409,8 |
| Önce | 6 | 41 | 1,489 | 5,380 | 7,289 | 2402,7 |
| Sonra | 1 | 49 | 2,372 | 10,231 | 11,365 | 2415,9 |
| Sonra | 2 | 48 | 2,328 | 8,925 | 9,986 | 2396,6 |
| Sonra | 3 | 48 | 1,727 | 10,571 | **17,991** | 2427,9 |
| Sonra | 4 | 51 | 2,439 | 9,274 | 14,809 | 2435,7 |
| Sonra | 5 | 49 | 1,616 | 3,648 | 13,070 | 2407,0 |
| Sonra | 6 | 37 | 1,853 | 13,017 | 16,340 | 2406,8 |

Merkez süreler önce/sonra örtüşüyor; düzeltmenin iddiası onları sıfırlamak
değil, libmpv'nin hedef kare beklemesini canlı resize ana thread'inden
çıkarmaktır. Ayırt edici sonuç, önceki **175,899 ms** en kötü çağrının
sonraki altı koşuda tekrarlanmaması ve aynı yaklaşık 2,4 saniyelik harekette
render sayısının `33–41` aralığından `37–51` aralığına çıkmasıdır. Sabit
FPS veya milisaniye eşiği türetilmedi.

## Gerçek `.app` kabulü

| Senaryo | Gözlem | Sonuç |
|---|---|---|
| Oynayan videoda kesintisiz köşe sürüklemesi | Altı ölçüm hareketi de aynı olay süresinde tamamlandı; sonrasında pencere olay yolunu takip etti ve video yüzeyi görünür kaldı | ✅ |
| Bırakma sonrası | Prob içermeyen son build'de bir büyütme/küçültme daha yapıldı; kontroller gizli oynatma durumuna döndü ve sonraki video karesi görüldü | ✅ |
| Normal zamanlama | Politika testi canlı resize dışında `BLOCK_FOR_TARGET_TIME=1`, resize sırasında `0` seçildiğini sabitliyor | ✅ |
| Aspect lock | Sürükleme boyunca 16:9 yüzeyde siyah bar oluşmadı; tam ekrandan dönüşte frame `693×422` (693×390 content) oldu | ✅ |
| Tam ekran | `F` ile `1470×923` tam ekrana girdi; ikinci `F` ile oran kilitli pencereye döndü | ✅ |
| Videosuz yüzey | Video ardından `audio-only-clip.mka` açıldı; önceki kare kalmadı, yüzey siyah kaldı ve sayaç ilerledi | ✅ |

## Otomatik kapılar

- `RenderTimingPolicyTests`: **2/2** — normal `true`, canlı resize `false`.
- `bash scripts/test-macos.sh`: **181 test / 22 suite / 0 failure**.
- `bash scripts/build-macos-app.sh`: exit 0.
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`:
  exit 0.
- `bash scripts/test.sh`: shell test dosyalarının **2/2**'si geçti.
- `bash scripts/check-docs.sh`: exit 0.
- `bash scripts/task-index.sh --check`: exit 0.
