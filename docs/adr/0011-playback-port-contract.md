---
adr: 0011
title: PlaybackEngine capability modeli ve contract test yaklaşımı
status: accepted
milestone: M3
tasks: [NEN-021, NEN-022, NEN-023]
date: 2026-08-25
---

# ADR-0011 — PlaybackEngine capability modeli ve contract test yaklaşımı

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §4 `PlaybackEngine`'in **capability tabanlı bir port**
olmasını ve en az 19 operasyonu kapsamasını istiyor; `docs/architecture.md`
application katmanının **motor adına göre asla dallanmamasını** ve
capability'si olmayan bir operasyonun **typed error** dönmesini (panik veya
sessiz no-op değil) şart koşuyor.

`ADR-0026` ownership yönünü kapattı: **core-owned session, reverse callback**
(yön A). Aynı ADR, NEN-021'in kontratının iki **ölçülmüş** riski açıkça ele
almasını zorunlu kıldı — bunlar bu ADR'nin 1. ve 2. kararlarıdır:

1. **Backpressure/backgrounding.** NEN-029'un proxy deneyi, Rust tarafındaki
   üretici thread'in Swift tüketicisi hazır olmasa da üretmeye devam ettiğini
   gösterdi (tüketici 2 sn askıya alındı, üretici tıklamaya devam etti, sonra
   birikme temizlendi). Sınırsız birikme yerine bir teslimat disiplini gerekiyor.
