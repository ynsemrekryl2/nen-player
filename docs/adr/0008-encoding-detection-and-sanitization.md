---
adr: 0008
title: Encoding tespiti ve sanitization politikası
status: accepted
milestone: M2
tasks: [NEN-015]
date: 2026-08-24
---

# ADR-0008 — Encoding tespiti ve sanitization politikası

## Durum

`accepted`

## Bağlam

Kullanıcı altyazı dosyaları ham byte olarak gelir ve encoding'i önceden
bilinmez: UTF-8 BOM'lu/BOM'suz, UTF-16LE/BE, veya Türkçe içerik için yaygın
legacy code page'ler (Windows-1254, Windows-1252). `nen-subtitle::srt::parse`
girdisini `&str` olarak alır ve bunu bilinçli olarak encoding-agnostik
tutar — modül dokümantasyonu bunu "encoding detection is NEN-015, which runs
before this" diyerek açıkça NEN-015'e devrediyor. Ayrıca altyazı metni
düşman kaynaklı olabilir (kullanıcı yüklemesi, OpenSubtitles); bidi override
karakterleri (`U+202E` vb., "Trojan Source" saldırı deseni) ve zero-width
karakterler gibi görünmez ama render/okunabilirlik açısından zararlı
karakterler içerebilir.

Karar verilmezse: her caller kendi encoding tahminini yazar (tutarsız,
denetlenemez), veya CP1254/CP1252 ayrımı gibi çözülemez bir problem
implicit/rastgele bir şekilde koda gömülür.

## Karar

**Bağımlılık:** `encoding_rs` (WHATWG Encoding Standard implementasyonu)
workspace'e eklenecektir — `core/Cargo.toml`'un ilk gerçek (aday değil) dış
bağımlılığı.

**Encoding tespiti:** Öncelik sırası — (1) BOM sniff: UTF-8 BOM → UTF-16LE
BOM → UTF-16BE BOM, eşleşirse BOM atılıp o encoding'le decode edilir ve
decode hatası **reddedilir** (typed error); (2) BOM yoksa önce sıkı UTF-8
denenir; (3) UTF-8 başarısızsa **Windows-1254**'e (tek legacy fallback)
düşülür — bu adım her zaman başarılıdır (Windows-1254 total bir fonksiyon).
CP1254/CP1252 arasında BOM'suz otomatik ayrım **yapılmayacaktır**.

**Sanitization:** Decode edilmiş metinden kontrol karakterleri (`\n`/`\r`
hariç), bidi override/embedding/isolate karakterleri (`U+202A–202E`,
`U+2066–2069`) ve zero-width karakterler (`U+200B–200D`, `U+2060`, dosya içi
`U+FEFF`) **tamamen silinir** — reddedilmez, temizlenir.

**Boyut sınırı:** Ham girdi `10 MiB`'ı aşarsa decode denenmeden reddedilir.

## Gerekçe

`encoding_rs`: Firefox/Servo'nun ürettiği, WHATWG spesifikasyonunun referans
implementasyonu; tek transitive bağımlılığı `cfg-if`; lisansı
`(Apache-2.0 OR MIT) AND BSD-3-Clause` (BSD-3-Clause kısmı WHATWG'den türeyen
index tablolarına özgü) — `core/deny.toml`'ın izin listesine `BSD-3-Clause`
bu task ile eklendi, `cargo deny check` yeşil. Elle CP1254 tablosu ve
UTF-16 surrogate doğrulayıcısı yazmak, aynı spesifikasyonu farklı bir oracle
olmadan yeniden türetmek anlamına gelir — sanitization'ın tek işi "bunu
yanlış yapma" olduğunda bu, tek küçük ve denetlenmiş bir bağımlılıktan daha
kötü bir seçimdir.

CP1254 tek fallback: CP1254 ve CP1252 yalnız 6 byte pozisyonunda ayrışır
(`0xD0 0xDD 0xDE 0xF0 0xFD 0xFE` — Türkçe Ğ/İ/Ş/ğ/ı/ş ile CP1252'nin
Ð/Ý/Þ/ð/ý/þ'si). BOM'suz genel durumda hangisinin doğru olduğunu güvenilir
şekilde ayırt etmek istatistiksel dil/encoding tahmini gerektirir — bu
NEN-020'nin kapsamına (ve açık non-goal sınırına) taşar. Projenin Türkçe
içerik önceliği nedeniyle CP1254 seçildi; sıradan Batı Avrupa aksanlı metin
(6 ayrışan pozisyon dışında) iki encoding'de de aynı şekilde çözülür, yani
kayıp yalnız o 6 spesifik karaktere özgü kesişmeyen durumda gerçekleşir.

Sanitization'ın "sil" (neutralize değil): meşru RTL metin (Arapça, İbranice)
Unicode bidi algoritmasıyla karakter özelliklerinden zaten doğru render
olur; override karakterleri yalnız görünen kodla çalışan koddan farklı
gösterme saldırı vektörüdür (Trojan Source). Escape'leyip görünür kılmak
yerine tamamen silmek, hem daha basit hem de altyazı gösterimini bozmaz.

Boyut sınırı 10 MiB: gerçek SRT dosyaları onlarca KB ile birkaç yüz KB
arasında; 10 MiB, kötü niyetli girdinin UTF-16 decode CPU/bellek maliyetini
sınırlarken meşru dosyalar için bolca pay bırakır.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Elle yazılmış CP1254/CP1252 tabloları | `encoding_rs`'in zaten çözdüğü WHATWG spesifikasyonunu, oracle'sız yeniden türetmek — hata riski yüksek, kazanç yok |
| CP1254/CP1252 arasında istatistiksel/otomatik ayrım | Dil tahmini gerektirir → NEN-020'nin açık non-goal sınırını ihlal eder |
| Bidi/zero-width karakterlerini escape'leyip görünür kılmak | Altyazı gösterimini bozar; meşru RTL kullanım için gereksiz; "sil" daha basit ve saldırı yüzeyini tamamen kapatıyor |
| Sanitization'ı reddetme (dosyayı bütünüyle atma) | Task DoD'si açıkça "temizleniyor" diyor — dosyanın geri kalanı meşru olabilir, tek kötü karakter yüzünden dosyayı reddetmek gereksiz sert |

## Sonuçlar

**Olumlu:** encoding/sanitization politikası tek yerde, denetlenebilir ve
gerekçeli; `srt::parse`'ın BOM'suz/temiz `&str` varsayımı artık gerçek bir
üst katmanla karşılanıyor; CP1254/CP1252 sınırlaması açıkça belgeli, gizli
bir varsayım değil.

**Olumsuz / kabul edilen maliyet:** workspace'e ilk gerçek dış bağımlılık
giriyor (`encoding_rs`); CP1252 kaynaklı dosyalarda 6 spesifik karakter
yanlış çözülebilir (nadir, belgeli, kabul edilen sınırlama).

**Geri dönüş maliyeti:** ucuz — `decode()`'un iç algoritması bu ADR'yi
superseded ederek değiştirilebilir, dış imza (`bytes -> Result<String,
EncodingError>`) aynı kalabilir.

## İlgili task'lar

`NEN-015`

## Notlar

—
