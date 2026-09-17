# NEN-137 kanıt kaydı — Medya kimliğini trafik ışıklarıyla hizalama

Tarih: **2026-09-17** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0 · Debug, ad-hoc imzalı `NenPlayer.app`

## Otomatik kanıt

| Kanıt | Sonuç |
|---|---|
| `PlayerChromeLayout` üst satır sözleşmesi | PASS — üst padding `0`, soldan boşluk `92 pt` |
| `bash scripts/test-macos.sh` | PASS — 260 shell/player, 57 playback/contract ve 4 Keychain testi |
| `bash scripts/build-macos-app.sh` | PASS — uygulama derlendi ve ad-hoc imzalandı |

## Elle kabul

1. Gerçek `NenPlayer.app`, telif-temiz `contract-clip.mkv` fixture'ı ile
   açıldı.
2. Başlık ve trafik ışıkları aynı üst satırda gözlendi; başlık artık safe-area
   altında ek bir 18 pt boşlukla aşağı itilmedi.
3. Basename görünümüyle aynı `PlayerRootView` yerleşimi doğrulanmış medya
   kimliği görünümüne de uygulanıyor.

Ekran görüntüsü gerçek uygulama koşusundan alındı. Koşu sırasında macOS
Keychain izin istemi öne çıktı; credential içeriği kullanılmadı ve ek güvenlik
izni verilmedi. Başlık/trafik ışığı hizası player penceresinde görünür kaldı.

## Doküman kapıları

```text
bash scripts/check-docs.sh          PASS — 10/10 denetim
bash scripts/task-index.sh --check  PASS
git diff --check                    PASS
```

Üç komutun da çıkışı 0 oldu.
