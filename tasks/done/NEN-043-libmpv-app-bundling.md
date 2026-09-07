---
id: NEN-043
title: Bundle libmpv into the .app and notarize
milestone: M3
size: M
state: done
closed: 2026-09-07
depends_on: [NEN-022, NEN-024]
blocks: []
adr: [12]
---

# NEN-043 — Bundle libmpv into the .app and notarize

## Sonuç

Nen Player'ın `.app`'i, üzerinde Homebrew kurulu olmayan bir Mac'te açılır ve
oynatır.

## Kapsam

- libmpv ve dinamik bağımlılıklarının `.app/Contents/Frameworks/` altına
  kopyalanması
- `install_name_tool` / `otool -L` ile mutlak yolların `@rpath`'e çevrilmesi ve
  doğrulanması
- Ad-hoc code signing (`codesign --sign -`)
- Bundling'i tekrarlanabilir kılan script (`scripts/bundle-macos.sh`)
- GPL-3.0 yükümlülüğü: `LICENSE` ve libmpv'nin lisans metinlerinin bundle
  içinde yer alması

## YAPILMAYACAK

- **Developer ID imzası, hardened runtime, notarization, `spctl -a -vv`** →
  **S11**. Ölçüldü (2026-09-07, bu makine):
  `security find-identity -v -p codesigning` → 0 valid identities;
  `xcrun notarytool history` → "Must provide credentials". Apple Developer
  Program üyeliği ve Developer ID Application sertifikası olmadan bu
  makinede kanıtlanamaz. Ad-hoc imza (`codesign --sign -`) bu task'ın
  kapsamında kalır ve `.app`'in gerçekten kendi bundle'ındaki libmpv'yi
  kullandığını (negatif kontrol) kanıtlamaya yeter.
- Dağıtım kanalı seçimi ve zamanlaması → **S11** (App Store GPL ile uyumsuz;
  ADR-0012 → "Sonuçlar")
- Auto-update mekanizması
- Windows/Linux paketleme → M9
- mpv'yi kaynaktan derlemek — Homebrew'un ikilisi olduğu gibi gömülür
  (ADR-0012 Karar 4)

## Neden ayrı task

`ADR-0012` Karar 3: geliştirmede dinamik link yeterli, gömme bir **paketleme**
işidir ve adapter kodu iki durumda da aynıdır. `NEN-022`'nin hiçbir DoD maddesi
paketlenmiş bir `.app` istemiyor. Bu task **M3'ün çıkış kriterlerinden
değildir**; dağıtımdan (S11) önce zorunludur.

## Kanıt (DoD)

- [x] `otool -L` taraması: bundle içindeki hiçbir Mach-O `/opt/homebrew` veya
      `/usr/local` altına işaret etmiyor (script kendi taramasını çalıştırıp
      bulursa çıkış 1 verir)
- [x] `codesign --verify --deep --strict` ad-hoc imzada geçiyor
- [x] `LICENSE`, mpv'nin kendi lisans metinleri ve üretilen `THIRD-PARTY.md`
      (her gömülü dylib → Homebrew formülü, sürüm, SPDX lisansı) bundle
      içinde
- [x] **Negatif:** Homebrew'un `Cellar`'ı geçici olarak erişilemez
      kılındığında (yeniden adlandırma) gömme-öncesi `.app` açılmıyor,
      gömülü `.app` yine açılıyor ve gerçek fixture medyayı oynatıyor — yani
      gerçekten bundle'daki kütüphane kullanılıyor. (Altyazı çizimi bu
      task'ın dokunmadığı, önceki task'larda kanıtlanmış bir yol; bu koşuda
      ayrıca sınanmadı — kullanılan fixture'da altyazı yoktu.)
- [x] `bash scripts/test.sh` yeşil (yeni `bundle-macos.test.sh` dahil)

## Kanıt kaydı

