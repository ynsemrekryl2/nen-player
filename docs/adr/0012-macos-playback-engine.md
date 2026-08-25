---
adr: 0012
title: macOS playback motoru, linkleme modeli ve proje lisansı
status: accepted
milestone: M3
tasks: [NEN-022, NEN-023, NEN-027]
date: 2026-08-26
---

# ADR-0012 — macOS playback motoru, linkleme modeli ve proje lisansı

## Durum

`accepted`

## Bağlam

`docs/product-spec.md` §4 macOS motorunu **libmpv** olarak yazıyor, ama
`docs/adr/README.md`'nin kuralı gereği başlıktaki teknoloji adı ilgili ADR
kabul edilene kadar **aday**dır. `NEN-022` (`adr: [11, 12]`) bu ADR olmadan
`done` olamaz; `docs/licensing.md` de lisans kararını açıkça buna bağlamış:

> "**ADR-0012** — macOS motor seçimi (aday: libmpv). libmpv ve bağımlılıkları
> LGPL/GPL bileşenler içerebilir. Linkleme modeli (dinamik/statik) ve hangi
> mpv derleme konfigürasyonunun kullanıldığı, seçilebilecek lisansları
> doğrudan kısıtlar."

Yani burada üç soru **birbirine bağlı** ve ayrı ayrı cevaplanamaz: hangi
motor · nasıl linklenir · proje hangi lisansla dağıtılır.

**Ölçülen durum (2026-08-26, bu makine).** Homebrew'un `mpv 0.41.0_8`
formülü kurulu; `pkg-config --modversion mpv` → **2.5.0**;
`/opt/homebrew/lib/libmpv.dylib` mevcut. Formülün bildirdiği lisans:

```
GPL-2.0-or-later AND LGPL-2.1-or-later
```

Yani Homebrew build'i **GPL kollu** — mpv `--enable-lgpl` ile de derlenebilir
(o zaman yalnız LGPL kalır) ama Homebrew'unki öyle derlenmemiş. Bağımlılıkları
arasında GPL'li bileşenler var (`libbluray`, `rubberband`, ve Homebrew'un
GPL konfigürasyonlu `ffmpeg`'i).

**Karar verilmezse ne olur:** `NEN-022` başlayamaz, dolayısıyla `NEN-023`,
`NEN-024`, `NEN-027`, `NEN-028` de başlayamaz — M3'ün tamamı durur. Ayrıca
`docs/licensing.md` ve roadmap S11/S12 belirsiz kalır ve depo lisanssız
kalmaya devam eder.

### Kısıtın günlük dille açıklaması

GPL şunu söyler: *"bu kodu kullanan programı başkasına verirsen, o programın
kaynak kodunu da açmak zorundasın."* LGPL daha gevşektir: *"kütüphaneyi
kullanabilirsin, kendi kodunu açmana gerek yok."*

Kritik nokta: **GPL yalnız dağıtımda devreye girer.** Programı kendi
bilgisayarında çalıştırmak hiçbir yükümlülük doğurmaz. Bugünkü kişisel
kullanım bu yüzden tamamen serbest; soru "ileride başkasına verecek miyiz"dir.

## Karar

Dört karar birlikte alınır.

### Karar 1 — macOS playback motoru **libmpv** olacaktır

Aday statüsü kalkar. `PlaybackEngine` portunun macOS adapter'ı libmpv'nin C
API'si (`mpv/client.h`) üzerine kurulur. Motorun adı ADR-0011 ve product-spec
§4 gereği application katmanına **sızmaz**; `nen-ports`, `nen-app` ve
`nen-ffi` motor adına göre dallanmaz.

### Karar 2 — Adapter **Swift'te**, `platforms/macos/` altında yaşayacaktır

Gerçek libmpv adapter'ı bir Rust crate'i değil, `platforms/macos/` altında bir
SwiftPM hedefidir. `nen-ffi` port'un FFI'ya uygun aynasını (`with_foreign`
trait) ve **paylaşılan contract kitini süren** bir giriş noktası açar; Swift
adapter o trait'i uygular ve kiti kendi platform testinde koşturur.

