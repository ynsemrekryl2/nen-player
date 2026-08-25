---
id: NEN-043
title: Bundle libmpv into the .app and notarize
milestone: M3
size: M
state: backlog
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
- Code signing (hardened runtime) ve notarization
- Bundling'i tekrarlanabilir kılan script (`scripts/bundle-macos.sh`)
- GPL-3.0 yükümlülüğü: `LICENSE` ve libmpv'nin lisans metinlerinin bundle
  içinde yer alması

## YAPILMAYACAK

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

- [ ] `otool -L` çıktısı: bundle içindeki hiçbir ikili `/opt/homebrew` veya
      `/usr/local` altına işaret etmiyor
- [ ] `codesign --verify --deep --strict` ve `spctl -a -vv` geçiyor
- [ ] Notarization ticket alındı ve staple edildi
- [ ] **Negatif:** Homebrew'un libmpv'si geçici olarak erişilemez kılındığında
      (PATH/dosya gölgelenmesiyle) `.app` yine açılıyor ve oynatıyor — yani
      gerçekten bundle'daki kütüphane kullanılıyor
- [ ] `LICENSE` ve libmpv lisans metinleri bundle içinde

## Kanıt kaydı

<!-- done olurken doldurulacak -->