2. **Reentrancy disiplini.** Bir callback içinden, delivery gate'in kilidini
   isteyen bir core metodunun **aynı thread'den senkron** çağrılması
   self-deadlock üretiyor (NEN-029'da kod incelemesiyle kanıtlandı; canlı
   çalıştırılmadı, çünkü gerçek bir deadlock `live_engines()` sayacını
   binary'nin geri kalan testleri için kalıcı bozardı).

Karar verilmezse: NEN-021 kontratı bu iki riski örtük bırakır, NEN-022'de
gerçek libmpv adapter'ı yazılırken teslimat modeli adapter'a göre şekillenir —
yani port'un anlamı adapter'dan adapter'a değişir, ki bu portun varlık
sebebini ortadan kaldırır.

Ayrı bir kısıt `docs/milestones/M3-macos-slice.md` çıkış kriterlerinde:
**"gerçek libmpv adapter'ı, fake adapter ile *aynı* contract kitini
geçmelidir."** Kit `nen-ports` içinde (Rust) yaşıyor; gerçek adapter ise
ADR-0026 gereği platform shell'de (Swift). Kitin nasıl paylaşılacağı bu yüzden
bir tasarım kararıdır, uygulama detayı değil.

## Karar

Dört karar birlikte alınır.

### Karar 1 — Event teslimatı: bounded kuyruk + sınıfa göre coalescing

Event stream **bounded** bir kuyrukla teslim edilir. Her olay tipi bir
**teslimat sınıfı** taşır:

| Sınıf | Olaylar | Kuyruk davranışı |
|---|---|---|
| **Coalescing** | `PositionChanged` | Kuyrukta **en fazla bir** bekleyen örnek tutulur; yenisi eskisinin yerine geçer (son değer kazanır). Ara değerlerin düşmesi **kayıp değildir** — pozisyon zaten mutlak bir değerdir, en yenisi doğru olandır. |
| **Kritik** | `StateChanged` · `SeekCompleted` · `TracksChanged` · `EndReached` · `Failed` | Sıra korunur, **sessizce düşürülmez**. |

Kritik olaylar kuyruk sınırını aşarsa teslimat **sessizce kaybetmez**: kuyruk
`EventsLost { dropped }` işaretiyle kapatılır ve tüketici tam durumu yeniden
sorgulamakla (resync) yükümlüdür. Yani tüketicinin gördüğü akış her zaman ya
eksiksizdir ya da eksik olduğunu **söyler**.

Üretici, tüketici hazır olmadığında **bloke olmaz** — NEN-029'un ölçtüğü
backgrounding senaryosunda uygulama arka plandayken kuyruk dolar, öne gelince
tüketici `EventsLost` görür ve resync eder.

### Karar 2 — Reentrancy: callback içinden senkron core çağrısı yasaktır

Kontrat, bir event callback'i **içindeyken** aynı thread'den yapılan senkron
port çağrısını **yasaklar**. Böyle bir çağrı deadlock'a girmez; typed error
(`PlaybackError::ReentrantCall`) ile **hemen döner**.

Uygulama: callback dispatch edilirken thread-local bir "callback içindeyim"
işareti kurulur, port metotları ilk iş olarak bunu kontrol eder. Kural
adapter'a değil **porta** aittir; her adapter aynı yasağı miras alır.

### Karar 3 — Capability yalnız **farklılaşan** yetenekleri sayar

`Capability` kümesi bir motorun **isteğe bağlı** yeteneklerini listeler.
Aşağıdaki taban her adapter için **zorunludur** ve capability olarak
sorgulanmaz — bunlar olmadan bir şey `PlaybackEngine` değildir:

> load · play · pause · stop · absolute seek · position · duration · state ·
> audio & subtitle track enumeration · audio & subtitle track selection ·
> event stream · shutdown

İsteğe bağlı capability'ler:

| Capability | Neden isteğe bağlı |
|---|---|
| `EmbeddedTextExtraction` | Şartname §4 bunu açıkça capability sayıyor; bitmap (PGS/VobSub) track'lerde metin **yok** (NEN-023) |
| `ExternalSubtitleInjection` | Şartname §4 bunu açıkça capability sayıyor; renderer yolu platforma göre değişiyor (NEN-027) |
| `PlaybackRate` | Motorlar arasında en çok farklılaşan yetenek; kısıtlı aralık veya hiç yok olabilir |
| `Volume` | Bazı platformlarda ses yalnız sistem düzeyinde kontrol edilir |

**Relative seek capability değildir.** Port, absolute seek ve position
üzerinden varsayılan bir implementasyon sağlar; bir adapter atomik bir native
relative seek'e sahipse bunu override eder. Aynı çağrının bazı motorlarda var
bazılarında yok olması, application katmanını gereksiz bir dallanmaya zorlardı.

Capability'si olmayan bir operasyon çağrılırsa `PlaybackError::Unsupported`
döner — **panik yok, sessiz no-op yok**.

### Karar 4 — Contract kiti senaryoları **veri**dir, kod değil

Contract kiti `nen-ports` içinde, senaryoları **veri olarak** (adım dizisi +
beklenen gözlem dizisi) tanımlar. Kiti sürmek ile kiti **tanımlamak** ayrılır:

- `nen-ports` senaryo listesini ve bir Rust runner'ını sağlar → NEN-021'de
  fake adapter bununla doğrulanır.
- NEN-022'de gerçek libmpv adapter'ı **aynı senaryo listesini** FFI üzerinden
  sürer; senaryolar veri olduğu için ikinci kez **yazılmaz**.

Senaryo listesi tek kaynaktır: bir senaryo eklendiğinde her iki adapter da
otomatik olarak ona tabi olur.

## Gerekçe

**Karar 1.** ADR-0026'nın sunduğu iki seçenekten (sınırlı/coalescing kuyruk vs.
demand-driven/ack) coalescing seçildi. **Belirleyici sebep performans
değildir** — bu, ADR'nin ilk taslağında yanlış ağırlık verilmiş ve inceleme
sırasında düzeltilmiş bir noktadır. Sayılar şöyle: NEN-029'un ölçtüğü per-call
maliyet p50 36.67 µs + ~40 µs MainActor hop ≈ 77 µs; ack **her olaya bir hop
daha** eklerdi, yani ≈ 117 µs. 60 Hz'de 4.6 ms/sn → **~7 ms/sn**; 60 fps'de bir
karenin (16.7 ms) %0.46'sı yerine %0.70'i. **İkisi de hissedilmez** —
ADR-0026'nın A/B karşılaştırmasında olduğu gibi, bu ölçekte performans kararı
taşımıyor.

