# NEN-142 kanıt kaydı — Transport kontrollerinde odak halkası kaldırıldı

Tarih: **2026-09-21** · Apple Silicon · macOS 27.0 (26A428) · Swift 6.4 ·
libmpv 2.5.0 · Debug, ad-hoc imzalı `NenPlayer.app`

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## Otomatik kanıt

| Kanıt | Sonuç |
|---|---|
| `bash scripts/test-macos.sh` | PASS — çıkış 0; 267 shell/player (27 suite), 57 playback/contract ve 4 Keychain testi |
| `bash scripts/build-macos-app.sh` | PASS — çıkış 0; debug ad-hoc `NenPlayer.app` üretildi |

Kök neden ölçüldü: `GlassSlider` `.focusable(isEnabled)` olduğu için slider'a
tıklama odağı slider'a taşıyordu; `GlassControls.swift` içindeki
`isFocused ? Color.accentColor.opacity(0.70) : .clear` overlay'i halkayı
çiziyordu. Düzeltme: özel overlay, kullanılmayan `@FocusState isFocused` ve
`.focused($isFocused)` kaldırıldı; `.focusEffectDisabled()` eklendi. Klavye
davranışı `.focusable` + `.onMoveCommand` ile, VoiceOver
`accessibilityAdjustableAction` ile korundu.

## Elle kabul

1. Gerçek `NenPlayer.app`, `contract-clip.mkv` ile açıldı.
2. Seek slider'ı tıklandı ve sürüklendi: mavi odak halkası **yok**.
3. Ses slider'ı tıklandı ve sürüklendi: mavi odak halkası **yok**.
4. Transport düğmeleri (oynat/duraklat, ±5 sn, süre, CC, hız, tam ekran)
   tıklandı: mavi odak halkası **yok**.
5. Oynatıcı penceresi öndeyken ok tuşlarıyla seek (sol/sağ) ve ses
   (yukarı/aşağı) çalışmaya devam ediyor.

Sonuç: **2/2 soru geçti**; kullanıcı 2026-09-21'de doğruladı.

## Doküman kapıları

```text
bash scripts/check-docs.sh          PASS — 10/10 denetim
bash scripts/task-index.sh --check  PASS
git diff --check                    PASS
```
