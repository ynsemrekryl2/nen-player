---
id: NEN-142
title: Remove focus ring from transport controls
milestone: M6
size: S
state: done
closed: 2026-09-21
depends_on: []
blocks: []
adr: []
---

# NEN-142 — Transport kontrollerinde odak halkasını kaldır

## Sonuç

Oynatıcı transport barındaki seek ve ses slider'larına tıklandığında,
sürüklendiğinde veya klavyeyle odaklandığında kenarlarında mavi odak halkası
oluşmaz; kontroller aynı pointer, klavye ve VoiceOver davranışıyla çalışmaya
devam eder.

## Bağlam

Kullanıcı 2026-09-21'de gerçek `.app` üzerinde seek slider'ına tıklandığında
çevresinde mavi çerçeve oluştuğunu bildirdi. Kaynak, `GlassSlider`'ın
odaklandığında kendi çizdiği halka: `GlassControls.swift` içinde
`RoundedRectangle(cornerRadius: 8).stroke(isFocused ? Color.accentColor.opacity(0.70) : .clear,
lineWidth: 2)`. Slider `.focusable(isEnabled)` olduğu için tıklama odağı
slider'a taşıyor ve halka anında görünüyor. `GlassSlider` yalnız transport
barında iki yerde kullanılıyor (`TransportControls.swift` — seek ve ses); bu
yüzden halka yalnız bu iki kontrolün etkileşiminde görülüyor. Ayrıca
`.focusable` özel görünümlerde sistemin varsayılan odak efekti de çizilebilir.

## Kapsam

- `GlassSlider` içindeki özel odak halkası overlay'inin kaldırılması;
  kullanılmayan `@FocusState isFocused` ve `.focused($isFocused)` bağının
  silinmesi.
- Sistemin varsayılan odak efektinin `.focusEffectDisabled()` ile kapatılması
  (macOS 14+ hedefiyle uyumlu).
- `.focusable(isEnabled)`, `.onMoveCommand`, `accessibilityElement`,
  `accessibilityAdjustableAction` ve drag davranışının **korunması** — klavye
  oku ve VoiceOver ayarı bozulmaz.
- Gerekirse (manuel kontrolde görülürse) transport barı kökünde düğmeler için
  de sistem odak efektinin kapatılması.
- Gerçek `.app` üzerinde pointer, klavye ve VoiceOver kontrolü.

## YAPILMAYACAK

- Slider'ların keyboard/VoiceOver davranışını veya transport yerleşimini
  yeniden tasarlamak.
- Ayarlar penceresindeki metin alanları ve düğmeler gibi oynatıcı dışı
  yüzeylerin odak gösterimini değiştirmek.
- Transport düğmelerine yeni etkileşim, kısayol veya görsel stil eklemek.
- Ekran görüntüsü/kanıtta tam dosya yolu, URL, hash veya özel medya
  metadata'sı göstermek (K23).

## Kanıt (DoD)

- [ ] Gerçek `.app`: seek ve ses slider'ına tıklama/sürükleme sırasında ve
      sonrasında mavi odak halkası yok; transport düğmelerinde de yok.
- [ ] Aynı slider'lar klavye okları ve VoiceOver adjustable action ile
      çalışmaya devam ediyor; `onMoveCommand` regresyonu yok.
- [ ] `bash scripts/test-macos.sh` çıkış 0.
- [ ] `bash scripts/build-macos-app.sh` çıkış 0.
- [ ] `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` çıkış 0.

## Kanıt kaydı

- **Kök neden:** `GlassSlider` `.focusable(isEnabled)` olduğundan tıklama
  odağı slider'a taşıyor; `GlassControls.swift` içindeki
  `RoundedRectangle(cornerRadius: 8).stroke(isFocused ? Color.accentColor.opacity(0.70) : .clear, lineWidth: 2)`
  overlay'i mavi halkayı çiziyordu. `GlassSlider` yalnız transport barındaki
  seek ve ses slider'larında kullanılıyor.
- **Düzeltme:** Özel overlay, kullanılmayan `@FocusState isFocused` ve
  `.focused($isFocused)` kaldırıldı; sistem varsayılan odak efekti için
  `.focusEffectDisabled()` eklendi (macOS 14+ hedefiyle uyumlu).
  `.focusable(isEnabled)`, `.onMoveCommand` ve
  `accessibilityAdjustableAction` korundu.
- **Tam paket:** `bash scripts/test-macos.sh` → çıkış 0; 267 shell/player
  (27 suite), 57 playback/contract ve 4 Keychain testi geçti.
- **Uygulama derlemesi:** `bash scripts/build-macos-app.sh` → çıkış 0; güncel
  debug ad-hoc `NenPlayer.app` üretildi.
- **Gerçek `.app` manuel kabulü:** kullanıcı 2026-09-21'de seek ve ses
  slider'larında ve transport düğmelerinde mavi odak halkasının kalmadığını;
  ok tuşlarıyla seek/ses ayarının çalışmaya devam ettiğini doğruladı.
  Ayrıntı: `evidence/M6/NEN-142-checklist.md`.
- **Kapılar:** `bash scripts/check-docs.sh` 10/10, `bash
  scripts/task-index.sh --check` ve `git diff --check` çıkış 0.