# NEN-062 — transport üstü altyazı paneli, acceptance

Koşum: **2026-08-29** · macOS · ad-hoc imzalı
`platforms/macos/.build/NenPlayer.app` · libmpv 2.5.0 (Homebrew, dinamik).

Kanıt medyası yalnız depodaki sentetik fixture'lardır:

- parlak yüzey ve menü semantiği: `fixtures/media/menu-clip.mkv`
- boş altyazı durumu: `fixtures/media/glass-dark-clip.mkv`
- fatal yüzey: `fixtures/media/broken-clip.mkv`
- kusurlu kaynak: koşu boyunca `end-before-start.srt`, `menu-clip.srt` adıyla
  parlak fixture'ın yanına geçici kondu; kapanışta kaldırıldı

Ekran görüntüleri:

- `NEN-062-panel.jpg` — parlak karede panel, bar ve aktif Türkçe kaynağı
- `NEN-062-defective-source.jpg` — bakılan kullanıcı grubu ile gerçekten aktif
  Türkçe kaynağın ayrı kaldığı kusurlu satır görünümü

İki görüntüde de yalnız sentetik fixture adı var; tam yol, query, motor adı ve
özel medya metadata'sı yok.

## Panel ve yaşam döngüsü

| Adım | Sonuç |
|---|---|
| CC düğmesiyle aç / yeniden basarak kapat | ✅ Panel açıldı; ikinci basışta panel ve iki kolon erişilebilirlik ağacından çıktı |
| Yerleşim | ✅ Panel barın trailing kenarına hizalı, barın 12 pt üstünde; ürün kodunda 440×260 pt ve divider toplam genişliği büyütmeden 190 pt sınırında |
| Cam ve geçiş | ✅ Panel ve transport aynı `GlassSurface`; parlak karede açık metin, 7 pt aktif noktalar ve kolon sınırı okundu |
| Otomatik gizleme | ✅ Oynarken panel açıldı; **3,2 sn** sonra panel, bar, medya adı ve imleç görünür kaldı |
| Video alanına dış tık | ✅ Panel dışındaki player container tıklanınca panel kapandı |
| Transport tıklaması | ✅ Panel açıkken süre düğmesi paneli kapattı ve görünümü `00:20 / 00:20` → `−00:00 / 00:20` değiştirdi; kontrol eylemi kaybolmadı |
| Panel içi gezinme | ✅ Grup ve kaynak tıklamaları paneli kapatmadı; kolon 2 ve CC etiketi aynı açık panel içinde güncellendi |
| Medya değişimi | ✅ Panel açıkken `glass-dark-clip.mkv` seçildi ve yeni medya son-açılan kaydı oldu; monoton `mediaPresentationRevision` aynı basename taşıyan iki farklı URL için de artan model testiyle kapatıldı |
| Fatal durum | ✅ `broken-clip.mkv` yalnız `Bu medya biçimi desteklenmiyor.` yüzeyini gösterdi; panel kalmadı. Model testi fatal olayın pini temizlediğini ayrıca ölçüyor |
| Pencere / shutdown | ✅ Normal `⌘Q` sonrası uygulama çalışır süreç listesinden çıktı; yeniden açılış boş ve etkileşimliydi. Shutdown pin state'i model testinde sıfırlandı |

Dosya seçiciden açık bir medyayı başka medyayla değiştiren iki kabul adımında
UI denetçisi geçiş anında zaman aşımına uğradı; uygulama `⌘Q` ile normal kapandı
ve yeniden açıldığında seçilen fixture son-açılan kayıt olarak doğrulandı. Aynı
basename ve stale-panel güvencesinin deterministik kanıtı bu yüzden ayrıca
model testindedir; yalnız ekran görüntüsü zamanlamasına dayanmıyor.

## NEN-026'nın güncel 16 adımı

| # | Beklenen | Sonuç |
|---|---|---|
| 1 | Fixture hemen açılır | ✅ |
| 2 | İki kolonlu, sabit yükseklikli panel açılır | ✅ popover değil, pencere içi cam yüzey |
| 3 | `Kapalı` → `Kullanıcı Altyazıları` → `Türkçe` → `English` → `Français` → `Dil Belirsiz` | ✅ tam bu sıra |
| 4 | Dil adları endonimdir | ✅ `Türkçe` · `English` · `Français` |
| 5 | Chrome Türkçedir | ✅ `Kapalı` · `Kullanıcı Altyazıları` · `Dil Belirsiz` |
| 6 | Kullanıcı grubu vardır ve sayacı 1'dir | ✅ |
| 7 | Kusurlu sidecar soluk, seçilemez ve `biçim hatalı` der | ✅ `NEN-062-defective-source.jpg` |
| 8 | Kusurlu satır seçimi değiştirmez | ✅ AX satırı disabled; aktif Türkçe noktası yerinde kaldı |
| 9 | Türkçe grubu `Türkçe` / `Gömülü` gösterir | ✅ |
| 10 | Dil belirsiz grubu `Adsız parça` / `Gömülü` gösterir | ✅ |
| 11 | Français seçimi iki kolondaki aktif kaynağı ve CC etiketini taşır | ✅ CC etiketi `Français` oldu, panel açık kaldı |
| 12 | Kapalı seçimi boş durum metnini gösterir | ✅ `Altyazılar kapalı.` ve CC etiketi `Kapalı` |
| 13 | Tercih edilen dil başlangıçta otomatik seçilir | ✅ `Türkçe` seçili açıldı |
| 14 | Kaynak seçimi indirme/çeviri başlatmaz | ✅ mevcut seçim yolu değişmedi; ağ/provider çağrısı eklenmedi |
| 15 | Aynı kaynak iki kez görünmez | ✅ bütün kaynak grubu sayaçları 1 |
| 16 | Kanıt yalnız fixture ve güvenli görünen metin taşır | ✅ |

Sidecar gelişinde seçimin, bakılan grubun ve sabit view kimliklerinin korunması
mevcut `SubtitleMenuTests` paketi değiştirilmeden koşularak doğrulandı. İki
kolon da ayrı `ScrollView`, satır kimlikleri grup ve kaynak token'ı olarak
kaldı; NEN-062 bu semantiği değiştirmedi.

## Otomatik kapılar

- `bash scripts/test-macos.sh` → **91 test / 10 suite, 0 failure**
- Mevcut `SubtitleMenuTests` → değişmeden yeşil
- Yeni model testleri → pinli zaman aşımı, oynayan/pauseli unpin, medya/fatal/
  shutdown temizliği ve aynı basename revision artışı yeşil
- `bash scripts/build-macos-app.sh` → çıkış 0
- `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app` →
  çıkış 0
- `bash scripts/check-docs.sh` → kapanış ağacı için son turda yeniden koşulacak

İlk kapanış turunda `window resume after shutdown restarts event polling`
bir kez `.ready` beklerken `.playing` gördü. Ürün davranışı doğruydu: ilk poll
`Ready`'yi tüketip `play()` çağırıyor, fake de gerçek bridge gibi `Playing`
olayını aynı anda kuyruğa koyuyor; test tam iki poll arasına denk gelmeye
çalışıyordu. Test artık o geçici aralığı değil kararlı `.playing` durumunu
bekliyor. Tek test koşusu ve ardından tam 91-test turu yeşil geçti.

## Temizlik

Geçici `fixtures/media/menu-clip.srt` kaldırıldı; depoya girmedi.
