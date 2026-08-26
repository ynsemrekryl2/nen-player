# NEN-024 — macOS shell manuel kabul kaydı

Tarih: 2026-08-26
Ortam: macOS 27.0 · Xcode 26.6 · Swift 6.3.3 · libmpv 2.5.0
Uygulama: `platforms/macos/.build/NenPlayer.app` (App Sandbox, app-scoped
bookmark, kullanıcı-seçimli salt-okunur dosya)
Medya: yalnız `fixtures/media/contract-clip.mkv` ve hata senaryosu için
`fixtures/media/broken-clip.mkv`

## Otomatik kanıt

- [x] `bash scripts/test-macos.sh`: 29 test, 6 suite, 0 failure.
- [x] Gerçek adapter paylaşılan contract kitini geçiyor.
- [x] Shell model testleri Ready → otomatik play, seek sınırları, volume,
  `EventsLost` tam resync, kapalı-küme hata metni ve kontrol görünürlüğünü
  doğruluyor.
- [x] Security-scoped bookmark gerçek Foundation API'siyle yeni store örneğinde
  round-trip oluyor.
- [x] `codesign --verify --deep --strict` geçiyor; entitlement'lar App Sandbox,
  app-scoped bookmark ve kullanıcı-seçimli salt-okunur dosya erişimi.

## Manuel checklist

- [x] Uygulama tek pencere ve boş durumla açıldı: sürükle-bırak alanı ile
  `Aç…` düğmesi görünür.
- [x] `contract-clip.mkv` seçilince katalog/altyazı beklemeden oynadı; libmpv
  render API'si sentetik hareketli test desenini AppKit yüzeyine çizdi.
- [x] Seek bar gerçek konumu yansıttı: `20 s → ← → 15 s`, `→ → 20 s`,
  `⇧← → 0 s`, `⇧→ → 29.999 s`.
- [x] Süre düğmesi `00:25 / 00:30` değerinden `−00:05 / 00:30` değerine geçti.
- [x] Ses kısayolları `1.00 → ↓ → 0.95 → ↑ → 1.00` olarak ölçüldü.
- [x] `Space` oynatma/duraklatma arasında geçti; duraklatıldığında kontroller
  kalıcı görünür kaldı.
- [x] Oynarken yaklaşık 2.5 saniye hareketsizlikten sonra kontroller ve imleç
  gizlendi; fare hareketinde geri geldi.
- [x] `F` tam ekrana geçti (pencere düğmeleri kayboldu), `Esc` aynı pencereyi
  normal moda döndürdü.
- [x] `⌘O` standart dosya seçiciyi açtı.
- [x] Uygulama Finder'a geçilerek arka plana alındı; öne geldiğinde konum
  `29.904 s`, durum `Bitti` olarak yeniden sorgulanmış ve doğruydu. Bildirim
  gösterilmedi.
- [x] `broken-clip.mkv` fatal yüzeyi `Bu medya biçimi desteklenmiyor.` metni ve
  `Başka dosya aç` eylemiyle gösterdi; uygulama çökmedi.
- [x] Standart `Ayarlar…` menüsü ayrı `Nen Player Ayarları` sahnesini açtı.
- [x] Seçilen fixture için security-scoped bookmark yazıldı; uygulama tamamen
  kapatılıp yeniden açıldığında boş durumda yalnız `contract-clip.mkv` satırı
  göründü ve bu satırdan medya yeniden oynadı.
- [x] Pencere başlıkları yalnız `contract-clip.mkv` / `broken-clip.mkv` oldu;
  tam yol, query veya motor adı hiçbir uygulama yüzeyinde görünmedi.

## Görsel kanıt

- `evidence/M3/NEN-024-playback.mp4` — 10.0 s, H.264, 1080×680, 5 fps;
  fixture penceresinden 50 ardışık kare. Sayaç hareketi (`12 → 18 → 24`)
  videonun gerçekten ilerlediğini gösteriyor.
- `evidence/M3/NEN-024-transport.jpg` — fixture oynatma yüzeyi, seek/süre ve
  volume kontrolleri. Görüntü yalnız uygulama penceresini içeriyor.

## İnceleme sırasında bulunan ve düzeltilen kusurlar

1. Eski `wid` embed yolu güncel macOS/libmpv üzerinde siyah yüzey üretti.
   libmpv render API + AppKit `NSOpenGLView` yüzeyine geçildi; aynı fixture
   görünür test deseniyle yeniden doğrulandı.
2. Salt-okunur sandbox yetkisiyle varsayılan (read-write) security scope
   bookmark oluşturma reddedildi. `bookmarks.app-scope` entitlement'ı ve
   `securityScopeAllowOnlyReadAccess` creation seçeneği eklendi; tam uygulama
   restart'ı sonrası yeniden açma geçti.
