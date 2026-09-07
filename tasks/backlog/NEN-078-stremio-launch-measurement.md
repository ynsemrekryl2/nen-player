---
id: NEN-078
title: Measure how Stremio launches an external player on macOS
milestone: M4
size: S
state: backlog
closed:
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

- [ ] `evidence/M4/NEN-078-measurement.md`: çağrı mekanizması, argüman şekli,
      pozisyon ve metadata bulguları; her biri nasıl gözlendiğiyle birlikte
- [ ] Gözlem gerçek Stremio 5.1.26 ile yapıldı (sürüm rapora yazılı)
- [ ] Harici oynatıcı ayarı **yoksa** veya çağrı mekanizması ölçülemiyorsa bu
      açıkça yazılır ve M4'ün kapsamına etkisi (hangi yüzeyler spekülatif
      kalıyor) kaydedilir — olumsuz sonuç da sonuçtur
- [ ] Rapor K23 uyumlu: gerçek medya URL'si, query, token veya özel dosya yolu
      içermiyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
