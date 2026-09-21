---
id: NEN-137
title: Align media identity with traffic lights
milestone: M6
size: S
state: done
closed: 2026-09-20
depends_on: [NEN-136]
blocks: [NEN-126]
adr: []
---

# NEN-137 — Align media identity with traffic lights

## Sonuç

Oynatıcıdaki medya kimliğinin yazı merkezi, trafik ışıklarının merkeziyle aynı
dikey çizgiye gelir ve başlık yatayda trafik ışıklarının yanında kalır.

## Kapsam

- `PlayerRootView` içindeki medya kimliği başlığının trafik ışıklarıyla aynı
  yatay satırda ve optik olarak aynı dikey merkezde görünmesini sağlamak;
  başlığı doğrudan trafik ışığı butonlarının AppKit superview'ında çizmek ve
  gerçek buton frame'inin dikey merkezine bağlamak.
- Başlık basename fallback'i ve doğrulanmış medya kimliği için aynı hizalama
  sözleşmesini korumak.
- macOS UI testinde yerleşim sözleşmesini ve gerçek uygulama ekran görüntüsünü
  doğrulamak.

## YAPILMAYACAK

- Medya kimliği çözümleme, provider araması veya başlık metni değişikliği.
- Transport kontrolleri, pencere boyutu/aspect lock veya güvenli alan
  davranışını yeniden tasarlamak.
- Tam dosya yolu, URL, hash veya özel medya metadata'sını test/log kanıtına
  çıkarmak.

## Kanıt (DoD)

- [x] Başlık yerleşimi için deterministic macOS UI/layout testi geçiyor.
- [x] Gerçek `.app` üzerinde basename ve doğrulanmış başlık için kısa manuel
      checklist, yazı merkezi ile trafik ışıkları merkezinin aynı dikey çizgide
      olduğunu gösteriyor. Kullanıcı 2026-09-20'de gerçek `.app` üzerinde
      doğruladı ve bu hâliyle kapanışı onayladı; ekran görüntüsü alınamadı
      (Keychain izin istemi / ortak workspace), UI kanıtı checklist olarak
      kabul edildi (`evidence/M6/NEN-137-checklist.md`).
- [x] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`,
      `bash scripts/task-index.sh --check` ve `git diff --check` çıkış 0.

## Kanıt kaydı

- `swift test --package-path platforms/macos --filter TransportControlsLayoutTests`
  (2026-09-20 kapanış koşusu): exit 0; 3/3 test geçti, `media identity shares
  the traffic-light titlebar row` dahil.
- `bash scripts/test-macos.sh` (2026-09-20 kapanış koşusu): exit 0; 265
  shell/player (26 suite), 57 playback/contract ve 4 Keychain testi geçti.
- `bash scripts/build-macos-app.sh` (2026-09-17, kullanıcı doğrulamasından
  önceki koşu): exit 0; güncel debug ad-hoc `NenPlayer.app` üretildi.
- `bash scripts/check-docs.sh` (2026-09-20): exit 0; 10/10 denetim geçti.
- `bash scripts/task-index.sh --check` (2026-09-20): exit 0.
- `git diff --check` (2026-09-20): exit 0.
- Kullanıcının son ekran görüntülerinde SwiftUI safe-area/ofset yaklaşımının
  piksel olarak değişiklik üretmediği doğrulandı; label artık gerçek trafik
  ışığı buton frame'inin `midY` değerine bağlanıyor.
- Kullanıcı 2026-09-20'de gerçek `.app` üzerinde basename ve doğrulanmış medya
  kimliği başlıklarının trafik ışığı hizasını manuel doğruladı; sonucun
  beklendiği gibi olduğunu bildirdi ve ekran görüntüsü olmadan bu hâliyle
  kapanışı onayladı. Optik offset kullanıcının doğruladığı `-1.0 pt` değeridir;
  kodda kapanış için ek değişiklik yapılmadı.
