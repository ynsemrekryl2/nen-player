# NEN-088 — Stremio → Nen Player kabul kanıtı

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0 · Stremio 5.1.26

## Önkoşullar

1. `bash scripts/doctor.sh M3` — tüm blocker'lar hazır, çıkış 0.
2. `bash scripts/stremio-mpv-bridge.sh status` — `installed`.
3. `bash scripts/build-macos-app.sh` — taze debug `.app`, çıkış 0.

## Kabul adımları

### Soğuk handoff

1. Nen Player kapalıyken Stremio'da gerçek “MPV içinde oynat” eylemi seçildi.
2. Nen Player açıldı ve oynatma sürdü.
3. Transport şeridi `00:02` gösterdi; güvenli ekran kanıtı:
   `NEN-088-cold-transport.png`.

### Sıcak handoff

1. Nen Player açık ve aynı medya oynarken Stremio'da aynı eylem tekrarlandı.
2. Mevcut Nen Player süreci korundu (`warm_nenplayer_process_count=1`).
3. Oynatma sürdü; transport şeridi `00:03` gösterdi; güvenli ekran kanıtı:
   `NEN-088-warm-transport.png`.

## Pozisyon ve metadata

Stremio 5.1.26'nın NEN-086'da ölçülen `--start=0` davranışı bu koşuda baştan
oynatma olarak gözlendi. Nonzero başlangıç değeri gelmedi; devam konumu tahmin
edilmedi. Handoff metadata alanı olmadan akış kesilmedi ve playback devam etti.

## Negatif log taraması

Ham unified log ve stdout/stderr geçici dosyalarda tutuldu, yalnız sayaçlar
raporlandı ve dosyalar koşu sonunda silindi:

| Kontrol | Eşleşme |
|---|---:|
| URL scheme (`http`, `https`, `file`, `nenplayer`) | 0 |
| Token/query anahtarları | 0 |
| Özel dosya yolu desenleri | 0 |
| Handoff argv (`--start*`, `--no-terminal`) | 0 |
| stdout/stderr yakalama | 0 |

Kanıt ekranları yalnız transport şeridini içerir; gerçek medya adı, URL,
token, query veya özel metadata içermez.
