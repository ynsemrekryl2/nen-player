# NEN-068 — Tam ekran çıkışı ve ardışık medya kabulü

Tarih: **2026-09-05** · Apple Silicon · macOS **27.0 (26A5421a)** ·
Swift **6.3.3**, Xcode **26.6**, Rust **1.98.0**, libmpv **2.5.0**.
Build: debug, ad-hoc imzalı `NenPlayer.app`, Homebrew dinamik libmpv.

## Yeniden üretim ve neden

Kullanıcı video/ses ile fare/klavye kontrollerinin birlikte durduğunu bildirdi.
Önceki checklist'in “otomasyon ulaşamıyor, uygulama sağlam” açıklaması
kanıtlanmış değildi. Gerçek uygulamada uzun, hareketli ve sesli sentetik
klip açıldı; oynarken `F`, ardından `Esc` gönderildi. Giriş tamamlandı;
çıkış `NSWindowWillExitFullScreenNotification` sonrasında takıldı.
`NSWindowDidExitFullScreenNotification` gelmedi ve pencere 1470×923 kaldı.

`sample <NenPlayer PID> 3` ile alınan örnekte ana iş parçacığı event loop'ta
çalışabiliyordu; kalıcı `renderLock`/ana iş parçacığı deadlock'u gözlenmedi.
Video output thread'inin `flip_page` beklemesi, takılan pencere geçişiyle
birlikte görüldü. Bu gözlem renderer'ı değiştirme gerekçesi sayılmadı.
Geçici tanılama yalnız pencere ölçüsü, oran, olay türü ve oynatma milisaniyesi
kaydetti; ürün derlemesinden kaldırıldı.

Aynı klip ve aynı `F` → `Esc` koşumu:

| Değişken | Sonuç |
|---|---|
| Özgün `WindowGeometryWriter` | Çıkış bildirimi yok, 1470×923'te kalıyor |
| Geometri yazıcısı tümüyle devre dışı | Çıkış tamamlanıyor, oynatma sürüyor |
| Yalnız geçiş boyunca yazımları ertelemek | Kusur sürüyor |
| Buna ek olarak minimum boyutu bırakmak | Kusur sürüyor |
| Özgün yazıcıda yalnız oran sıfırlama çağrısını düzeltmek | Çıkış tamamlanıyor, 693×390'a dönüyor; 7, 8, 9, 10, 11 saniye pozisyonları geliyor |

Gerçek `NSWindow` üzerinde ölçülen ayrım:

| İşlem | `contentAspectRatio` | `resizeIncrements` |
|---|---|---|
| Yeni pencere | (0,0) | (1,1) |
| Oranı 640×360'a kilitle | (640,360) | (0,0) |
| `contentAspectRatio = .zero` | (0,0) | **(0,0)** |
| `resizeIncrements = NSSize(width: 1, height: 1)` | (0,0) | **(1,1)** |

Önceki test yalnız oran getter'ını kontrol ettiği için yanlış sıfırlamayı
geçiriyordu. Düzeltme, Apple'ın belgelediği karşılıklı dışlama davranışını
kullanır: oran kilidi, birim resize increment atanarak kaldırılır.
Kaynak: [NSWindow.aspectRatio](https://developer.apple.com/documentation/appkit/nswindow/aspectratio).

Son kod ayrıca `willEnterFullScreen` ile `didExitFullScreen` arasında
pencereye oran, minimum veya frame yazmaz. Son geometri ve medya revision'ı
saklanır; yeni medya tam ekranda açılmışsa boyutlandırma çıkışa ertelenir.
Video kalmamışsa eski oran geri getirilmez. Minimumu sıfırlama deneyi ürün
koduna alınmadı; renderer ve FFI sözleşmesi değiştirilmedi.

## Ardışık medyada bulunan ikinci kusur

4:3 klipten anamorphic klibe geçişte ilk boyut 600×450 kaldı; elle resize
sonrasında doğru 16:9 oranına oturdu. Ölçüm, yeni medya `buffering` durumundayken
`video-out-params`'ın önceki **160×120** boyutunu verdiğini, `video-params`'ın
ise henüz mevcut olmadığını gösterdi. Sonraki `playing` okuması **1024×576**
veriyordu. Pencere yeni revision'ın ilk boyutlandırmasını eski sayıyla
harcamış oluyordu.

Adapter artık `.loading` sırasında `nil` döndürür. Yeni dosyanın yüklenmesi
ve geometri olayı beklenir; başlangıç boyutlandırması eski VO sayısıyla
harcanmaz. Test, gerçek motorda eski boyutun bulunduğu bu kısa aralığı
adapter'ın loading fazını sabitleyerek deterministik sınar; ayrıca gerçek
ardışık 16:9 → 4:3 → anamorphic → audio-only yüklemeleri kontrol edilir.

## Negatif kontroller

| Kontrollü geri alma | Gerçek sonuç |
|---|---|
| Birim increment yerine sıfır oran | 4 test başarısız, 5 assertion |
| Tam ekran yazım korumasını kaldırma | 2 test başarısız, 7 assertion |
| Loading sorgu korumasını kaldırma | Eski boyutun reddi testi başarısız |

Her kontrol sonrası doğru kaynak geri kondu. İlk negatif kontrol girişimi,
Python alt sürecinin CommandLineTools SDK'sını Xcode Swift ile eşlemesi
nedeniyle **derleme aşamasında** kaldı; kanıt sayılmadı. Aynı Xcode SDK'sı
explicit seçilerek tekrarlanan koşumlarda yukarıdaki assertion hataları alındı.

## Sentetik klibi yeniden üretme

Uzun klip yalnız kabul koşusu içindir; üretilen medya geçici dizinde tutulur.
180 saniye, 640×360, 15 fps, H.264 video ve 440 Hz mono Opus ses içerir.

```bash
ffmpeg -y -loglevel error \
  -f lavfi -i 'testsrc2=size=640x360:rate=15:duration=180' \
  -f lavfi -i 'sine=frequency=440:duration=180' \
  -c:v libx264 -preset ultrafast -crf 35 -g 30 \
  -c:a libopus -b:a 12k -ac 1 /tmp/nen068-motion.mkv
```

Kısa oran fixture'ları ve üretim tarifleri `fixtures/media/` içindedir.
Gerçek pencere ölçümleri `CGWindowListCopyWindowInfo` ile, kenar sürüklemesi
mouse-down/drag/up olaylarıyla alındı. Kanıt ekranları yalnız sentetik medya
oynatan uygulama penceresini içerir.
