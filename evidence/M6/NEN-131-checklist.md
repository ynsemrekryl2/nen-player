# NEN-131 — Olay günlüğü penceresi UI checklist

Tarih: **2026-09-17** · Apple Silicon · macOS 27.0 · Xcode 27.0 · Swift 6.3 ·
libmpv 2.5.0 (Homebrew). `bash scripts/build-macos-app.sh` ile üretilen gerçek
`NenPlayer.app`, `computer-use` ile arka planda sürüldü (kullanıcı onayı, task
bazında). Ekran görüntüleri oturumda canlı görüldü; bu ortamda depoya
yazılamadığından gözlem **metin olarak birebir** aktarılıyor (Kural 3: UI kanıtı
screenshot **veya** checklist). Yalnız `fixtures/media/` altındaki klipler
kullanıldı; OpenSubtitles anahtarı yok, hiçbir ağ isteği yapılmadı (Kural 8).

## Checklist

- [x] **Menü var.** Menü çubuğu: `Apple · Nen Player · Dosya · Düzen · Görüntü ·
      Oynatma · Altyazı · Olaylar · Pencere · Yardım`. `Olaylar` altında
      `Olay Günlüğünü Göster` ve `Günlüğü Temizle`.
- [x] **Pencere açılıyor.** `Olaylar ▸ Olay Günlüğünü Göster` (⌥⌘L) 760×460
      `Olaylar` başlıklı ayrı pencereyi açtı; player penceresi
      (`contract-clip.mkv`) açık kaldı, yan yana.
- [x] **Satırlar tek satır.** `contract-clip.mkv` argv ile açılınca pencerede
      sırayla (saat · ton noktası · özet):
      `Medya açıldı: contract-clip.mkv (yerel dosya)` →
      `Kimlik araması başladı — yerel dosya hash'i → OpenSubtitles` →
      `Kimlik araması yapılmadı — medya hash'i hesaplanamadı` (turuncu; fixture
      klip OpenSubtitles hash'inin boyut eşiğinin altında — sağlayıcıya hiç
      sorulmadı, "eşleşme yok"tan ayrı raporlandı) →
      `Yan dosya taraması bitti — altyazı dosyası yok` →
      `OpenSubtitles araması başladı — tr, en` →
      `OpenSubtitles araması yapılmadı — ne hash ne kimlik var` (turuncu) →
      `Oynatma hazır` (yeşil) → `Gömülü altyazı izleri: 2` →
      `Otomatik seçim: yerel kaynak açıldı — Türkçe` → `Altyazı seçildi: Türkçe`.
      Hiçbir satır kırılmadı; hiçbir satırda yol, URL, hash yok — medya yalnız
      basename.
- [x] **Tıklayınca detay açılıyor / kapanıyor.** `Kimlik araması yapılmadı`
      satırına tıklandı: chevron döndü, altında monospace `Durum: hash yok ·
      Yöntem: OpenSubtitles hash eşleşmesi · Otomatik indirme: kimlik
      doğrulanmadı, kapı kapalı` açıldı. `Kimlik araması başladı` satırında
      `Yöntem · Sağlayıcı: OpenSubtitles`. Diğer satırlar yerinde kaldı.
- [x] **Temizle boşaltıyor.** `Olaylar ▸ Günlüğü Temizle` → liste boşaldı, boş
      durum: `Olay yok — Henüz olay yok — bir medya açın.`; toolbar `Temizle`
      devre dışı.
- [x] **İkinci medyada önceki satırlar kalıyor.** `menu-clip.mkv` open-with ile
      açıldı: önceki medyanın satırları duruyor, ardından
      `Medya açıldı: menu-clip.mkv (yerel dosya)` ile yeni zincir
      (`Gömülü altyazı izleri: 4`, `Yan dosya taraması bitti — 4 altyazı
      dosyası`, …) eklendi; liste kendiliğinden en alta kaydı.
- [x] **Kusur bulundu ve giderildi.** İlk koşuda `Gömülü altyazı izleri: 2`
      satırı medya başına iki kez düşüyordu (`ready` resync'te
      `catalogEmbeddedTracks` ikinci kez çalışıyor). Medya başına bir kez
      raporlanacak şekilde düzeltildi; ikinci koşuda tek satır. Regresyon
      testi: `openRecordsTheChainInOrder` — `ready` iki kez verilir, satır bir.

## Gözlem notu

Uygulama arka plandayken `Oynatma hazır` satırı ancak pencere öne gelince
düştü — playback'in etkinlik durumuna bağlı mevcut davranışı; bu task'ın
kapsamı dışında, değişiklik yapılmadı.
