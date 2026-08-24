---
adr: 0006
title: Monorepo yapısı ve crate sınırları
status: accepted
milestone: M1
tasks: [NEN-007]
date: 2026-08-24
---

# ADR-0006 — Monorepo yapısı ve crate sınırları

## Durum

`accepted`

## Bağlam

`NEN-007` shared core iskeletini kuruyor: bir Cargo workspace ve içindeki
crate'ler. Bu, kod yazılmadan önce verilmesi gereken bir mimari karardır —
`docs/adr/0001-adr-process.md` → "ADR ne zaman gerekir?" listesindeki
**port sınırı** ve **teknoloji seçimi** maddelerinin kapsamına girer.

`docs/architecture.md` katman modelini, port listesini ve crate tablosunu zaten
tarif ediyor; ancak bu bir *mimari anlatı*, kabul edilmiş bir karar değil.
Karar verilmezse iki risk somuttur:

1. Crate sınırı `Cargo.toml` yerine "dikkatli olma"ya bırakılırsa, `nen-domain`
   zamanla I/O'ya veya platform tiplerine bağlanır ve M2'de geri alınması
   pahalı olur.
2. FFI yüzeyi tek bir crate'te toplanmazsa, her crate kendi binding'ini açar;
   `docs/security-policy.md` (K23) redaction kuralları tek bir sınırda
   uygulanamaz hale gelir.

Ayrıca `CLAUDE.md` kural 7 spike kodunun ürün koduna karışmamasını istiyor;
bunun karşılığı da dizin düzeyinde tanımlanmalı.

**Bu ADR'nin kapsamadığı:** shared core dilinin Rust olduğu (ADR-0002) ve
binding teknolojisinin UniFFI olduğu (ADR-0003) kararları. Aşağıda geçen
teknoloji adları **adaydır**; bu ADR yapıyı ve sınırları karara bağlar,
teknolojiyi değil. ADR-0002 `no-go` verirse dizin düzeni ve bağımlılık grafiği
korunur, taşıyıcı teknoloji değişir.

## Karar

Depo tek bir monorepo olarak, aşağıdaki dört üst dizinle yapılandırılacaktır:

