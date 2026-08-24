---
adr: 0027
title: FFI performans bütçesi (M1 baseline'larından)
status: accepted
milestone: M1
tasks: [NEN-008, NEN-009, NEN-029, NEN-012]
date: 2026-08-24
---

# ADR-0027 — FFI performans bütçesi

## Durum

`accepted`

## Bağlam

`M1-core-spike.md`: "M1'in işi eşik doğrulamak değil, **bütçeyi öğrenmek**.
[...] Performans bütçesi baseline ve gerçek kullanıcı deneyimi
değerlendirildikten sonra ADR-0027 ile kabul edilir." `docs/DECISIONS.md`
zaten şunu kayıt altına almıştı: "Ölçülmemiş performans eşikleri baseline'a
çevrildi — `< 100 MB`, `< 250 ms` hiçbir ölçüme dayanmıyordu."

NEN-008/009/029 üç ayrı FFI yüzeyini ölçtü: büyük veri erişimi (cue listesi),
async/cancellation gecikmesi, ve yüksek frekanslı reverse-callback trafiği.
Bu ADR o üç ölçümden **M2 ve sonrası tasarımların uyacağı bir bütçe**
önerir — sıkı bir alt/üst sınır değil, "bu aralığın dışına çıkarsan
tasarımını gözden geçir" işareti.

## Karar

Aşağıdaki üç bütçe M2+ tasarımları için **öneri** olarak kabul edilecektir.
Sayılar ölçülen p50/p95'in üzerine kasıtlı marj eklenerek seçildi (cihaz
çeşitliliği, debug build, gerçek veri varyasyonu için); geçilmesi "kırık"
anlamına gelmez, tasarımın yeniden gözden geçirilmesi gerektiği anlamına
gelir.

| Bütçe | Değer | Ölçülen baseline | Marj |
|---|---|---|---|
| Cue listesi FFI erişimi (pencereli, ~40 cue) | **≤ 500 µs** | p50 44.3 µs (release) | ~11× |
| Cue listesi FFI erişimi (tam liste, 50k cue) | **≤ 150 ms**, ve **main-thread'i tek seferde bloke etmemeli** (chunked/async çağrı) | p50 37.7 ms, en uzun blok 48.4 ms (release) | ~4× |
| Cancellation gecikmesi (kullanıcının hissettiği "iptal" ile son commit arası) | **≤ 10 ms** | checkpoint_every=100 → p50 2.02 ms (release) | ~5× |
| Reverse-FFI callback maliyeti, 60 Hz (core-owned yön, ADR-0026) | **≤ 500 µs per-call**, toplam **≤ 5% of frame budget** (16.7 ms @ 60 fps) | p50 36.67 µs + hop ~40 µs ≈ 76.67 µs/call (~0.46% frame budget) | ~6.5× |

**Kapsam dışı:** typed-error eşleme maliyeti (NEN-010/011) bütçeye
alınmadı — p50 2.04 µs (Swift) / Kotlin'in en kötü ölçümü dahi düşük
mikrosaniye mertebesinde, hiçbir tasarımı kısıtlamıyor.

## Gerekçe

Marjlar, üç kaynaktan geliyor: (1) tüm ölçümler tek bir cihazda (Apple M5)
yapıldı — gerçek kullanıcı donanımı daha yavaş olabilir; (2) release/debug
farkı NEN-008'de tam liste için 1.75×, pencereli erişimde 1.24× ölçüldü —
geliştirme sırasında debug build'in bütçeyi zorlamaması için pay bırakıldı;
(3) fixture'lar (50k cue, 3.1 MiB) M1'in seçtiği tek boyuttu — gerçek
kullanıcı verisi daha büyük olabilir.

Tam liste bütçesinin "main-thread'i tek seferde bloke etmemeli" koşulu
NEN-008'in kendi bulgusundan geliyor: 48.4 ms'lik tek blok ~3 kare
@60fps'e denk geliyor — kabul edilebilir değil, bu yüzden bütçe hem bir
süre sınırı hem bir **tasarım kısıtı** (chunking/async) içeriyor.

Reverse-FFI bütçesi `ADR-0026`'nın seçtiği yön A'ya göre yazıldı; yön B
seçilseydi bütçe gereksiz olurdu (B'nin maliyeti zaten ihmal edilebilir).

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| Ölçülen p50/p95'i doğrudan bütçe olarak kullanmak (marjsız) | Tek cihaz, tek fixture boyutu, yalnız release build ölçüldü — marjsız bütçe ilk farklı cihazda veya debug build'de kırılırdı, bu da M1-core-spike.md'nin "eşik değil baseline" felsefesine aykırı olurdu. |
| Bütçeyi M1'de hiç belirlememek, her task'ı kendi baseline'ıyla bırakmak | `docs/DECISIONS.md` zaten "performans bütçeleri M1 sonrası ADR-0027 ile" kararını kaydetmişti — bütçesiz bırakmak M2+ task'larının kendi keyfi eşiklerini icat etmesine yol açardı (tam olarak `< 100 MB`/`< 250 ms` sorununun tekrarı). |

## Sonuçlar

**Olumlu:** M2+ task'ları (özellikle `NEN-017` indexed cue lookup ve
`NEN-021` playback port contract) somut bir bütçeye karşı tasarlanabilir.

**Olumsuz / kabul edilen maliyet:** Bütçeler tek cihazda (Apple M5) ve tek
fixture boyutunda ölçülen sayılardan türetildi; gerçek kullanıcı
donanımı/verisiyle marjın yetersiz çıkma riski var — bu durumda bütçe
`superseded` bir ADR ile güncellenir.

**Geri dönüş maliyeti: ucuz.** Bu bir performans hedefi, mimari bir
kısıt değil; yeni ölçümle güncellenmesi yalnız bu ADR'yi `superseded`
yapıp yenisini yazmayı gerektirir, kod değişikliği zorunlu kılmaz.

## İlgili task'lar

`NEN-008` · `NEN-009` · `NEN-029` · `NEN-012`

## Notlar

<!-- Karar sonrası gözlemler -->
