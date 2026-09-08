# NEN-086 — Stremio MPV launcher düzeltme kanıtı

Tarih: **2026-09-08** · macOS 27.0 · Stremio shell 5.1.26

## Ölçülen launcher sözleşmesi

Kurulu Stremio shell'in `server.js` kaydındaki MPV tanımı aşağıdaki kapalı
kümeyi kullanıyor:

- macOS yolları, sırayla: `/usr/local/bin/mpv`, `/opt/local/bin/mpv`,
  `/sw/bin/mpv`
- argümanlar: `--start=<saniye>`, `--no-terminal`, ardından tek locator
- seçim: var olan ilk yol
- çalıştırma: seçilen executable'a argv aktarımı

Bu şekil, Stremio shell'in yayımlanmış launcher kaydıyla da örtüşür:
[stremio-shell #351](https://github.com/Stremio/stremio-shell/issues/351).
Rapor hiçbir gerçek medya URL'si, query, token, port veya özel kullanıcı yolu
içermez.

## Önceki ölçümün düzeltmesi

`NEN-078` ve `NEN-084` sırasında başlayan uygulama, Stremio tarafından
özel bundle olarak keşfedilmemiştir. İlk mevcut launcher yolu olan
`/usr/local/bin/mpv`, önceden kurulmuş bir shell wrapper'ıdır; wrapper hedef
uygulamaya `exec "$EXECUTABLE" "$@"` ile geçer ve kendi başına locator/argv
loglamaz. Böylece gözlenen uygulama süreci ile Stremio launcher yolu arasındaki
nedensel bağ artık açıkça kayıtlıdır.

## Güvenli fixture koşusu

Wrapper'ın mevcut ortam değişkeniyle geçici bir, telif-temiz executable fixture'ı
kullanıldı. Wrapper çıktısı:

```text
--start=0 --no-terminal stremio-fixture
```

Sonuç: çıkış kodu `0`; fixture'a ulaşan argv sırası ve değerleri değişmeden
korundu. Wrapper'ın kendi stderr çıktısı boştu; görülen tek stdout fixture
executable'ının bilinçli argv yankısıydı. Geçici fixture ve çıktı saklanmadı.

## Karar ve kapsam etkisi

`ADR-0044` kabul edildi: Stremio paketi veya bundle kimliği değişmeden,
kullanıcı onaylı ve geri alınabilir `/usr/local/bin/mpv` köprüsü kurulacak.
Yabancı mevcut dosya sessizce ezilmeyecek; gerçek kurulum `NEN-087`'ye,
soğuk/sıcak gerçek Stremio kabulü `NEN-088`'e bırakıldı.

Bu nedenle M4 henüz kapanmadı. `NEN-084`, Nen Player'ın ölçülmüş MPV argv
sözleşmesini kabul eden tarihsel kanıttır; gerçek Stremio → Nen Player eylemi
kanıtlanana kadar M4'ün ilk çıkış kriteri tamamlanmış sayılmaz.

## Doğrulama

- Kurulu Stremio `server.js` launcher tablosu: `83162`–`83171` satırlarında
  sabit yol ve argv şekli doğrulandı.
- Geçici fixture: `wrapper_status=0`,
  `forwarded=--start=0 --no-terminal stremio-fixture`,
  `stderr_bytes=0`.
- `bash scripts/check-docs.sh`: **SONUÇ: tüm denetimler geçti.**
- `git diff --check`: **çıkış 0**.