Bu, ADR-0011'in bağlamda varsaydığı ve `docs/testing-strategy.md`'nin
"libmpv adapter (macOS platform testinde)" satırının zaten söylediği şeydir —
bu ADR onu numaralı bir karara çevirir.

### Karar 3 — Geliştirmede **dinamik link**, dağıtımda **`.app` içine gömme**

- **M3 boyunca:** `pkg-config mpv` ile Homebrew'un dylib'ine dinamik link.
  Hiçbir kopyalama, imzalama veya paketleme işi yok. `scripts/doctor.sh`
  zaten tam olarak bu dosyayı arıyor (`NEN_DOCTOR_MPV_PATHS`).
- **Dağıtımdan önce:** libmpv ve bağımlılıkları `.app` bundle'ının içine
  kopyalanır, `install_name_tool` ile `@rpath`'e çevrilir, imzalanır ve
  notarize edilir. Bu ayrı bir task'tır (**NEN-043**) ve `NEN-022`'yi
  bloke etmez.

Ayrım şuna dayanıyor: **adapter kodu iki durumda da aynıdır.** Gömme bir
paketleme işidir, bir mimari değişiklik değil — M3'ün "video oynuyor mu"
sorusunu cevaplamak için gerekmiyor.

### Karar 4 — Proje **GPL-3.0-or-later** ile lisanslanacaktır

Nen Player açık kaynak olacaktır. Depo köküne `LICENSE` dosyası (GPL-3.0
tam metni) eklenir ve `docs/licensing.md` bu karara işaret edecek şekilde
yeniden yazılır. Roadmap **S12** (lisans ailesi) bu kararla **kapanır**.

Bu, Homebrew'un GPL kollu mpv'sini olduğu gibi gömebilmemizi sağlar — ayrı
bir LGPL mpv derleme altyapısı kurulmasına gerek kalmaz.

## Gerekçe

**Karar 1.** libmpv bu proje için üç somut nedenle uygun: (a) format ve codec
kapsamı ffmpeg üzerinden geliyor, yani M3'ün hedeflediği "elimdeki dosyayı aç,
oynasın" senaryosunda AVFoundation'ın desteklemediği konteynerler (MKV) ve
codec'ler çalışıyor; (b) gömülü altyazı track'lerini ve **harici altyazı
enjeksiyonunu** (`sub-add`) doğrudan veriyor — bu ADR-0011'in
`ExternalSubtitleInjection` capability'sinin macOS'taki karşılığı ve
`NEN-027`'nin dayanağı; (c) C API'si kararlı ve sürüm numaralı
(`libmpv.2.dylib`), Swift'ten `pkg-config` ile sorunsuz bağlanıyor. Bu makinede
kurulu ve çalışır durumda olduğu ölçüldü.

