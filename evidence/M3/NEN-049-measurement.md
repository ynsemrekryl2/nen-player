# NEN-049 — Paralel macOS paketindeki kırmızının ölçümü

Tarih: 2026-09-06
Makine: Apple Silicon · macOS 27.0
libmpv: 2.5.0 (mpv 0.41.0_8, Homebrew)
Paket: `platforms/macos`, **159 test / 18 suite**

Ölçülen komut her satırda aynı: `bash scripts/test-macos.sh` — yani script'in
**varsayılan, paralel** modu. Seri karşılaştırma
`swift test --package-path platforms/macos --no-parallel` ile alındı; script
argüman geçirmiyor.

## Soru 1 — Kırmızı hâlâ üretilebiliyor mu?

`NEN-069` kapanışındaki oran bayattı: o gün paket 157 testti, bugün `NEN-036`
ile 159. Dokunulmamış `HEAD` üzerinde beş koşu:

| Koşu | Sonuç | Kırmızı olan |
|---|---|---|
| 1 | ✘ | `ContractTests.successiveMediaReportTheirOwnDisplaySize` |
| 2 | ✓ | — |
| 3 | ✓ | — |
| 4 | ✘ | aynı test |
| 5 | ✘ | aynı test |

**2/5 yeşil.** Kırmızı veren test her koşuda aynı ve `~0,2 s`'de düşüyor —
helper'ın 5 s'lik timeout'una hiç ulaşmıyor.

## Soru 2 — Kusur zaman aşımı mı, bayat değer mi?

`#expect(try waitForGeometry(engine) == geometry)` yalnız "eşit değil" diyor;
**hangi** değeri gördüğünü söylemiyordu. Geçici bir tanı satırı eklendi
(commit edilmedi) ve dört koşu daha alındı:

```
run1 exit=0
run2 exit=1  DIAG loaded aspect-4x3-clip.mkv,  saw Optional(FfiVideoGeometry(width: 160,  height: 90))
run3 exit=1  DIAG loaded anamorphic-clip.mkv,  saw Optional(FfiVideoGeometry(width: 160,  height: 120))
run4 exit=0
```

Cevap kesin: helper **giden medyanın** boyutunu döndürüyor. 4:3 klip
yüklenirken bir önceki `contract-clip.mkv`'nin 160×90'ı, anamorphic klip
yüklenirken bir önceki 4:3 klibin 160×120'si. Yani bekleme aç kalmıyor —
**yanlış cevabı zamanında** alıyor.

Mekanizma `aLoadingMediumDoesNotExposeTheOutgoingDisplaySize`'ın zaten bilerek
sabitlediği davranış: `loadfile` sonrası `MPV_EVENT_FILE_LOADED` geldiğinde
`video-out-params` hâlâ giden medyayı tarif edebiliyor. `waitForGeometry` ilk
non-nil değeri kabul ettiği için yük altında o pencereye düşüyor.

## Düzeltme — ve neden izolasyon değil

Task'ın özgün kapsamı "en dar test-runner veya suite izolasyonu" diyordu.
Ölçüm başka bir çare seçti: izolasyon bu kusuru **gizler**, helper düzeltmesi
**kaldırır**. Bekleme artık giden medyanın boyutunu dışlıyor
(`waitForGeometry(_:after:)`), beklenen boyutu değil — beklenen boyutu
vermek iddiayı kendi öncülüne sordurmuş olurdu.

Ürün kaynak kodu değişmedi; tek dosya
`platforms/macos/Tests/NenPlaybackMPVTests/ContractTests.swift`.

## Soru 3 — Oran ayırt edici mi?

Makinenin durumu oturum içinde kaydığı için ölçüm ardışık bloklar hâlinde
değil, **dönüşümlü** tekrarlandı (NEN-069'un kurduğu desen): aynı ağaçta
helper'ın eski ve yeni hâli sırayla konarak.

| Ağaç | Paralel tam paket |
|---|---|
| `HEAD`, düzeltme yok (A turu) | **2/5 yeşil** |
| Düzeltme var (A turu) | **5/5 yeşil** |
| Düzeltme geri alındı (B turu) | **1/4 yeşil** |
| Düzeltme geri kondu (B turu) | **4/4 yeşil** |
| **Toplam** | öncesi **3/9**, sonrası **9/9** |

`NEN-051`'de oran ayırt edici çıkmamıştı; burada çıkıyor. Düzeltmeli ağaçta
art arda **9 yeşil** var, yani DoD #1'in istediği "≥ 3 ardışık çıkış 0"
fazlasıyla karşılandı.

Seri mod karşılaştırması, düzeltmeli ağaçta: **2/2 yeşil** (159/159).

## Negatif kontroller

**1. Düzeltme geri alınınca semptom geri geliyor.** B turunun ilk yarısı tam
olarak bu: 1/4 yeşil, kırmızı olan her seferinde
`successiveMediaReportTheirOwnDisplaySize`. Kusur yüke bağlı olduğu için tek
koşu kanıt sayılmadı, oranla raporlandı.

**2. Bekleme, çağıranın sorusunu cevaplamıyor.** Beklenen boyutlardan biri
bilinçli olarak yanlış yazıldı (`160×120` → `161×121`) ve test kırmızıya
döndü — üstelik gerçek değeri söyleyerek:

```
✘ Expectation failed: (seen → FfiVideoGeometry(width: 160, height: 120))
                   == (geometry → FfiVideoGeometry(width: 161, height: 121))
  ↳ loaded aspect-4x3-clip.mkv, saw Optional(FfiVideoGeometry(width: 160, height: 120))
```

Dışlanan tek şey giden medyanın değeri; üçüncü bir boyut gelse test yine
kırmızı olurdu.

**3. Düzeltme "daha uzun bekleyerek" geçmiyor.** Dokuz yeşil koşuda testin
süresi **0,117–0,186 s** aralığında. 5 s'lik timeout'a yaklaşan tek koşu yok,
yani doğru değer dışlamadan hemen sonra geliyor.

## Değişmeyenler

- `aSeekIsAnsweredThroughTheSession` hâlâ `12_000 ± 100 ms` bekliyor
  (`SessionTests.swift:75`); dosyaya dokunulmadı.
- `aLoadingMediumDoesNotExposeTheOutgoingDisplaySize`'ın iddiası aynı — bayat
  değerin gerçekten var olduğunu sabitleyen test bu.
- Ürün kaynak kodu (`platforms/macos/Sources/`, `core/`) değişmedi.

## Diğer kapılar

Rust workspace **568 passed / 1 ignored**, `cargo fmt --check`, `cargo clippy
--workspace --all-targets -D warnings` ve `bash scripts/test.sh` yeşil.
