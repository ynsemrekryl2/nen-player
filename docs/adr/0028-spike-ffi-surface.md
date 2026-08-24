---
adr: 0028
title: Spike'ların geçici FFI yüzeyi
status: accepted
milestone: M1
tasks: [NEN-008]
date: 2026-08-24
---

# ADR-0028 — Spike'ların geçici FFI yüzeyi

## Durum

`accepted`

## Bağlam

M1'in beş ölçümünden dördü (`NEN-008` büyük cue listesi, `NEN-009` async +
cancellation, `NEN-010` typed error, `NEN-029` reverse-FFI) **FFI sınırının
kendisini** ölçüyor. Ölçülen şey sınırın maliyeti olduğu için, bu spike'ların
Swift (ve `NEN-011`'de Kotlin) tarafından çağrılabilen bir yüzeye ihtiyacı var.

Bugünkü üç kural bu yüzeye yer bırakmıyor:

| Kaynak | Kural |
|---|---|
| ADR-0006, kural 2 | "`nen-ffi` tek dış kapıdır. **Başka hiçbir crate** FFI yüzeyi (`extern "C"`, binding makrosu, üretilen header) açmaz." |
| ADR-0006, kural 3 | "`core/spikes/` ürün kodu değildir. … ok yönü yalnız spikes → crates." |
| CLAUDE.md, kural 7 | "Spike kodu ürün kodu değildir. `core/spikes/` altında kalır, terfi etmez." |

Kural 2 spike'a kendi kapısını yasaklıyor; kural 3 `nen-ffi`'ın bir spike
crate'ine bağımlı olmasını yasaklıyor; kural 7 spike kodunun `nen-ffi`'ın içine
yazılmasını yasaklıyor. Üçü birlikte, **M1'in var oluş sebebi olan ölçümü
imkânsız kılıyor.**

Karar verilmezse iki kötü sonuçtan biri gerçekleşir: ya bir kural sessizce
çiğnenir ve altı ay sonra kimse `nen-ffi`'daki `spike` modülünün neden orada
olduğunu bilemez, ya da M1 ölçülemez ve ADR-0002 sayısız verilir.

ADR-0006 bu durumu öngörmemiştir; kapsamı ürün kodunun katman grafiğiydi.
Bu ADR onu **değiştirmez**, sınırını netleştirir.

## Karar

`core/spikes/*` altındaki bir spike crate, **yalnız ölçüm amacıyla ve yalnız
kendi içinde**, kendi FFI scaffolding'ini açabilir; kendi `staticlib`/`cdylib`
çıktısını ve kendi binding'ini üretebilir.

ADR-0006 kural 2 — "`nen-ffi` tek dış kapıdır" — **ürün kodu için** (`core/crates/*`
ve `platforms/*`'ın linklediği her şey) değişmeden bağlayıcı kalır.

Spike kapısına dört sınır konur:

1. **Ürün paketine linklenmez.** `platforms/apple-shared/Package.swift` ve
   ileride Android/Windows/Linux ürün paketleri spike binding'ini referans
   etmez. Spike'ın kendi harness paketi ayrıdır ve `core/spikes/` altındadır.
2. **Ok yönü korunur.** Hiçbir `core/crates/*` crate'i bir spike crate'ine
   bağımlı olamaz — ADR-0006 kural 3 aynen geçerlidir.
3. **Üretilen binding commit edilmez.** `.gitignore` → `/core/spikes/**/generated/`,
   `/platforms/**/generated/` kuralının karşılığı.
4. **Terfi yok.** Spike API'si ürün API'sinin taslağı değildir; ölçümün
   sonucu koda değil, ADR'ye gider (CLAUDE.md kural 7).

Bu sınırların üçü **mekanik olarak** doğrulanır ve çıktısı ilgili task'ın kanıt
kaydına girer:

```bash
# Ürün kodunda nen-ffi dışında FFI yüzeyi yok  → çıktı boş olmalı
grep -rn --include='*.rs' -E "uniffi::|extern \"C\"" core/crates | grep -v crates/nen-ffi/

# Hiçbir ürün crate'i spike'a bağımlı değil    → çıktı boş olmalı
cd core && cargo metadata --format-version 1 --no-deps   # crates/* → spikes/* kenarı yok

# Ürün Swift paketi spike binding'ini tanımıyor → çıktı boş olmalı
grep -rn "spike" platforms/apple-shared/Package.swift
```

## Gerekçe

**Kural 2'nin koruduğu şey ne?** ADR-0006'nın kendi gerekçesi açık: tek kapı,
`docs/security-policy.md` K23 redaction'ının ve `NEN-006` guard testinin **tek
bir yüzeyi** koruyabilmesi içindir. Guard testi ürünün log çıktısını korur.
Ürüne linklenmeyen, dağıtılmayan, ölçüm bitince silinebilir bir spike binary'si
bu yüzeyi genişletmez — kullanıcının cihazına hiç ulaşmaz.

Yani kural 2'nin **amacı** ihlal edilmiyor; lafzı, öngörülmemiş bir duruma
uygulanıyordu. Bu ADR lafzı amacına eşitliyor.

**Neden spike'ın kendi kapısı, `nen-ffi` üzerinden değil?** Ölçümün kendisi
temiz kalıyor: spike'ın `Vec<SpikeCue>` geçişi ürün tiplerinden, ürün
feature bayraklarından ve ürün bağımlılıklarından etkilenmiyor. `nen-ffi`
içinden ölçseydik, ölçülen sayı "FFI'ın maliyeti" değil "bugünkü `nen-ffi`'ın
maliyeti" olurdu — ve `nen-ffi` M2'de dolmaya başladığında ölçüm
tekrarlanamaz hale gelirdi.

**Neden ADR-0006'yı düzenlemek yerine yeni ADR?** ADR-0001: "Kabul edilmiş bir
ADR düzenlenmez." ADR-0006 kabul edildi ve gövdesi olduğu gibi kalır; yalnız
"Notlar" bölümüne bu ADR'ye işaret eklenir — ADR-0001'in açıkça izin verdiği
istisna.

**Neden 0028?** `docs/adr/README.md` → "Planlanan" tablosu 0002–0027'yi konu
konu haritalıyor (0002 core dili, 0003 binding, 0007 subtitle domain…). Numara
rezerve değil ama harita kullanışlı; ilk boş numara alınıyor, harita
kaydırılmıyor.

**Varsayım:** spike sayısının az ve ömrünün kısa kalacağı. M1'de dört spike
öngörülüyor. Bu sayı büyürse veya bir spike M1 sonrasında yaşamaya devam
ederse, varsayım bozulmuş demektir ve bu ADR yeniden değerlendirilir.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Ölçümü `nen-ffi` içine `spike` (default olmayan) feature'ı ardında koymak | Spike kodu ürün crate'ine girer — CLAUDE.md kural 7'nin tam olarak yasakladığı şey. Ayrıca ölçüm `nen-ffi`'ın o günkü bağımlılık ve tip yüküne bulaşır; M2'de `nen-ffi` dolduğunda aynı ölçüm tekrarlanamaz |
| `nen-ffi`'a spike crate'ine optional dependency eklemek | ADR-0006 kural 3'ün ters oku (`crates → spikes`). Feature kapalıyken bile `Cargo.toml`'da duran kenar, "ok tek yönlüdür" iddiasını `cargo metadata` ile kanıtlanamaz hale getirir |
| Spike'ları ayrı bir workspace'e almak | ADR-0006'da zaten reddedildi: ikinci `Cargo.lock` ve path bağımlılığı doğurur. Ayrıca sorunu çözmez — ayrı workspace'teki crate de "başka bir crate"tir |
| Ölçümü FFI'sız yapmak (yalnız Rust tarafında `criterion`) | NEN-008'in ölçtüğü şey **sınırın** maliyeti: marshalling, kopya, Swift tarafındaki bellek. Rust içi benchmark bu sayıların hiçbirini vermez; DoD'daki "UI thread bloklanma süresi" ölçülemez |
| Kuralı çiğneyip ADR yazmamak | ADR-0001: "altı ay sonra biri 'bu neden böyle?' diye sorar mı?" — `core/spikes/` altında ikinci bir `uniffi::setup_scaffolding!()` gören herkes sorar |

## Sonuçlar

**Olumlu:** M1'in dört ölçüm task'ı da (`NEN-008`, `NEN-009`, `NEN-010`,
`NEN-029`) kural ihlal etmeden koşabilir. Ölçümler ürün kodundan izole olduğu
için tekrarlanabilir kalır — aynı spike crate'i M2'de yeniden koşulup sayılar
karşılaştırılabilir. `NEN-011` aynı spike crate'inden Kotlin binding üreterek
pariteyi ölçer; parite ölçümü ürün koduna hiç dokunmaz.

**Olumsuz / kabul edilen maliyet:** depoda birden fazla FFI scaffolding bulunur;
"tek kapı" iddiası artık koşulsuz değil, `core/crates/*` ile sınırlı. Bu, her
okuyanın akılda tutması gereken bir istisnadır — maliyeti yukarıdaki üç grep'in
ilgili task'ların kanıt kaydında koşulmasıyla ödenir. Ayrıca spike binding
üretimi için ürününkine paralel ikinci bir script (`scripts/spike-cues.sh`)
bakım yüzeyi ekler.

**Geri dönüş maliyeti:** **ucuz.** Spike'lar terfi etmediği için karardan dönmek
`core/spikes/` altındaki dizinleri silmekten ibarettir; ürün kodunda hiçbir iz
bırakmaz. Karar M1 ile birlikte doğal olarak ölür.

## İlgili task'lar

`NEN-008` · sonraki kullanıcıları: `NEN-009` · `NEN-010` · `NEN-011` · `NEN-029`

## Notlar

<!-- Karar sonrası gözlemler -->
