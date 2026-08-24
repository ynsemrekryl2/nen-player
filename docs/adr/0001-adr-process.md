---
adr: 0001
title: ADR süreci
status: accepted
milestone: M0
tasks: [NEN-003]
date: 2026-08-24
---

# ADR-0001 — ADR süreci

## Durum

`accepted`

## Bağlam

Nen Player uzun soluklu, çok platformlu ve çok kararlı bir proje. Şartname
(`docs/product-spec.md` §18) mimari değişikliklerin ADR ile yapılmasını
zorunlu kılıyor.

ADR süreci tanımlanmazsa iki şey olur: (1) mimari kararlar kod içinde örtük
kalır ve gerekçesi kaybolur, (2) aynı karar aylar sonra farklı bir şekilde
yeniden verilir ve tutarsızlık oluşur.

## Karar

Her mimari karar `docs/adr/####-slug.md` altında bir ADR olarak yazılacaktır.
ADR `proposed` durumunda açılır, **kullanıcı onayıyla** `accepted` olur, ve bir
task'ın referans verdiği ADR `accepted` olmadan o task `done` olamaz.

## Gerekçe

Dosya tabanlı ADR, repo içi markdown task sistemiyle aynı yerde yaşar; git
geçmişi kararın ne zaman ve neden değiştiğini kendiliğinden kaydeder. Ek araç
gerekmez, offline çalışır, PR diff'inde görünür.

`accepted` kapısının task yaşam döngüsüne bağlanması, ADR'nin "sonradan
yazılan belge" olmasını engeller — karar koddan **önce** verilir.

## ADR ne zaman gerekir?

**Gerekir:**

- Port sınırı değişiyor (bir şey core'dan platforma veya tersine taşınıyor)
- Yeni bir port ekleniyor veya kaldırılıyor
- Teknoloji seçimi (dil, kütüphane, motor, veritabanı, binding)
- Veri formatı / persistence şeması / cache identity bileşenleri
- Güvenlik veya gizlilik kuralı ekleniyor veya gevşetiliyor
- Kullanıcı-görünür davranış sözleşmesi (şartnamedeki bir maddenin yorumu)
- Bir spike'ın go/no-go sonucu

**Gerekmez:**

- Fonksiyon/dosya adı değişikliği, iç refactor
- Test ekleme
- Doküman düzeltmesi
- Bir ADR'nin zaten kapsadığı uygulama detayı

Emin değilsen: *"altı ay sonra biri 'bu neden böyle?' diye sorar mı?"* Cevap
evetse ADR yaz.

## Akış

```
1. ADR dosyası oluştur (0000-template.md kopyası), status: proposed
2. Bağlam / Karar / Gerekçe / Reddedilen alternatifler doldurulur
3. Kullanıcıya sunulur
4. Onay → status: accepted, date doldurulur
   Ret  → status: rejected (dosya SİLİNMEZ — ret de bilgidir)
5. İlgili task(lar)ın frontmatter'ındaki `adr:` alanına numara yazılır
```

`scripts/check-docs.sh`, `adr:` alanı dolu bir task `done` iken ADR'nin
`accepted` olmadığını yakalar.

## Numaralandırma

- Sıralı, dört haneli, sıfır dolgulu: `0001`, `0002`, …
- Numara **asla yeniden kullanılmaz** — reddedilen ADR'nin numarası da ölüdür.
- `0000` şablondur, ADR değildir.

## Değişiklik

Kabul edilmiş bir ADR **düzenlenmez**. Karar değişiyorsa:

1. Yeni ADR yazılır, bağlamında eskisine referans verir
2. Eski ADR `status: superseded by ADR-XXXX` yapılır
3. Eski ADR'nin gövdesi olduğu gibi kalır

İstisna: yazım hatası düzeltmesi ve "Notlar" bölümüne ekleme serbesttir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| ADR yok, kararlar CLAUDE.md'de | Tek dosya şişer; kararın ne zaman/neden değiştiği kaybolur; alternatifler kaydedilmez |
| GitHub Discussions / issue | Repo dışında kalır, offline çalışmaz, git geçmişine bağlanmaz |
| Kod yorumlarında gerekçe | Dosya taşınınca/silinince gerekçe kaybolur; çapraz kesen kararlar tek yere sığmaz |

## Sonuçlar

**Olumlu:** kararlar tek yerde ve gerekçeli; yeni katılan biri `docs/adr/`
okuyarak mimariyi anlar; `check-docs.sh` süreci mekanik olarak zorlar.

**Olumsuz / kabul edilen maliyet:** her mimari karar için ek yazım işi; ADR
yazmaktan kaçınmak için kararı "mimari değil" saymaya eğilim oluşabilir — bu
riske karşı yukarıdaki "gerekir/gerekmez" listesi bağlayıcıdır.

**Geri dönüş maliyeti:** ucuz. Süreç bırakılırsa mevcut ADR'ler doküman olarak
kalır.

## İlgili task'lar

`NEN-003`
