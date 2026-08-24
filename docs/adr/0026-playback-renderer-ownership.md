---
adr: 0026
title: Playback/renderer ownership yönü
status: accepted
milestone: M1
tasks: [NEN-029]
date: 2026-08-24
---

# ADR-0026 — Playback/renderer ownership yönü

## Durum

`accepted`

## Bağlam

`docs/architecture.md`'nin "Ownership yönü — spike bekliyor" bölümü (M0'dan
beri) merkezi bir Rust application-session'ın `PlaybackEngine` ve
`SubtitleRenderer` portlarını **reverse callback** ile yöneteceğini
varsayıyordu — ama bu varsayım hiç ölçülmemişti. `NEN-021` (port contract)
bu ADR kabul edilmeden başlamıyor; M3'ün tamamı ve M7'nin manuel sync
çözünürlüğü buna bağlı.

Karşılaştırılan iki yön:

- **A — Core-owned session.** Rust core, `PlaybackEngine`/`SubtitleRenderer`
  callback interface'lerini çağırır ve playback session'ın sahibidir.
  (`docs/architecture.md`'deki mevcut tercih.)
- **B — Shell-owned session.** Platform shell playback motorunun sahibidir;
  core'a coarse-grained playback state, position ve track snapshot gönderir.

`NEN-029` bunu fake Swift adapter'lar üzerinde ölçtü —
`core/spikes/spike-reverse-ffi/` (ADR-0028'in izin verdiği throwaway FFI
kapısıyla), gerçek libmpv/AVPlayer/Media3'e hiç dokunmadan. Ölçülenler ve tam
tablolar `NEN-029`'un kanıt kaydında; bu ADR yalnız kararı ve gerekçesini
taşır.

**Ölçülen bulgular (özet):**

| Konu | Bulgu |
|---|---|
| Reverse callback uygulanabilirliği | Evet — `#[uniffi::export(foreign)]` trait, NEN-009'un delivery-gate deseniyle stabil |
| Per-call maliyet, 60 Hz, release | A: p50 36.67 µs · B: p50 1.33 µs |
| MainActor hop maliyeti (yalnız A) | p50 ~40-58 µs, callback her zaman ana thread DIŞINDA düşüyor |
| 60 Hz'de toplam FFI payı | A ~4.6 ms/sn · B ~0.08 ms/sn — ikisi de 1000 ms/sn bütçenin binde biri mertebesinde |
| I1 (iptal sonrası late callback yok) | Sağlandı (yalnız A'da geçerli — B'de ters çağrı yok) |
| I4 (sızıntı yok) | Sağlandı, her iki yönde |
| Event ordering / seek-complete sırası | Sağlandı, her iki yönde |
| Typed error aktarımı | Sağlandı, her iki yönde |
| Reentrancy | callback içinden `seek()` güvenli; callback içinden `cancel()` **aynı thread'den** çağrılırsa gate'in kendi kilidiyle self-deadlock (kod incelemesiyle kanıtlandı, çalıştırılmadı — bkz. kanıt kaydı) |
| Backgrounding (yalnız A, proxy deney) | Rust'ın üretici thread'i tüketici (Swift) hazır olmasa da üretmeye devam ediyor — birikme B'de yapısal olarak yok |

## Karar

**Yön A (core-owned session, reverse callback) benimsenecektir.**
`PlaybackEngine`/`SubtitleRenderer` portları değişmeden kalır; `NEN-021`
port contract'ını A'nın reverse-callback modeli üzerine kurar.

`NEN-021`'in gerçek kontratı iki ölçülmüş riski açıkça ele almak
**zorundadır** — bunlar bu ADR'yi geçersiz kılmaz, ama tasarım kısıtıdır:

1. **Backpressure/backgrounding.** Callback teslimatı, üreticiyi (Rust
   tarafı) tüketicinin (platform shell) hazır olup olmadığından bağımsız
   tutmalı — sınırsız birikme yerine sınırlı/coalescing bir kuyruk ya da
   demand-driven (ack tabanlı) bir teslimat deseni gerekir.
2. **Reentrancy disiplini.** Bir callback içinden, delivery gate'in kilidini
   isteyen bir core metodu (`cancel()` benzeri) **aynı thread'den senkron**
   çağrılamaz — kontrat bunu ya açıkça yasaklamalı ya da çağrıyı zorunlu
   olarak asenkron/başka-thread'e dispatch etmelidir.

## Gerekçe

A'nın mutlak maliyeti (60 Hz'de bile ~5 ms/sn mertebesinde, 1000 ms/sn'lik
kare bütçesinin binde biri) hiçbir makul UI bütçesini zorlamıyor — B'nin
~60× daha ucuz olması pratikte kararı değiştirecek büyüklükte değil.
Merkezi Rust session'ın asıl gerekçesi zaten performans değildi: subtitle
sync, çeviri tetikleme gibi mantığın **tek, platformdan bağımsız bir
kaynaktan** yönetilmesi — bu spike o gerekçeyi ölçmedi (ölçemezdi), yalnız
A'nın **uygulanabilir ve makul maliyetli** olduğunu doğruladı. Ölçülen hiçbir
invariant ihlali yok (I1, I4 her iki yönde de sağlandı), M1'in "no-go"
kriterlerinden hiçbiri tetiklenmedi.