Ack'in gerçek kusuru yapısal: **yığılmayı çözmez, yerini değiştirir.** Video
gerçek zamanda oynuyor; tüketici "hazırım" demediğinde motorun konumu ilerlemeye
devam ediyor. NEN-029'un backgrounding deneyi tam olarak bunu ölçtü — tüketici
2 sn askıya alındı, üretici tıklamaya devam etti. O anda hâlâ aynı iki seçenek
var: haberi at, ya da biriktir. Ack bu kararı ortadan kaldırmıyor, üreticiye
devrediyor — yani sınırsız kuyruk sorunu geri geliyor ya da yine bir düşürme
kuralı yazmak gerekiyor. **Ack tüketicinin hızını sınırlayabilir, üreticinin
gerçek zamanını sınırlayamaz.**

İki ikincil kusur: kritik bir olayın (`EndReached`) teslimi bir round-trip
gecikir; ve her iki taraf sürekli demand muhasebesi tutmak zorunda kalır — bu,
**her yeni adapter'ın** tekrar tekrar doğru yapması gereken bir yük.

Coalescing ise yüksek frekanslı olan tek olay tipinde (pozisyon) **hiçbir bilgi
kaybetmiyor**, çünkü pozisyon mutlak bir değer — ara değerlerin tüketiciye ayrı
ayrı ulaşmasının kullanıcı-görünür karşılığı yok. Kritik olaylar zaten kullanıcı
eylemi hızında üretiliyor, coalescing gerektirmiyorlar.

`EventsLost` işareti, "sessiz kayıp" ile "bloke olan üretici" arasındaki üçüncü
yol: kuyruğun dolması ölçülmüş gerçek bir durumdur ve tasarım bunu **görünür**
kılmak zorunda.

**Karar 2.** ADR-0026 iki seçenek bırakmıştı: yasaklamak ya da çağrıyı zorunlu
olarak asenkron dispatch etmek. Asenkron dispatch, kullanıcının gördüğü sırayı
bozar — callback içinden yapılan bir `pause()` çağrısı, kuyruktaki sonraki
olaylardan **sonra** işlenirdi. Yasak, hatayı derleme zamanına değil ama en
azından **ilk çağrıya** taşıyor ve deterministik olarak test edilebiliyor.

**Karar 3.** Alternatif "her operasyon bir capability"dir. Bu, her adapter'ı
19 capability bildirmeye zorlar ve bunların çoğu her motorda her zaman `true`
olurdu — application katmanı için bilgi taşımayan gürültü. Capability'nin
amacı motorlar arasındaki **gerçek** farkı ifade etmek; hiç farklılaşmayan bir
yeteneği capability yapmak, `if capabilities.contains(.play)` gibi anlamsız
kontroller üretir.

