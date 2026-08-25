---
adr: 0029
title: Subtitle dili tespiti, güven eşiği ve metadata çelişki politikası
status: accepted
milestone: M2
tasks: [NEN-020]
date: 2026-08-25
---

# ADR-0029 — Subtitle dili tespiti, güven eşiği ve metadata çelişki politikası

## Durum

`accepted`

## Bağlam

Şartname §7, kullanıcı altyazılarının strict parse sonrasında dilinin tanınmasını;
§8 ise dili çözülemeyen kaynakların `Dil Belirsiz` grubunda gösterilmesini
istiyor. ADR-0010 Karar 6 katalog ile tespiti özellikle ayırdı:
`SubtitleSourceCatalog` yalnız `Option<LanguageTag>` gruplar, metne erişmez;
tespit ve güven eşiği NEN-020'nin sorumluluğudur. NEN-020 ayrıca metadata dil
etiketinin önceliğini ve metadata ile metin çeliştiğinde ne olacağını açık bir
sözleşmeye bağlamak zorunda.

Kısıtlar:

- Tespit tamamen local ve deterministik olmalı; NEN-020 ağ tabanlı dil
  servisini açıkça kapsam dışında bırakıyor.
- Kaynak ve hedef dil dinamiktir (şartname §10); yalnız birkaç tercih edilen
  dile göre daraltılmış bir detector ürün sözleşmesini karşılamaz.
- Subtitle diyaloğu K23 #4 gereği loglanamaz. Detector girdisi bellekte
  kullanılabilir, fakat `Debug`, hata, log, fixture kanıt kaydı veya başka bir
  artifact'e taşınamaz.
- `nen-domain` sıfır bağımlılıklı kalmalı. Dil tespiti ADR-0006'nın zaten
  parse/write/timeline sorumluluğunu verdiği `nen-subtitle` içinde yaşamalı.
- `LanguageTag` iki veya üç harfli BCP-47 primary subtag'i kabul ediyor;
  detector çıktısı metadata ve kullanıcı tercihlerindeki yaygın iki harfli
  etiketlerle aynı kanonik biçime gelmezse tek dil iki katalog grubuna ayrılır.
- Tespit tek tek kısa cue'lar üzerinde değil, parse edilmiş belgenin bütün
  diyaloğu üzerinde yapılabilir. Bu, detector'a tipik olarak çok cümleli ve
  uzun bir örnek verir.

Teknoloji seçimi, sayısal eşik ve metadata çelişkisinin kullanıcı-görünür
sonucu karara bağlanmazsa farklı platform akışları aynı kaynağı farklı dil
gruplarına koyabilir; düşük güvenli bir tahmin sessizce gerçek dil gibi
gösterilebilir veya açık metadata istatistiksel bir tahminle ezilebilir.

## Karar

**1. Yerleşim ve detector.** Dil çözümleme `nen-subtitle` içinde saf ve offline
bir modül olacaktır. Metin tespiti için **Whatlang 0.18** kullanılacaktır.
`whatlang` yalnız `nen-subtitle`'ın doğrudan dış bağımlılığıdır;
`nen-domain`'in sıfır-bağımlılık özelliği değişmez. Whatlang'ın opsiyonel
özellikleri açılmayacaktır.

**2. Detector girdisi.** Tespit bir `SubtitleDocument`'ın bütün cue satırları
doküman sırasında ve satırlar arasında tek bir ayırıcıyla birleştirilerek bir
kez yapılacaktır. Cue başına oylama yapılmayacak, zamanlar ve cue kimlikleri
girdiye katılmayacaktır. Birleştirilmiş metin yalnız çağrı süresince bellekte
yaşayacak; hiçbir sonuç veya hata tipi metni taşımayacak ve metin hiçbir
koşulda loglanmayacaktır.

