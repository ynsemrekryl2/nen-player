# NEN-137 kanıt kaydı — Medya kimliğini trafik ışıklarıyla hizalama

Tarih: **2026-09-17** · Apple Silicon · macOS 27.0 · Xcode 26.6 · Swift
6.3.3 · libmpv 2.5.0 · Debug, ad-hoc imzalı `NenPlayer.app`

## Otomatik kanıt

| Kanıt | Sonuç |
|---|---|
| `PlayerChromeLayout` optik merkez sözleşmesi | PASS — label frame’i gerçek trafik ışığı butonunun `midY` değerine bağlanıyor; glyph optiği için `-1.0 pt` düzeltme ve yatay başlangıç `92 pt` |
| `swift test --package-path platforms/macos --filter TransportControlsLayoutTests` | PASS — 3/3 |
| `bash scripts/test-macos.sh` | PASS — 265 shell/player (26 suite), 57 playback/contract ve 4 Keychain testi (2026-09-20 kapanış koşusu) |
| `bash scripts/build-macos-app.sh` | PASS — güncel uygulama derlendi ve ad-hoc imzalandı |

## Elle kabul

1. Gerçek `NenPlayer.app`, telif-temiz `contract-clip.mkv` fixture'ı ile
   açıldı.
2. Başlık ve trafik ışıkları aynı üst satırda gözlendi; başlık artık safe-area
   altında ek bir 18 pt boşlukla aşağı itilmedi.
3. Basename görünümüyle aynı `PlayerRootView` yerleşimi doğrulanmış medya
   kimliği görünümüne de uygulanıyor.

Kullanıcının son ekran görüntülerinde önceki SwiftUI safe-area/ofset yaklaşımının
piksel olarak hiç değişmediği doğrulandı. Başlık artık SwiftUI içerik alanında
değil, trafik ışığı kapatma butonunun gerçek AppKit superview'ına eklenen label
olarak çiziliyor; dikey hizası aynı buton frame'inin `midY` değerinden hesaplanıp
`NSTextField` glyph merkezinin optik farkı için `-1.0 pt` düzeltiliyor.

## Kapanış onayı (2026-09-20)

Kullanıcı gerçek `NenPlayer.app` üzerinde basename başlığı ve doğrulanmış medya
kimliği başlığı için trafik ışığı hizasını manuel olarak inceledi; sonucun
beklendiği gibi olduğunu bildirdi ve bu hâliyle kapanışı onayladı. Temiz
post-change ekran görüntüsü, macOS Keychain izin istemi ve ortak workspace'te
temiz bir canlı koşu yeniden alınamadığı için eklenmedi; UI kanıtı bu manuel
checklist olarak kaydedildi. Kodda ek değişiklik yapılmadı; optik offset
kullanıcının doğruladığı `-1.0 pt` değeridir.

## Doküman kapıları

```text
bash scripts/check-docs.sh          PASS — 10/10 denetim
bash scripts/task-index.sh --check  PASS
git diff --check                    PASS
```

Üç komutun da çıkışı 0 oldu.