Kapsam kullanıcı kararıyla daraltıldı: Developer ID imzası, hardened
runtime, notarization ve `spctl -a -vv` bu makinede kanıtlanamıyor
(`security find-identity` → 0 kimlik, `notarytool` → kimlik bilgisi yok) ve
**S11**'e ertelendi (bkz. YAPILMAYACAK). Bu task ad-hoc imzalı, kendi
kendine yeterli bir bundle üretir.

`scripts/bundle-macos.sh` (yeni) + `scripts/lib/rewrite_macho_deps.py`
(yeni, bash 3.2'nin ilişkisel dizi desteklememesi yüzünden graf işini
üstlenen yardımcı) libmpv'nin **48 dylib'lik** geçişli Homebrew kapanışını
hesaplayıp `Contents/Frameworks/`'e kopyalıyor, `install_name_tool` ile tüm
yolları `@rpath`/`@loader_path`'e çeviriyor, `Contents/Resources/licenses/`
altına `LICENSE` + mpv'nin kendi lisans metinlerini + üretilen bir
`THIRD-PARTY.md`'yi (48 satır, formül+sürüm+SPDX) koyuyor, ad-hoc imzalıyor
ve bundle'ı bağımsızca tarayıp Homebrew referansı kalmadığını doğruluyor.

Negatif kontrol gerçek makinede, gerçek `.app`'te koşuldu: `/opt/homebrew/
Cellar` geçici olarak yeniden adlandırıldığında gömme-öncesi (Homebrew'a
dinamik bağlı) snapshot `dyld: Library not loaded` ile çöküyor (kontrol
sağır değil), gömülü `.app` **aynı pencerede** açılıyor, uygulamanın kendi
son-açılanlar listesinden gerçek bir dosyayı (`GTAVI_An_Extended_Look.mp4`)
oynatıyor ve `vmmap` yüklü image listesinde tek bir `/opt/homebrew` girdisi
kalmadığını gösteriyor. İki ekran görüntüsü:
`evidence/M3/NEN-043-hidden-cellar-empty-state.png` ·
`evidence/M3/NEN-043-hidden-cellar-playback.png`. Altyazı çizimi bu koşuda
ayrıca sınanmadı (kullanılan fixture'da altyazı yoktu) — bu task'ın
dokunmadığı, önceki task'larda (`NEN-066`, `NEN-027`) kanıtlanmış bir yol.

Aynı mekanizma `scripts/tests/bundle-macos.test.sh` ile deterministik hale
getirildi: sahte bir Homebrew düzeni (iki katmanlı sembolik bağ, sürümlü
gerçek dosya) üzerinde gerçek bir `mainbin → liba → libb` zinciri
derlenip yeniden yazılıyor ve vendor prefix'i diskten kaldırıldıktan
**sonra** çalıştırılarak sınanıyor (yalnız statik `otool` okuması değil).
Negatif kontrol ayrı: yeniden yazma atlanmış ham kopya aynı koşulda çöküyor
ve bundle-macos.sh'in tarama deseni bu ham kopyada vendor yolunu buluyor —
tarama sağır değil.

Tam kanıt: `evidence/M3/NEN-043-checklist.md`. `otool -L` taraması (48
dylib, hiçbiri `/opt/homebrew`/`/usr/local`'a işaret etmiyor), `codesign
--verify --deep --strict` (`valid on disk`), `bash scripts/test.sh`
(3/3 dosya, yeni test dahil), `cargo fmt`/`clippy`/`test --workspace`/
`deny check` (Rust'a dokunulmadı, regresyon yok), `bash scripts/test-macos.sh`
(**202/202**, Swift'e dokunulmadı), `bash scripts/check-docs.sh` (çıkış 0)
hepsi yeşil. Değişiklik yalnız `scripts/` altında; Rust çekirdeği,
`nen-ffi`, `platforms/macos/Sources/**` ve `Package.swift` dokunulmadı.
