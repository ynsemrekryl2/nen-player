---
id: NEN-137
title: Align media identity with traffic lights
milestone: M6
size: S
state: done
closed: 2026-09-17
depends_on: [NEN-136]
blocks: [NEN-126]
adr: []
---

# NEN-137 — Align media identity with traffic lights

## Sonuç

Oynatıcıdaki medya kimliği, trafik ışıklarıyla aynı üst yatay hizaya gelir ve
başlığın gereksiz yere aşağıda görünmesi ortadan kalkar.

## Kapsam

- `PlayerRootView` içindeki medya kimliği başlığının üst krom yerleşimini trafik
  ışıklarının bulunduğu satıra göre düzeltmek.
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
      checklist ve ekran görüntüsü, başlığın trafik ışıklarıyla aynı satırda
      olduğunu gösteriyor.
- [x] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`,
      `bash scripts/task-index.sh --check` ve `git diff --check` çıkış 0.

## Kanıt kaydı

`evidence/M6/NEN-137-checklist.md` içindeki gerçek `.app` gözlemi ve otomatik
test kayıtlarıyla desteklenmiştir.

- `bash scripts/test-macos.sh`: exit 0; 260 shell/player, 57 playback/contract
  ve 4 Keychain testi geçti.
- `bash scripts/build-macos-app.sh`: exit 0; debug ad-hoc `NenPlayer.app`
  üretildi.
- `bash scripts/check-docs.sh`: exit 0; 10/10 denetim geçti.
- `bash scripts/task-index.sh --check`: exit 0.
- `git diff --check`: exit 0.
