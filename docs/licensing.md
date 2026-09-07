# Lisans Durumu

> Nen Player **GPL-3.0-or-later** ile lisanslanmıştır. Karar ve gerekçesi:
> [`ADR-0012`](adr/0012-macos-playback-engine.md) (accepted, 2026-08-26).
> Bu dosya kararı **anlatır**; hukuki metin depo kökündeki
> [`LICENSE`](../LICENSE) dosyasıdır.

## Bugünkü durum

| | |
|---|---|
| **Lisans** | GPL-3.0-or-later |
| **Lisans dosyası** | depo kökünde `LICENSE` (GNU GPL v3 tam metni) |
| **Dağıtım** | side-loading; public dağıtım hâlâ **S11**'de ertelenmiş |
| **App Store** | **kapalı** — GPL ile uyumsuz, aşağıya bakınız |

## Neden GPL

Karar tek bir teknik kısıttan çıktı: macOS playback motoru **libmpv**
(ADR-0012 Karar 1) ve dağıtımda libmpv `.app` içine gömülecek (Karar 3).
Homebrew'un mpv'si `GPL-2.0-or-later AND LGPL-2.1-or-later` — yani GPL kollu
parçalar içeriyor.

GPL'in kuralı şu: **bu kodu kullanan programı başkasına verirsen, o programın
kaynak kodunu da açmak zorundasın.** Buradan iki yol çıkıyordu:

| | Homebrew mpv'sini göm | Kendi LGPL mpv'ni derle |
|---|---|---|
| Proje lisansı | GPL (kod açık) | serbest (kapalı olabilir) |
| Ek iş | yok | mpv + bağımlılıklarını LGPL konfigürasyonuyla derleyen build altyapısı |
| Kaybedilen özellik | yok | Blu-ray, bazı codec/filtreler |

Proje açık kaynak olacağı için ikinci sütunun ek işini yapmanın karşılığı
kalmadı.

**Neden v2 değil v3:** `core/deny.toml`'un izin listesinde **Apache-2.0** var
(uniffi ve bağımlılık ağacının büyük kısmı bu kolu kullanıyor) ve Apache-2.0
GPLv2 ile **uyumsuz**, GPLv3 ile uyumludur — patent hükmü GPLv2'nin kabul
etmediği bir ek şart sayılır. mpv `GPL-2.0-**or-later**` olduğu için v3'e
yükseltilebiliyor. Yani GPLv3 bu bağımlılık ağacındaki tek uyumlu nokta.

## Bunun pratikte anlamı

- **Kendi bilgisayarında çalıştırmak hiçbir yükümlülük doğurmaz.** GPL yalnız
  programı **başkasına verdiğinde** devreye girer.
- **Yayımladığın her sürümün kaynağı açık olmalı.** GitHub release, arkadaşına
  `.app` göndermek — hepsi dağıtımdır.
- **Türetilen işler de GPL kalır.** Nen Player'ın kodunu alıp başka bir ürün
  yapan kişi de kaynağını açmak zorunda.
- **App Store yolu kapalı.** Apple'ın şartları kullanıcıya GPL'in verdiği
  hakların ötesinde kısıtlar getirir; GPL ek kısıt konmasını yasakladığı için
  ikisi bir arada olamaz. Side-loading ve GitHub release açık.
- **Geri dönüşü yok.** Bir kez GPL altında dağıtılan sürüm geri alınamaz.
  Bu yüzden karar ilk dağıtımdan **önce** verildi.

## Bağımlılık uyumu

`core/deny.toml`'un allow listesi bugün: `MIT` · `Apache-2.0` ·
`Apache-2.0 WITH LLVM-exception` · `Unicode-3.0` · `MPL-2.0` ·
`BSD-3-Clause` · `Zlib`. Hepsi GPL-3.0 ile uyumludur — MPL-2.0 kendi §3.3'ü
ile açıkça GPL'e izin veriyor, kalanlar permissive. Bu karar `deny.toml`'u
değiştirmedi.

Yeni bir bağımlılık eklenirken **GPL-3.0 uyumluluğu** artık ek bir kabul
koşuludur; `cargo deny check` allow listesi bunu mekanik olarak koruyor.

## Açık kalan

| Konu | Nerede |
|---|---|
| **S11** — public dağıtım kanalı ve zamanı | `docs/roadmap.md` |
| **Developer ID imzası, hardened runtime, notarization** — `NEN-043` libmpv gömme kolunu kapattı (ad-hoc imzalı `.app` Homebrew olmadan çalışıyor); kalan bu üçü Apple Developer Program üyeliği gerektiriyor | dağıtımdan önce zorunlu, **S11**'de numaralandırılacak |

## İlgili

- [`LICENSE`](../LICENSE) — hukuki metin
- [`docs/adr/0012-macos-playback-engine.md`](adr/0012-macos-playback-engine.md) — karar ve gerekçe
- [`docs/roadmap.md`](roadmap.md) — S11
- [`docs/DECISIONS.md`](DECISIONS.md) — karar durumu