**3. Güven eşiği.** Metadata yoksa Whatlang'ın en iyi adayının güven değeri
**`0.90`'dan büyükse** aday kabul edilecek; `<= 0.90`, detector'ın sonuç
üretememesi ve boş belge `None` döndürecektir. `None`, ADR-0010 gereği
`Dil Belirsiz` grubudur. Eşik proje sabiti olarak ifade edilecek; bağımlılığın
örtük varsayılanına bırakılmayacaktır. Bu sınır Whatlang 0.18'in kendi
`is_reliable` sözleşmesiyle aynıdır.

**4. Dil etiketi kanonikleştirmesi.** Whatlang'ın ISO 639-3 sonucu, aynı dilin
metadata ve tercihlerdeki yaygın etiketiyle birleşmesi için varsa kanonik
ISO 639-1 primary subtag'ine (`eng` → `en`, `tur` → `tr`, `cmn` → `zh` gibi)
eşlenecektir. Detector'ın desteklediği bütün diller için eşleme exhaustive
olacak; sessiz fallback veya serbest biçimli etiket üretilmeyecektir.

**5. Metadata önceliği ve çelişki.** Geçerli bir metadata `LanguageTag`'i varsa
nihai dil her zaman metadata etiketi olacaktır; metin tahmini metadata'yı
ezmeyecektir. Metin de `> 0.90` güvenle farklı bir **primary language** bulursa
çözüm sonucu `metadata conflict` durumunu ayrıca taşıyacaktır. Metadata'nın
yalnız region'ı farklıysa (`en-us` metadata, `en` metin) bu bir çelişki
değildir ve metadata'nın region'ı korunur. Metin eşik altındaysa metadata ile
farkı çelişki sayılmaz. Bu durum yalnız dil etiketleri ve güven değeriyle
gözlemlenebilir; subtitle metni taşımaz veya loglamaz.

**6. Yan etki yok.** Dil çözümleme yalnız etiketi ve çözüm kaynağını üretir.
Katalog güncellemesi çağıranın işidir; tespit kaynak seçmez, indirme veya AI
çevirisi başlatmaz.

## Gerekçe

Whatlang 0.18 saf Rust'tır, 70 dili tek pakette destekler, UTF-8 metin üzerinde
script ve güven bilgisi üretir ve MIT lisanslıdır. MIT zaten
`core/deny.toml`'ın izinli listesindedir. Ayrı model dosyası, native kütüphane
veya çalışma zamanı ağ erişimi istemediği için macOS, Android TV ve iOS/tvOS
shared core'una aynı biçimde girer. Whatlang'ın güvenilirlik sınırı hem benzersiz
trigram sayısını hem ilk iki aday arasındaki farkı puana yansıtır; 0.90 sınırı
kütüphanenin belgeli `is_reliable` davranışıyla aynı olduğundan projeye özgü,
ölçülmemiş ikinci bir katsayı icat edilmez.

Lingua kısa metinde daha yüksek doğruluğa odaklanır, fakat NEN-020 tek cue'yu
değil bütün subtitle belgesini sınıflandırır; tipik girdi çok cümlelidir.
Lingua'nın bütün 75 dil modeli varsayılan kurulumda yaklaşık 300 MB indirme
getirir ve kendi karşılaştırmasında Whatlang'dan daha yavaştır. Yalnız birkaç
modeli feature olarak seçmek bu maliyeti azaltırdı, ancak şartnamenin dinamik
kaynak dilini dar ve ürün-özel bir allowlist'e çevirirdi. Bu takas içinde
Whatlang'ın daha geniş tek-paket davranışı shared mobile core için daha uygun.

