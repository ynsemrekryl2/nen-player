---
id: NEN-075
title: Sidecar discovery does not see a language-suffixed subtitle
milestone: M3
size: S
state: backlog
depends_on: [NEN-025, NEN-057]
blocks: []
adr: [31]
---

# NEN-075 — Sidecar taraması dil alt-uzantılı dosyayı görmüyor

## Sonuç

`Film.mkv`'nin yanındaki `Film.tr.srt`, kullanıcı ⇧⌘O ile açıkça seçmeden de
sidecar olarak bulunur ve kataloğa girer — bugün yalnız tam basename eşleşmesi
(`Film.srt`) taranıyor.

## Bağlam

`NEN-057` uygularken ölçüldü: `subtitle_files::sidecar_of(media)` yalnız
`media.with_extension("srt")` döndürüyor, yani `Film.mkv` için aranan tek
dosya `Film.srt`. `Film.tr.srt`, `Film.en.srt` gibi dil alt-uzantılı
sidecar'lar — kullanıcı kütüphanelerinde yaygın — bu taramadan **hiç**
geçmiyor; kullanıcı onları yalnız dosya seçiciyle elle yükleyebiliyor.

`NEN-025`'in bilinçli kapsam kararıydı (tek dosya, dizin listelemesi yok —
`sidecar_of`'un kendi dokümanı: "Deliberately not a directory listing").
`NEN-057`'nin Kapsam listesinde yoktu, Kural 5 gereği oraya eklenmedi.

## Karar verilecek

Bugünkü tek-dosya taraması `security-policy.md` §4'ün "recursive search yok"
ilkesini **dizin listelemesi olmadan** sağlıyor — `fs::metadata` tek bir
bilinen yola soruluyor. Dil alt-uzantılı adları görmek en az bir dizin
listelemesi gerektirir (`Film.*.srt` deseni), ki bu bugünkü tasarımın
bilinçli olarak kaçındığı şey. Karar verilecek olan:

1. Dizin listelemesine izin ver — `admit()`'in güvenlik kapıları (symlink,
   traversal, boyut) her aday için aynen çalışır, yalnız aday **kümesi**
   büyür.
2. Sınırlı bir liste dene — yaygın dil kodlarının bir tablosu (`en`, `tr`,
   `pt-br`, …) ile sabit sayıda `stat` çağrısı; dizin listelemez ama her yeni
   dili elle eklemek gerekir.
3. Bugünkü hâli koru, yalnız belgelenmiş bir sınır olarak bırak — kullanıcı
   dosya seçiciyle elle yükler.

## Kanıt (DoD)

- [ ] Karar yazılı: hangi seçenek, neden
- [ ] Seçilen davranış negatif ve pozitif testle kanıtlı (gerçek dosya
      sistemi, `subtitle_file_gates.rs`'in deseninde)
- [ ] Negatif: seçenek 1 veya 2 ise, sidecar taraması hâlâ symlink/traversal/
      boyut kapılarından geçiyor — yeni yüzey eski kapıları atlamıyor

## Kanıt kaydı

<!-- done olurken doldurulacak -->