**Karar 2.** Gerçek belirleyici ADR-0011 Karar 4'tür: contract kiti senaryoları
**veri**dir ve "NEN-022 aynı senaryo listesini FFI üzerinden sürer; senaryolar
veri olduğu için ikinci kez yazılmaz". Adapter Rust'ta olsaydı bu kural
anlamsızlaşırdı (kit doğrudan Rust'tan koşardı) — ama o zaman da video yüzeyi
sorunu ortaya çıkar: `NEN-024`'te ekrana çizim AppKit'e ait bir katmandır ve
mpv'nin render API'si o katmanı isteyen taraftan sürülür. Adapter'ı en baştan
Swift'te tutmak, `NEN-024`'te adapter'ı taşımak zorunda kalmamızı önler.

**Karar 3.** Gömme dağıtım için **şart**, geliştirme için **gereksiz**. M3'ün
çıkış kriterlerinin hiçbiri paketlenmiş bir `.app` istemiyor; hepsi
"oynuyor/duruyor/atlıyor/altyazı görünüyor" davranışını istiyor. Gömmeyi
NEN-022'ye katmak, task'ı `install_name_tool` ve notarization ile
büyütür ve asıl kanıtı (contract kitinin gerçek motorda geçmesi) geciktirir.
Buna karşılık kararın **kendisini** şimdi kaydetmek gerekiyor, çünkü "hangi
mpv gömülecek" sorusu Karar 4'ü belirliyor.

**Karar 4.** İki seçenek vardı ve fark, ek iş miktarında:

| | Homebrew mpv'sini göm | Kendi LGPL mpv'ni derle |
|---|---|---|
| Proje lisansı | GPL (kod açık) | serbest (kapalı olabilir) |
| Ek iş | **yok** | mpv + bağımlılıklarını LGPL konfigürasyonuyla derleyen bir build altyapısı |
| Kaybedilen özellik | yok | Blu-ray, bazı codec/filtreler |

Kullanıcı kararı: **proje açık kaynak olacak.** O zaman ikinci sütunun ek işini
yapmanın hiçbir karşılığı yok — LGPL derleme altyapısının tek faydası kodu
kapalı tutabilmekti.

GPL sürümü olarak **3.0-or-later** seçildi ve bu sürüm seçimi *zorunlu*:
`core/deny.toml`'un izin verdiği lisanslar arasında **Apache-2.0** var (uniffi
ve bağımlılık ağacının büyük kısmı bu kolu kullanıyor), ve **Apache-2.0
GPLv2 ile uyumsuz, GPLv3 ile uyumludur** — patent hükmü GPLv2'nin kabul
etmediği bir ek şart sayılır. mpv `GPL-2.0-**or-later**` olduğu için v3'e
yükseltilebiliyor; yani GPLv3 bu ağaçtaki tek uyumlu nokta. `-or-later` eki,
ileride GPLv4 çıkarsa yeni bir karar gerekmemesi için.