Metadata'nın kazanması NEN-020 kapsamındaki açık önceliği deterministik yapar;
çelişkiyi ayrıca taşımak ise yanlış metadata'yı görünmez kılmaz. Yalnız yüksek
güvenli metin sonucunun çelişki sayılması, iki zayıf sinyal arasındaki farkı
gerçek bir uyuşmazlık gibi sunmayı engeller. Region farkının çelişki olmaması,
detector'ın yalnız primary language çözebilmesiyle uyumludur.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Lingua, bütün modeller | Kısa metin doğruluğu daha yüksek; fakat bütün belge sınıflandırıldığı için bu avantaj küçülüyor, tüm modeller yaklaşık 300 MB indirme ve Whatlang'dan daha yüksek çalışma maliyeti getiriyor |
| Lingua, sınırlı dil feature'ları | Paket küçülür; fakat dinamik kaynak dili birkaç önceden seçilmiş dile indirir ve desteklenmeyen geçerli altyazıları sistematik olarak `Dil Belirsiz` yapar |
| Whichlang | Çok hızlı ve saf Rust; yalnız 16 dil desteklediği için ürünün dinamik kaynak dili kapsamı için fazla dar |
| Platform-native detector | Üç platformda ayrı sonuç/eşik davranışı üretir; headless shared-core golden'ı ve platformlar arası tutarlılığı bozar |
| Ağ tabanlı detector | NEN-020'nin açık non-goal'ü; subtitle diyaloğunu cihaz dışına çıkararak ayrıca yeni bir gizlilik sözleşmesi gerektirir |
| Yalnız metadata, metin tespiti yok | Etiketsiz kullanıcı dosyaları ve bozuk/eksik track metadata'sı sürekli `Dil Belirsiz` kalır; NEN-020'nin ana sonucunu sağlamaz |
| Metin metadata'yı yüksek güvende ezsin | Metadata önceliğini ihlal eder; istatistiksel bir sınıflandırma açık kullanıcı/provider etiketini sessizce değiştirebilir |
| Metadata ve metin çelişince `None` | Çelişkiyi güvenli gösterir fakat açık metadata'yı kaybedip kaynağı `Dil Belirsiz` grubuna taşır; metadata önceliği sözleşmesini anlamsızlaştırır |
| Eşik altındaki en iyi adayı yine döndürmek | NEN-020'nin güven eşiğini etkisizleştirir ve kısa/karışık metni yanlış bir dil grubuna koyar |

## Sonuçlar

**Olumlu:** ağsız ve platform bağımsız tek sonuç; düşük güvenli tahminlerde
güvenli `Dil Belirsiz` fallback'i; metadata önceliği kaybolmadan gözlemlenebilir
çelişki; metadata, tercih ve detector sonuçları için tek kanonik dil grubu.

**Olumsuz / kabul edilen maliyet:** `nen-subtitle` yeni bir dış bağımlılık ve
70 dilli exhaustive eşleme tablosu taşır. Whatlang'ın trigram modeli kısa veya
dengeli çok dilli metinlerde sonuç vermeyebilir; bu vakaların yanlış bir dil
yerine `Dil Belirsiz` olması bilinçli seçimdir. `0.90` eşiği kütüphane
semantiğine bağlıdır; detector değişirse aynı sayının anlamı korunmayabilir ve
yeni ADR gerekir.

**Geri dönüş maliyeti:** orta. Detector modülünün proje içi sonucu bağımlılık
tiplerini dışarı sızdırmayacağı için kütüphane değişebilir; ancak yeni detector
golden korpusunu, eşik semantiğini ve 70 dilli eşlemeyi yeniden doğrulamak
zorundadır.

## İlgili task'lar

`NEN-020`; tüketiciler: `NEN-025`, `NEN-026`

## Notlar

Kaynaklar: [Whatlang 0.18 API](https://docs.rs/whatlang/0.18.0/whatlang/) ·
[Whatlang `is_reliable` kaynağı](https://docs.rs/whatlang/0.18.0/src/whatlang/core/info.rs.html) ·
[Whatlang proje açıklaması](https://github.com/greyblake/whatlang-rs) ·
[Lingua model ve performans notları](https://github.com/pemistahl/lingua-rs)

Implementation sırasında `cargo deny check`, Whatlang 0.18'in
`hashbrown 0.15` üzerinden `foldhash 0.1.5` getirdiğini ve bu transitif crate'in
Zlib lisanslı olduğunu gösterdi. Zlib permissive ve OSI-onaylı olduğundan
`core/deny.toml` izin listesine gerekçeli olarak eklendi; copyleft veya ürün
dağıtım modelini değiştiren yeni bir yükümlülük doğurmadı.
