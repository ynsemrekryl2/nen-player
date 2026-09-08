---
id: NEN-086
title: Correct Stremio MPV launcher measurement
milestone: M4
size: S
state: done
closed: 2026-09-08
depends_on: []
blocks: [NEN-087]
adr: [44]
---

# NEN-086 — Correct Stremio MPV launcher measurement

## Sonuç

Stremio'nun gerçek MPV launcher yolu ve Nen Player'a bağlanacak geri alınabilir
köprü kararı ölçülmüş, ADR-0044'e bağlanmış ve M4'ün gerçek kabul koşulu açık
biçimde yeniden kurulmuştur.

## Kapsam

- Stremio 5.1.26'nın macOS launcher yollarını ve argv şeklini güvenli kanıtla
  kaydetmek
- Önceki ölçümde başlayan uygulamanın, sabit MPV yolu üzerindeki mevcut wrapper
  üzerinden çalıştığını düzeltme notuyla belgelemek
- Geri alınabilir `/usr/local/bin/mpv` köprüsü için ADR-0044'ü kabul etmek
- M4 kriterini gerçek Stremio → Nen Player akışını bekleyecek şekilde düzeltmek
- `NEN-087` kurulum ve `NEN-088` gerçek kabul görevlerini açmak

## YAPILMAYACAK

- Köprüyü kurmak veya mevcut sistem executable'ını değiştirmek — `NEN-087`
- Gerçek Stremio kabul koşusunu yapmak — `NEN-088`
- Stremio paketini yamamak veya MPV/VLC bundle kimliğini taklit etmek
- Nen Player çalışma zamanı/API kodunu değiştirmek

## Kanıt (DoD)

- [x] `evidence/M4/NEN-086-stremio-mpv-bridge.md`, sabit yol sırası, argv
      şekli, mevcut wrapper korelasyonu ve güvenli fixture koşusunu içeriyor
- [x] ADR-0044 `accepted` ve bu task'a bağlı
- [x] NEN-078 ve NEN-084 tarihsel kayıtları silinmeden düzeltme notu taşıyor
- [x] M4 kriteri, roadmap ve STATUS gerçek akış tamamlanmadan kapanmayacak
      şekilde tutarlı
- [x] NEN-087 ve NEN-088 backlog'da doğru bağımlılıklarla yer alıyor
- [x] `bash scripts/check-docs.sh` ve `git diff --check` yeşil

## Kanıt kaydı

**2026-09-08.** Kurulu Stremio shell'in `server.js` launcher tablosu
`/usr/local/bin/mpv`, `/opt/local/bin/mpv`, `/sw/bin/mpv` sırasını ve
`--start=<saniye> --no-terminal <locator>` argv şeklini doğruladı. Mevcut
wrapper geçici fixture'a argv'yi aynen aktardı:
`wrapper_status=0`, `forwarded=--start=0 --no-terminal stremio-fixture`,
`stderr_bytes=0`. ADR-0044 kabul edildi; M4 kriteri ve tarihsel NEN-078/
NEN-084 kayıtları düzeltildi; NEN-087/NEN-088 bağımlı backlog görevleri açıldı.

- `bash scripts/check-docs.sh` — **SONUÇ: tüm denetimler geçti**
- `git diff --check` — **çıkış 0**
