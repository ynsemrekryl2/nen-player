---
id: NEN-078
title: Measure how Stremio launches an external player on macOS
milestone: M4
size: S
state: done
closed: 2026-09-07
depends_on: []
blocks: [NEN-079]
adr: []
---

# NEN-078 — Measure how Stremio launches an external player on macOS

## Sonuç

Stremio'nun macOS'ta harici oynatıcıyı **hangi mekanizmayla** çağırdığı, medyayı
hangi biçimde verdiği, başlangıç pozisyonunu taşıyıp taşımadığı ve yanında
metadata gönderip göndermediği ölçülmüş ve kayda geçmiştir.

## Bağlam

M4'ün alıcı yüzeyi (`CFBundleDocumentTypes` open-with · `CFBundleURLTypes`
scheme · positional argv) **kararlaştırılmamıştır** ve doğru cevabı yalnız
gönderen taraf söyler. Bu makinede Stremio 5.1.26 kurulu
(`/Applications/Stremio.app`), yani soru tahmin edilmek zorunda değil.

Emsal `NEN-025`: sandbox davranışı kod yazılmadan önce ölçüldü, ölçüm planlanan
çözümü **eledi** ve ADR-0034'ü doğurdu. Buradaki ölçüm de ADR'nin (`NEN-079`)
girdisidir; ölçmeden verilen yüzey kararı tahmindir.

## Kapsam

- Stremio'nun harici oynatıcı ayarının **var olup olmadığı** ve nerede olduğu
- Çağrı mekanizması: `open -a` / doğrudan `exec` + argv / `stremio://` scheme /
  open-with · hangisi ve kaç tanesi
- Medyanın verildiği biçim: `file://` · `http://127.0.0.1:<port>/...` (Stremio'nun
  yerel streaming server'ı) · uzak `https` · path mi URL mi
- Başlangıç pozisyonu taşınıyor mu; taşınıyorsa hangi anahtar, hangi birim
- Yanında metadata (başlık, yıl, sezon/bölüm, canonical ID) geliyor mu
- Bulgunun ADR'ye taşınacak hâli: hangi yüzey(ler) gerçekten gerekli

## YAPILMAYACAK

- Ürün kodu yazmak — bu bir ölçüm task'ı, çıktısı rapor
- `Info.plist`'e tip eklemek → `NEN-080`
- Yüzey kararını vermek → `NEN-079` (ADR)
- Stremio'ya sonuç/pozisyon **döndürmek** — M4 kapsam dışı, M10'da
- Stremio'nun kendi kaynağını değiştirmek veya add-on yazmak (non-goal)
- Ölçüm sırasında görülen medya URL'sini, token'ını veya port'unu rapora
  yazmak — K23; rapor **şekli** anlatır, değeri değil

## Kanıt (DoD)

- [x] `evidence/M4/NEN-078-measurement.md`: çağrı mekanizması, argüman şekli,
      pozisyon ve metadata bulguları; her biri nasıl gözlendiğiyle birlikte
- [x] Gözlem gerçek Stremio 5.1.26 ile yapıldı (sürüm rapora yazılı)
- [x] Harici oynatıcı ayarı **yoksa** veya çağrı mekanizması ölçülemiyorsa bu
      açıkça yazılır ve M4'ün kapsamına etkisi (hangi yüzeyler spekülatif
      kalıyor) kaydedilir — olumsuz sonuç da sonuçtur
- [x] Rapor K23 uyumlu: gerçek medya URL'si, query, token veya özel dosya yolu
      içermiyor

## Kanıt kaydı

**Ayar var, ama Nen Player onun kapalı listesine giremez** — Ayarlar →
Oynatıcı → Gelişmiş → "Harici oynatıcıda oynat" beş seçenekli sabit bir liste
(`Etkisizleştirildi · MPV · IINA · Infuse · M3U Playlist`), serbest metin/özel
komut alanı yok. Bu ayarı değiştirmek oynatmanın kendisini etkilemiyor; harici
oynatıcı yalnız oynatıcı ekranındaki `...` menüsünden elle tetikleniyor ve o
menü — hem `MPV` hem `IINA` ayarlıyken de — hep aynı iki sabit seçeneği
gösteriyor: "VLC içinde oynat" / "MPV içinde oynat". IINA/Infuse/M3U
Playlist'in gerçek davranışı bu makinede (kurulu olmadıkları için)
gözlenemedi.

