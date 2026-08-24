# Nen Player

Yerel ve uzak medyaları oynatan, erişilebilir bütün altyazı kaynaklarını tek bir
katalogda toplayan, seçilen altyazıyı isteğe bağlı olarak AI ile çeviren ve
video ile senkron gösteren multiplatform medya oynatıcı.

> **Durum: M0 — Foundation.** Henüz ürün kodu yoktur. Bu repo şu anda roadmap,
> milestone, task ve ADR sistemini içerir. İlk kod M1 (Core Technical Spike)
> ile başlar.

## Temel akış

```
media handoff / dosya seçimi
  → medya mümkün olan en kısa sürede oynatılır
  → medya kanıtları toplanır
  → altyazı kaynakları arka planda taranır
  → tek altyazı kataloğu oluşturulur
  → kullanıcı bir altyazı seçer
  → kullanıcı isterse AI çeviri komutu verir
  → çeviri sıkı biçimde doğrulanır
  → kalıcı subtitle artifact oluşturulur
  → video üzerinde senkron gösterilir
```

## Mimari özet

Platform-native UI → application API → **cihaz içinde çalışan shared portable
core** → platform portları ve adapter'ları.

Node.js companion yok · Express server yok · localhost HTTP orchestration yok ·
browser player yok · web dashboard yok.

Ayrıntı: [`docs/architecture.md`](docs/architecture.md)

## Nereden başlamalı

| Ne öğrenmek istiyorsun | Dosya |
|---|---|
| **Şu an ne durumdayız** | [`docs/STATUS.md`](docs/STATUS.md) |
| Hangi kararlar verildi, hangileri açık | [`docs/DECISIONS.md`](docs/DECISIONS.md) |
| Ürün ne yapar, ne yapmaz | [`docs/product-spec.md`](docs/product-spec.md) |
| Sıradaki iş ne | [`tasks/INDEX.md`](tasks/INDEX.md) |
| Milestone planı ve açık sorular | [`docs/roadmap.md`](docs/roadmap.md) |
| Mimari ve port sınırları | [`docs/architecture.md`](docs/architecture.md) |
| Mimari kararlar | [`docs/adr/`](docs/adr/) |
| Log/gizlilik kuralları | [`docs/security-policy.md`](docs/security-policy.md) |
| Test yaklaşımı ve kanıt formatı | [`docs/testing-strategy.md`](docs/testing-strategy.md) |
| Terimler | [`docs/glossary.md`](docs/glossary.md) |
| Nasıl çalışılır (insan + agent) | [`CLAUDE.md`](CLAUDE.md) |

## Repo yapısı

```
docs/        roadmap, şartname, mimari, ADR'ler, milestone'lar
tasks/       task dosyaları (backlog / active / done) + üretilen INDEX.md
core/        shared core (Rust adayı — ADR-0002 ile kesinleşir)
platforms/   platforma özgü UI ve adapter'lar
fixtures/    golden test verileri
scripts/     doctor, task-index, check-docs
```

## Ön koşullar

```bash
bash scripts/doctor.sh
```

Bu script gerekli araçları kontrol eder ve eksikleri kurulum komutlarıyla
birlikte raporlar. M1 için Rust, M3 için tam Xcode ve libmpv gerekir.

## Lisans

**Henüz seçilmedi.** Kişisel kullanım + side-loading aşamasındayız; karar
ADR-0012 (libmpv linkleme) ile birlikte M3 öncesi verilecek. Gerekçe ve
bağımlılıklar: [`docs/licensing.md`](docs/licensing.md).

Depo kökünde bilerek placeholder `LICENSE` dosyası tutulmuyor — bkz. aynı dosya.