| Dizin | İçerik |
|---|---|
| `core/` | Shared core workspace. `core/crates/` ürün kodu, `core/spikes/` ölçüm kodu |
| `platforms/` | Platform kabukları ve adapter'lar (`macos`, `apple-shared`, `android`, `windows`, `linux`) |
| `fixtures/` | Test verisi (subtitle, media, redakte edilmiş provider cevapları) |
| `scripts/` | Repo tooling (doctor, task-index, check-docs, build script'leri) |

Shared core, `core/Cargo.toml` altında **tek bir workspace** olacak ve
`docs/architecture.md` → "Crate sınırları" tablosundaki **on bir crate**'ten
oluşacaktır. Bağımlılık grafiği **tek yönlü ve döngüsüzdür**; her crate yalnız
aşağıdaki tabloda izin verilen crate'lere bağımlı olabilir:

| Crate | Sorumluluk | İzinli iç bağımlılıklar |
|---|---|---|
| `nen-domain` | Saf model ve değer tipleri | **hiçbiri** |
| `nen-ports` | Trait tanımları + contract test kitleri | domain |
| `nen-subtitle` | SRT/WebVTT parse-write, encoding, timeline | domain |
| `nen-identity` | Media evidence, hash, release-name parse | domain |
| `nen-catalog` | Source catalog, gruplama, dedup | domain, subtitle, identity |
| `nen-translate` | Blok pipeline, validation, checkpoint | domain, subtitle, ports |
| `nen-sync` | Offset, drift, piecewise, SyncProfile | domain, subtitle |
| `nen-persist` | Persistence adapter | domain, ports |
| `nen-providers` | mock · OpenAI · OpenRouter · OpenSubtitles | domain, ports |
| `nen-app` | Use-case / application katmanı | yukarıdakilerin hepsi |
| `nen-ffi` | FFI yüzeyi — **tek dış kapı** | app |

Üç sınır kuralı bağlayıcıdır:

1. **`nen-domain` hiçbir iç crate'e bağımlı değildir ve I/O yapmaz.** Ne dosya,
   ne ağ, ne saat, ne rastgelelik. Bunlar `nen-ports` üzerinden enjekte edilir.
2. **`nen-ffi` tek dış kapıdır.** Başka hiçbir crate FFI yüzeyi (`extern "C"`,
   binding makrosu, üretilen header) açmaz. Redaction ve typed error eşlemesi
   bu tek sınırda uygulanır.
3. **`core/spikes/` ürün kodu değildir.** Workspace member'ıdır ama hiçbir
   `core/crates/*` crate'i ona bağımlı olamaz; ok yönü yalnız spikes → crates.

Platform tarafında: `platforms/apple-shared/` Apple hedeflerinin ortak Swift
paketi olur; `platforms/macos/` yalnız macOS kabuğunu taşır. Üretilen binding
kodu **commit edilmez** — `.gitignore` içindeki `/platforms/**/generated/`
kuralı korunur ve binding tek komutla yeniden üretilir.

## Gerekçe

**Neden monorepo?** Core ile platform kabuğu arasındaki FFI kontratı her iki
tarafı aynı anda değiştiriyor. Ayrı depolarda bu, her değişiklikte sürüm
eşitleme işi doğurur; tek depoda tek commit'te doğrulanır. `scripts/` altındaki
doctor/check-docs tooling'i de zaten depo genelini denetliyor.

**Neden on bir crate, bugünden?** Sınır `Cargo.toml`'da olduğunda derleyici
zorlar; belgede olduğunda insan zorlar. `nen-domain`'e yanlışlıkla bir I/O
bağımlılığı eklemek, crate ayrımı varken derleme hatası; tek crate varken fark
edilmeyen bir `use` satırıdır. Crate'lerin bugün boş olması maliyeti düşük
tutar: sınır bedavaya kilitlenir, içerik sırası gelen milestone'da dolar.

**Neden tek FFI kapısı?** `docs/security-policy.md` K23, medya URL'si, subtitle
diyaloğu ve provider cevabı gibi verilerin loglanmasını yasaklıyor; FFI sınırı
bu verilerin core'dan çıktığı yerdir. Tek kapı, redaction'ın tek yerde
denetlenebilmesi ve NEN-006 guard testinin tek bir yüzeyi koruyabilmesi
demektir. Çok kapılı bir tasarımda guard testi eksiksiz yazılamaz.

**Varsayım:** `nen-app`'in tüm crate'lere bağımlı olması kabul edilebilir bir
"god crate" riski taşır. Bu risk bilinçli alınıyor; use-case katmanının
tanımı gereği tüm alt katmanları orkestre etmesi gerekiyor. Riskin gerçekleşip
gerçekleşmediği M5'te (`TranslationOrchestrator` yerleştikten sonra) yeniden
değerlendirilir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Tek crate (`nen-core`), modüllerle ayrım | Sınırı derleyici değil disiplin korur; `nen-domain`'in I/O'suzluğu ve FFI'ın tek kapı olması mekanik olarak kanıtlanamaz — NEN-007 DoD'unun "bağımlılık grafiği kanıtı" maddesi karşılıksız kalır |
| Her crate ayrı depo (polyrepo) | FFI kontratı core ve platformu birlikte değiştiriyor; ayrı depolarda her değişiklik sürüm eşitleme işine dönüşür. Tek geliştiricili bir projede saf maliyet |
| Core ve platform ayrı depolarda, geri kalanı monorepo | Aynı sorunun küçük hâli: M1'in tüm ölçümleri tam da bu iki tarafın kesişiminde |
| Crate'leri sırası geldikçe eklemek (önce yalnız domain + app + ffi) | Sınırlar en pahalı oldukları anda — kod zaten yazılmışken — çizilir. Boş crate eklemenin maliyeti bir `Cargo.toml`; sonradan crate ayırmanın maliyeti tüm `use` yollarının değişmesi |
| FFI yüzeyini ihtiyaç duyan her crate'in kendi açması | K23 redaction'ı tek noktada uygulanamaz, NEN-006 guard testi eksiksiz yazılamaz, ve platform tarafı crate sayısı kadar binding üretmek zorunda kalır |
| `spikes/`'ı ayrı bir workspace yapmak | Spike'ların ürün crate'lerini kullanması gerekiyor (ölçüm zaten onların üstünde); ayrı workspace path bağımlılığı ve ikinci bir `Cargo.lock` doğurur. Tek yönlü ok kuralı aynı korumayı daha ucuza verir |

## Sonuçlar

**Olumlu:** `nen-domain` izolasyonu ve tek FFI kapısı `cargo tree` ile
kanıtlanabilir hale gelir — belge iddiası değil, mekanik olgu. Yeni crate
eklemek ucuzdur; sınır ihlali derleme hatasıdır. NEN-006 redaction guard testi
tek bir yüzeyi hedefleyebilir.

**Olumsuz / kabul edilen maliyet:** on bir `Cargo.toml` bakım yüzeyi ve
başlangıçta çoğu boş crate. Crate arası tip paylaşımı zaman zaman `nen-domain`'e
tip taşımayı gerektirecek; bu, sınırın çalıştığının işareti sayılır, sürtünme
olarak değil. Ayrıca `nen-app`'in geniş bağımlılık yüzeyi izlenmesi gereken bir
risk olarak kalır.

**Geri dönüş maliyeti:** **ucuz.** Crate'ler bugün boş; birleştirmek dosya
taşımaktan ibaret. Karar ne kadar geç verilirse o kadar pahalılaşır — ADR'nin
şimdi yazılmasının nedeni budur. ADR-0002 `no-go` verse bile dizin düzeni ve
katman grafiği aynen taşınır.

## İlgili task'lar

`NEN-007` · uygulaması: `core/Cargo.toml`, `core/crates/*`

## Notlar

<!-- Karar sonrası gözlemler -->
