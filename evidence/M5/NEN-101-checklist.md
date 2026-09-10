# NEN-101 — macOS translate command and target language setting

Tarih: **2026-09-10** · Apple Silicon · macOS 27.0 (26A5425a) · Xcode CLT ·
Swift 6.3.3 · libmpv 2.5.0 (mpv 0.41.0_8, Homebrew)

ADR-0031 Karar 2 gereği yalnız `fixtures/` altındaki medya ve altyazılarla
üretildi: `fixtures/media/contract-clip.mkv` (gömülü Türkçe altyazı taşıyor)
ve `fixtures/subtitles/languages/english.srt`. Ekran görüntülerinde özel yol,
URL veya gerçek bir film adı yok.

## Adımlar ve gözlem

1. **`bash scripts/build-macos-app.sh` ile ad-hoc imzalı `.app` üretildi**,
   doğrudan `open` ile başlatıldı (Stremio handoff'u değil — bu task'ın
   kapsamı yalnız komut/ayar yüzeyi).
2. **`contract-clip.mkv` açıldı**, gömülü Türkçe altyazı otomatik seçildi
   (ADR-0031 Karar 4.3) — ekranda "Birinci Türkçe satır." göründü.
3. **Ayarlar (⌘,) açıldı: üçüncü satır "AI çeviri hedef dili" mevcut**,
   sistem dilinden bir kez tohumlanmış hâliyle "Türkçe" gösterdi (DoD'un
   varsayılan tohumlama maddesi).
4. **`Altyazı Dosyası Yükle…` ile `english.srt` yüklendi** ve `Kullanıcı
   Altyazıları` grubunda listelendi — **seçilmedi**, ekrandaki altyazı
   Türkçe kalmaya devam etti (kaynak seçme/yükleme komutu tetiklemiyor,
   negatif DoD #4'ün canlı karşılığı).
5. **`english.srt` seçildi** (badge "english.srt" oldu). **Menü çubuğunda
   yeni `Altyazı` menüsü** altında `AI ile Türkçe Çevir` etiketi hedef dil
   değişince otomatik `AI ile Deutsch Çevir`e döndü (adım 6'nın hedef dil
   değişikliğinden sonra) — etiketin hedef dile göre canlı güncellendiği
   doğrulandı.
6. **Ayarlarda hedef dil "Deutsch" olarak değiştirildi.** `Altyazı ▸ AI ile
   Deutsch Çevir` **etkin** görünüyordu (kaynak İngilizce, hedef Almanca —
   ikisi farklı).
7. **Komut verildi.** İş arka planda başlayıp bitti (mock provider anlık);
   menüde yeni bir `Deutsch` başlığı belirdi, altında **"AI çevirisi (de)"**
   satırı, `AI` rozetiyle. **Ekrandaki altyazı ve seçili token değişmedi**
   (badge hâlâ "english.srt") — §9'un "zorla AI çıktısına geçilmez" kuralı
   canlı doğrulandı.
8. **Negatif: kaynak zaten hedef dilde.** Gömülü Türkçe altyazı yeniden
   seçildi, Ayarlar'da hedef dil tekrar "Türkçe" yapıldı. `Altyazı` menüsü
   `AI ile Türkçe Çevir`i **soluk (devre dışı)** gösterdi — ekran görüntüsü
   ile doğrulandı (bkz. aşağıdaki yakınlaştırılmış kırpma).

## Yol üstünde bulunan ve düzeltilen gerçek kusur

İlk koşuda adım 7 `"Çeviri deposu kullanılamıyor."` (`StoreUnavailable`)
hatasıyla düştü. Kök neden: `nen-persist::FilesystemArtifactStore::new`
kökü `fs::canonicalize` ile açıyor — bu, **kökün zaten var olmasını** şart
koşuyor (`artifacts/` alt dizinini kendisi yaratıyor, kökü değil).
`PlayerModel.defaultTranslationStoreRoot()` `~/Library/Application
Support/NenPlayer/`'ı döndürüyordu ama hiçbir yer bu dizini **yaratmıyordu**
— temiz bir kurulumda ilk komut her zaman düşerdi. Swift testleri bunu
yakalayamadı çünkü `TempFixture` kendi kökünü zaten yaratıyor.

Düzeltme: `translateSelectedSubtitle()` motoru kurmadan önce
`FileManager.default.createDirectory(at: storeRoot, withIntermediateDirectories: true)`
çağırıyor; başarısızlık `.StoreUnavailable` ile aynı yüzeye düşüyor. Düzeltme
sonrası `.app` yeniden derlendi ve tam akış (adım 1-8) baştan tekrarlandı —
temiz bir profil üstünde (`~/Library/Application Support/NenPlayer/` bu
oturumdan önce yoktu, `ls` ile doğrulandı) ikinci koşu sorunsuz tamamlandı.
`bash scripts/test-macos.sh` düzeltmeden sonra **249/249** yeşil kaldı
(Swift testleri zaten kendi kökünü yarattığı için bu regresyonu hiç
görmemişti — kanıt kaydına dürüstçe not düşülüyor).

## Diskteki kanıt

Gerçek koşu sonunda `~/Library/Application Support/NenPlayer/artifacts/`
altında tek bir `<64hex>.json` dosyası oluştu (bir başarılı çeviriye karşılık
geliyor) — ADR-0017 Karar 2'nin içerik-adresli tek dosya deseniyle birebir.
Kanıt toplandıktan sonra bu geliştirme-zamanı dizini temizlendi (kullanıcının
makinesinde kalıcı bir iz bırakmamak için); üretim davranışı değişmedi.

## Ekran görüntüleri

- Adım 3: Ayarlar penceresi, üç satır (Birinci/İkinci tercih edilen dil +
  AI çeviri hedef dili — "Türkçe").
- Adım 6-7: `Altyazı ▸ AI ile Deutsch Çevir` etkin; komuttan sonra menüde
  `Deutsch ▸ AI çevirisi (de)` satırı, seçili altyazı hâlâ `english.srt`.
- Adım 8: `Altyazı ▸ AI ile Türkçe Çevir` soluk/devre dışı (yakınlaştırılmış
  kırpma ile doğrulandı).

Bu dosya bir kod incelemesi sırasında elle üretildi; ekran görüntüleri
oturumun kendi aracıyla (computer-use, kullanıcı onaylı) alındı, depoya ayrı
dosya olarak eklenmedi — adımlar ve gözlemler yukarıda metinle kayıtlı
(NEN-024/028 emsalindeki checklist formatı).