**Karar 4.** M3 çıkış kriteri gerçek adapter'ın **aynı** kiti geçmesini
istiyor. Kit Rust fonksiyonları olarak yazılırsa, gerçek adapter Swift'te
olduğu için NEN-022'de kitin ikinci bir kopyası yazılmak zorunda kalır — ve iki
kopya zamanla ayrışır, yani "aynı kit" iddiası ilk değişiklikte sessizce
yalan olur.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| **Demand-driven (ack tabanlı) teslimat** — tüketici hazır olduğunu bildirdikçe olay çekilir. | **Performans sebebiyle değil:** ack'li maliyet 60 Hz'de ~7 ms/sn olurdu (karenin %0.70'i), bugünkü ~4.6 ms/sn (%0.46) kadar hissedilmez. Reddedilme sebebi yapısal — video gerçek zamanda oynadığı için ack üretimi durduramaz; yığılma kararını yok etmez, üreticiye devreder, yani sınırsız kuyruk sorunu geri gelir ya da yine bir düşürme kuralı gerekir. Ayrıca kritik olayı bir round-trip geciktirir ve her adapter'a demand muhasebesi yükü bindirir. |
| **Sınırsız kuyruk** | NEN-029'un backgrounding deneyi tam olarak bunun tehlikesini ölçtü: tüketici askıdayken üretici üretmeye devam ediyor. Sınırsız kuyruk, arka planda kalan bir uygulamada bellek tüketimini playback süresiyle doğru orantılı büyütürdü. |
| **Kritik olayları da coalesce etmek** | `StateChanged` ve `SeekCompleted` sıralı anlam taşıyor; `buffering → ready` çiftinin ilkini düşürmek tüketiciyi yanlış duruma sokar. Coalescing yalnız mutlak-değerli olaylarda güvenli. |
| **Reentrant çağrıyı asenkron dispatch etmek** (ADR-0026'nın diğer seçeneği) | Callback içinden yapılan komutun, kuyruktaki sonraki olaylardan sonra işlenmesine yol açar — kullanıcının gördüğü neden-sonuç sırasını bozar. Yasak daha az esnek ama davranışı öngörülebilir. |
| **Her operasyonu capability yapmak** | 19 capability'nin çoğu her motorda `true` olur; capability kümesi motorlar arası farkı değil, operasyon listesini tekrar eder. Application katmanına bilgi taşımayan kontroller ekler. |
| **Capability'yi hiç kullanmayıp eksik operasyonda `Unsupported` dönmek** (sorgulanamaz capability) | Application katmanı bir özelliği **denemeden önce** bilemez; UI'da altyazı enjeksiyonu düğmesini gösterip gösterememe kararı ancak çağrı yapıp hata alarak verilebilirdi. §4'ün "capability tabanlı port" ifadesine de aykırı. |
| **Contract kitini Rust fonksiyonları olarak yazmak** | Gerçek adapter Swift'te olduğu için NEN-022'de kitin ikinci kopyası yazılırdı; iki kopya ayrışınca "aynı kit" iddiası sessizce yalana döner. M3 çıkış kriteri bunu açıkça yasaklıyor. |
| **Kiti baştan FFI üzerinden sürülebilir yazmak (NEN-021'de FFI runner dahil)** | NEN-021'in kapsamı port contract'ı; FFI runner'ı gerçek adapter olmadan doğrulanamaz. ADR yalnız senaryoların **veri** olmasını zorunlu kılıyor — bu, NEN-022'nin runner'ı eklemesi için yeterli ve gereklidir. |

## Sonuçlar

**Olumlu:** NEN-021 kontratı ADR-0026'nın iki ölçülmüş riskini örtük
bırakmıyor; her ikisi de test edilebilir kurala dönüşüyor. Capability kümesi
küçük ve anlamlı kalıyor (4 giriş), application katmanı için gerçek bilgi
taşıyor. NEN-022 contract kitini yeniden yazmak zorunda kalmıyor.

**Olumsuz / kabul edilen maliyet:**

- **Pozisyon olaylarının ara değerleri kaybolur.** Tüketici her tick'i
  görmeyebilir. M7'nin (manuel sync) pozisyon çözünürlüğüne ihtiyacı ara
  değerlerden değil, sorgulama anındaki `position()`'dan gelmeli — bu ADR bunu
  bir kısıt olarak kabul ediyor.
- **Reentrancy yasağı adapter yazarına yük bindiriyor.** Callback içinden
  komut vermek isteyen platform kodu, çağrıyı kendisi başka bir thread'e/
  event loop turuna ertelemek zorunda.
- **`EventsLost` sonrası resync tüketicinin sorumluluğu.** Kontrat bunu
  zorunlu kılıyor ama uygulamasını platform shell'e bırakıyor; unutulursa
  UI bayat kalır. NEN-024/NEN-026'da bu açıkça ele alınmalı.

**Geri dönüş maliyeti: orta.** Karar 3 ve 4'ten dönmek ucuz (capability
kümesine giriş eklemek, senaryo listesine senaryo eklemek zaten öngörülen
değişiklik). Karar 1'den demand-driven'a dönmek, kontratın event yüzeyini ve
her iki adapter'ın teslimat kodunu yeniden yazmayı gerektirir — NEN-029'un
spike kodu terfi etmediği için (`core/spikes/` altında kalıyor) kaybedilen
şey ölçüm değil, üstüne yazılmış M3 kodudur.

## İlgili task'lar

`NEN-021` (yazarı) · `NEN-022` (contract kitini gerçek adapter'da sürer) ·
`NEN-023` (track enumeration, `EmbeddedTextExtraction` capability'sini kullanır)

## Notlar

<!-- Karar sonrası gözlemler -->