**Bağımlılık uyumu doğrulandı.** `core/deny.toml`'un allow listesi bugün
şunlardan oluşuyor: `MIT` · `Apache-2.0` · `Apache-2.0 WITH LLVM-exception` ·
`Unicode-3.0` · `MPL-2.0` · `BSD-3-Clause` · `Zlib`. Hepsi GPL-3.0 ile uyumlu
(MPL-2.0 kendi §3.3'ü ile açıkça GPL'e izin veriyor; kalanlar permissive).
`core/deny.toml` değişmiyor.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| **AVFoundation / AVPlayer** (macOS'un yerlisi) | Format kapsamı dar: MKV ve yaygın "release" codec'lerinin çoğu desteklenmiyor. Hedef senaryo kullanıcının elindeki dosyayı açmak; motorun formatı reddetmesi ürünün ana vaadini kırar. AVPlayer M11'de iOS/tvOS için ayrı bir adapter olarak yine gelecek — portun varlık sebebi tam olarak bu. |
| **VLCKit / libVLC** | libmpv ile aynı lisans sınıfında (LGPL/GPL), yani lisans avantajı yok; buna karşılık altyazı enjeksiyonu ve property gözlemi API'si libmpv'ninki kadar doğrudan değil ve macOS paketi daha ağır. |
| **Rust tarafında libmpv crate'i (`nen-playback-mpv`)** | ADR-0011 Karar 4'ün gerekçesini geçersiz kılar (kit FFI'dan sürülmezse ikinci kopya sorunu hiç doğmaz, ama o zaman ADR-0011'in kabul edilmiş tasarımı boşa çıkar). Daha önemlisi `NEN-024`'te video yüzeyi AppKit katmanına bağlanacak; adapter'ı o zaman Swift'e taşımak, bu kararı bir milestone gecikmeyle vermekten başka bir şey olmazdı. Ayrıca ADR-0006'nın crate tablosunda böyle bir crate yok ve tabloyu değiştirmek ADR-0006'yı `superseded` etmeyi gerektirirdi. |
| **Kendi LGPL mpv'sini derleyip gömmek, projeyi kapalı tutmak** | Kullanıcı kararı projenin açık kaynak olması yönünde. Bu alternatifin tek faydası kodu kapalı tutabilmekti; karşılığında mpv ve bağımlılıklarını LGPL konfigürasyonuyla derleyen bir build altyapısı bakmak gerekiyordu. Karşılıksız maliyet. |
| **Statik link (libmpv'yi binary'ye gömmek)** | GPL yükümlülüğü açısından dinamik/statik ayrımı bizim için bir şey değiştirmiyor (proje zaten GPL olacak), ama statik link mpv'nin ffmpeg dahil tüm bağımlılık ağacını statik derlemeyi gerektirir — Homebrew bunu vermiyor. Dinamik link + bundle aynı sonucu, hazır ikililerle veriyor. |
| **Lisans kararını M3 sonrasına ertelemek** | `docs/licensing.md` kararı açıkça bu ADR'ye bağlamış ve `NEN-022` bu ADR olmadan `done` olamıyor. Ertelemek M3'ü durdurur. |

## Sonuçlar

**Olumlu:**

- `NEN-022` başlayabilir; M3'ün geri kalanının önü açılır.
- Depo artık lisanssız değil; roadmap **S12** kapanır.
- Gömme kararı verilmiş olduğu için `NEN-043` yazılabilir ve dağıtım
  (**S11**) tek bir açık iş olarak kalır.
- LGPL mpv derleme altyapısı kurma yükü tamamen ortadan kalkar.

**Olumsuz / kabul edilen maliyet:**

- **Kaynak kodu açık olacak.** GPL, dağıtılan her sürümün kaynağının
  verilmesini şart koşar; ayrıca Nen Player'dan türetilen işler de GPL
  kalmak zorundadır.
- **App Store yolu kapanır.** Apple'ın App Store şartları kullanıcıya GPL'in
  verdiği hakların ötesinde kısıtlar getirir; GPL ek kısıt konmasını yasakladığı
  için ikisi bir arada olamaz. Roadmap **S11** bunu bilerek kabul eder —
  side-loading ve GitHub release açık kalır. (Bu zaten bugünkü dağıtım
  biçimi; `docs/licensing.md` App Store hedefini S11'e ertelemişti.)
- **Kullanıcının mpv kurması gerekiyor** — NEN-043 kapanana kadar. M3 boyunca
  tek kullanıcı geliştirici olduğu için bu bir maliyet değil, ama ilk
  dağıtımdan önce kapatılması zorunlu.
- **Track enumeration NEN-022'de port seviyesinde yapılmak zorunda.**
  ADR-0011 Karar 3 track enumeration/selection'ı zorunlu tabana koyduğu için
  contract kiti onsuz geçmiyor; `NEN-023`'ün işi katalog seviyesindeki
  semantik (bitmap tespiti, metin çıkarımı) olarak daralır.

**Geri dönüş maliyeti:**

- **Karar 1'den dönmek: pahalı.** Motor değişirse adapter baştan yazılır.
  Ama portun varlık sebebi bu: `nen-ports`, `nen-app` ve UI değişmez, yalnız
  `platforms/macos/` altındaki adapter değişir.
- **Karar 2'den dönmek: orta.** Adapter'ı Rust'a taşımak FFI köprüsünü
  gereksiz kılar (silinir) ama adapter'ın tamamı yeniden yazılır.
- **Karar 3'ten dönmek: ucuz.** Linkleme bir paketleme ayarı; adapter kodu
  değişmez.
- **Karar 4'ten dönmek: pahalı ve tek yönlü.** Bir kez GPL altında dağıtılan
  sürüm geri alınamaz; kapalı kaynağa dönmek ancak libmpv'yi LGPL build ile
  değiştirip *sonraki* sürümler için mümkün olur ve o ana kadarki katkıların
  izni gerekir. Bu yüzden karar dağıtımdan **önce** veriliyor.

## İlgili task'lar

`NEN-022` (bu ADR'nin ilk kullanıcısı — adapter) · `NEN-023` (track
enumeration) · `NEN-027` (`SubtitleRenderer`, `sub-add` üzerinden enjeksiyon) ·
`NEN-043` (bundling/notarization — bu ADR ile açılacak)

## Notlar

<!-- Karar sonrası gözlemler -->