A'nın gerçek dezavantajları (backpressure riski, reentrancy hassasiyeti)
performans değil **disiplin** meseleleri — NEN-021'in kontratına açık kural
olarak yazılabilir, mimariyi terk etmeyi gerektirmez.

## Reddedilen alternatifler

| Alternatif | Neden reddedildi |
|---|---|
| **B — Shell-owned session.** Daha ucuz (60 Hz'de ~60× daha az FFI maliyeti) ve yapısal olarak daha basit (backpressure riski yok, reentrancy hassasiyeti yok, thread-hop yok). | A'nın mutlak maliyeti zaten ihmal edilebilir olduğundan bu avantaj kararı değiştirecek büyüklükte değil. B seçilirse subtitle sync/çeviri tetikleme gibi mantığın platforma özgü coarse-grained snapshot'lardan yeniden inşa edilmesi gerekir — bu, merkezi Rust session'ın asıl gerekçesini (tek kaynak) zayıflatır ve bu spike'ın kapsamı dışında, ölçülmemiş bir maliyettir. |
| Melez model (kritik olaylar reverse callback, yüksek frekanslı position B gibi forward-push) | Spike'ın kapsamı dışı — `NEN-029`'un görevi A/B karşılaştırması, üçüncü bir tasarımın icadı değil. Ölçülen maliyetler (her iki yönde de küçük) böyle bir karmaşıklığı haklı çıkarmıyor; gerekirse ayrı bir ADR ile ele alınır. |

## Sonuçlar

**Olumlu:** `NEN-021` net bir temel üzerine port contract'ını yazabilir;
`docs/architecture.md`'nin "spike bekliyor" notu kalkabilir (bu ADR kabul
edildikten sonra, ayrı bir düzenlemeyle — `NEN-029` kendisi o dosyayı
düzenlemedi).

**Olumsuz / kabul edilen maliyet:** A'nın backpressure ve reentrancy
risklerini gerçek kontratta (NEN-021) disiplinli biçimde ele almak ek
tasarım yükü. B'nin daha basit/ucuz alternatifi bilerek terk ediliyor.

**Geri dönüş maliyeti: orta.** `PlaybackEngine`/`SubtitleRenderer` portları
her iki yönde de aynı kaldığından port sınırları değişmez; ancak session
ownership'i B'ye çevirmek `NEN-021`'in kontratını (callback teslimat modeli,
backpressure tasarımı) ve muhtemelen M3'teki adapter implementasyonlarını
yeniden yazmayı gerektirir. Spike kodu terfi etmediği için (`core/spikes/`
altında kalır) bu kararın kendisi ucuz geri alınır — pahalı olan üstüne
inşa edilecek NEN-021/M3 kodudur.

## İlgili task'lar

`NEN-029` · sonraki kullanıcısı: `NEN-021`

## Notlar

<!-- Karar sonrası gözlemler -->