**Mekanizma custom URL scheme değil.** Depoya girmeyen, `NEN-025` emsalindeki
gibi geçici bir prob (`$SCRATCHPAD/HandoffProbe.app`) `mpv:`/`iina:`/`vlc:`
scheme'lerini claim etti; iki ayrı tetiklemede de prob'a **hiç** düşmedi.
Bunun yerine Stremio, hedef oynatıcıyı **doğrudan, kendi CLI biçimiyle**
başlatıyor — `ps aux`'un yalnız bayrak adları okundu (URL hiç dosyaya
yazılmadı): "MPV içinde oynat" → `--start=<sn> --no-terminal <url>` (mpv'nin
kendi CLI'ı); "VLC içinde oynat" → `--start-time=<sn> --no-video-title-show
<url>` (VLC'nin kendi CLI'ı, farklı bayrak adı). Sağırlık kontrolünün kendisi
ayrı bir bulgu üretti: bu makinede gerçek Developer ID imzası yok
(`security find-identity` → 0), ve ad-hoc imzalı prob `spctl -a -vv` altında
`rejected` — böyle bir uygulama hiçbir göndericiye scheme handler olarak
görünmüyor (kontrol grubu: düzgün imzalı `claude://`/`bitwarden://` normal
çözülüyor). Stremio zaten scheme kullanmadığı için bu kısıt ölçümü
engellemedi, ama gelecekte bir scheme yüzeyi seçilirse (NEN-079) bu kısıt
NEN-043'ün kaydettiği notarization boşluğuyla birleşir.

**Pozisyon taşınmıyor.** Aynı medya iki farklı gerçek oynatma konumunda
(~00:00:48 ve ~00:15:29) harici oynatıcıya gönderildi; argv'daki başlangıç
bayrağı **her iki denemede de 0** geldi. Metadata için ayrı bir alan/bayrak
yok — yalnız URL'nin son path segmentindeki orijinal dosya adı zımni ipucu
taşıyor. Medya URL'si bu ölçümde Stremio'nun kendi loopback sunucusundan değil,
uzak bir `https` add-on host'undan geldi; loopback biçimi bu oturumda ayrıca
gözlenemedi.

**Bulgular `evidence/M4/NEN-078-measurement.md`'de tam ölçüm izi, K23
uyumlu şekilde kayıtlı** (gerçek URL/token/port hiçbir dosyaya yazılmadı,
yalnız kategorik şekil). `NEN-079`'a taşınan: argv/doküman tabanlı bir alıcı
custom scheme'den daha güvenilir görünüyor (Stremio zaten scheme kullanmıyor)
ama bu, iki sabit isim yuvasından (MPV/VLC) birini taklit etmeyi gerektirmez —
ölçüm kesin bir yüzey önerisi üretmiyor. Pozisyonun bu mekanizmadan hiç
gelmemesi M4'ün "doğru pozisyondan oynuyor" kriterini ve `NEN-081`'in
kapsamını doğrudan etkiliyor; karar `NEN-079`'a bırakıldı.

Prob temizliği doğrulandı: `lsregister -u` sonrası `lsregister -dump`'ta prob
referansı **0**; geçici kopya ve log dosyaları silindi. Stremio'nun kendi
ayarı (`Etkisizleştirildi`) ve seek edilen konum ölçüm sonunda özgün haline
geri getirildi. Ürün koduna dokunulmadı.

## Düzeltme kaydı — 2026-09-08 (`NEN-086`)

Bu kaydın canlı koşusunda başlayan uygulama, Stremio'nun keşfettiği özel bir
yardımcı bundle değildi. Kurulu Stremio shell'in `server.js` tablosu macOS'ta
MPV için sırasıyla `/usr/local/bin/mpv`, `/opt/local/bin/mpv` ve `/sw/bin/mpv`
yollarını kontrol eder; ilk mevcut executable `--start=<saniye> --no-terminal
<locator>` argv'siyle çalıştırılır. İlk yol bu makinede mevcut bir wrapper
olduğu için gözlenen uygulamaya geçmiştir.

Özgün ölçüm sonuçları ve kanıtları tarihsel kayıt olarak korunur. Launcher
korelasyonu ve geri alınabilir köprü kararı `evidence/M4/NEN-086-stremio-mpv-bridge.md`
ve `docs/adr/0044-stremio-mpv-bridge.md` içinde güncellenmiştir.
