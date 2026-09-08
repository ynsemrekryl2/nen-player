---
id: NEN-087
title: Install reversible Stremio MPV bridge
milestone: M4
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-086]
blocks: [NEN-088]
adr: [44]
---

# NEN-087 — Install reversible Stremio MPV bridge

## Sonuç

Kullanıcı onayıyla kurulan köprü, Stremio'nun sabit MPV launcher yolundan Nen
Player'a gider; yabancı mevcut executable sessizce ezilmez ve kurulum geri
alınabilir.

## Kapsam

- `/usr/local/bin/mpv` için açık onaylı, idempotent kurulum ve kaldırma akışı
- Yabancı dosyada varsayılan ret; açık değiştirme seçeneğinde atomik yedek
- Nen tarafından yönetilen wrapper için değişmez işaret ve bütünlük kontrolü
- Stremio argv şeklinin Nen handoff girişine güvenli aktarımı
- Tanınmayan doğrudan `mpv` çağrılarının önceki/gerçek MPV'ye devri
- Soğuk ve zaten açık Nen Player süreçlerinin aynı handoff yolunu kullanması

## YAPILMAYACAK

- Stremio uygulama paketini veya imzasını değiştirmek
- MPV/VLC bundle kimliğini taklit etmek
- Locator, token, query veya özel yolu loglamak
- Stremio'dan geri pozisyon/sonuç protokolü almak

## Kanıt (DoD)

- [ ] Temiz makinede kurulum, mevcut dosyada güvenli ret, açık değiştirme ve
      kaldırma sonrası birebir geri yükleme testleri
- [ ] Stremio biçimli argv'nin değişmeden handoff'a ulaştığı test
- [ ] Tanınmayan çağrının gerçek MPV'ye devredildiği test
- [ ] Negatif: wrapper stdout/stderr/log yüzeyine locator veya argv sızdırmıyor
- [ ] Soğuk ve sıcak uygulama açılışı platform testiyle doğrulanıyor

## Kanıt kaydı

`evidence/M4/NEN-087-checklist.md` içindeki deterministik kök testi, gerçek
kurulu köprü soğuk/sıcak handoff kontrolü ve tam regresyon kapıları tamamlandı.

- `bash scripts/tests/stremio-mpv-bridge.test.sh`: tüm senaryolar geçti.
- Kurulu köprü: `status=installed`, marker v2 ve manifest checksum eşleşti;
  hedef/yedek metadata'sı `0755 root:wheel`.
- Gerçek app koşusu: `cold_running=yes`, `warm_command_rc=0`,
  `warm_process_after=yes`; koşu sonrası NenPlayer süreci yok.
- `bash scripts/test.sh`, `bash scripts/test-macos.sh` (239/0), macOS build,
  Rust workspace test/fmt/clippy/deny, `check-docs` ve `git diff --check`
  yeşil.
