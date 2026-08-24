---
id: NEN-029
title: Playback/renderer reverse-FFI boundary spike
milestone: M1
size: M
state: backlog
depends_on: [NEN-007, NEN-009]
blocks: [NEN-012, NEN-021]
adr: [26]
---

# NEN-029 — Playback/renderer reverse-FFI boundary spike

## Sonuç

Playback session'ın sahibinin **shared core mu platform shell mi** olacağı,
fake adapter'lar üzerinde ölçülmüş A/B karşılaştırmasıyla bilinir ve ADR-0026
ile karara bağlanabilir.

## Neden

M0'da `PlaybackEngine` ve `SubtitleRenderer` portları tasarlandı ve merkezi bir
Rust application-session'ın bu portları **reverse callback** ile yöneteceği
varsayıldı. Bu varsayım hiç doğrulanmadı — oysa M3'ün tamamı ve M7'nin position
çözünürlüğü buna bağlı. Yanlışsa, öğrenmek için en ucuz an şimdi.

## Karşılaştırılacak iki yaklaşım

- **A — Core-owned session.** Rust core, `PlaybackEngine`/`SubtitleRenderer`
  callback interface'lerini çağırır ve playback session'ın sahibidir.
  *(architecture.md'deki mevcut tercih)*
- **B — Shell-owned session.** Platform shell playback motorunun sahibidir;
  core'a coarse-grained playback state, position ve track snapshot gönderir.

Fake Swift adapter zorunlu; fake Kotlin adapter **mümkünse**. NEN-011 önce
biterse Kotlin yarısı ucuzlar, ama bu task ona bağlı değildir.

## Ölçülüp belgelenecekler

| Konu | Not |
|---|---|
| reverse callback uygulanabilirliği | A yönü teknik olarak mümkün mü |
| Swift MainActor / UI thread dönüşü | callback hangi thread'e düşüyor, maliyeti ne |
| Kotlin main thread dönüşü | aynı soru JVM tarafında |
| object ownership ve lifetime | adapter nesnesini kim tutuyor, ne kadar |
| callback sonrası object release | release doğru anda mı oluyor |
| reentrancy | callback içinden core'a çağrı güvenli mi |
| event ordering | event'ler gönderim sırasında mı geliyor |
| position update sıklığı | **iki uçta ölçülecek:** ~4 Hz (UI-yeterli) ve ~60 Hz (frame-senkron) |
| seek command / seek-complete sırası | komut ve tamamlanma sinyali karışıyor mu |
| cancellation | akış ortasında iptal — I1 ve I4 geçerli |
| shutdown | motor kapanırken sızıntı/çökme var mı |
| platform lifecycle | arka plana alma / öne getirme davranışı |
| typed error aktarımı | adapter hatası core'a typed olarak dönüyor mu |
| gereksiz yüksek frekanslı FFI trafiği | çağrı hacmi ve maliyeti |

Tüm ölçümler **baseline**'dır (pass/fail eşiği değil); bağlamıyla kaydedilir:
cihaz · OS/toolchain · debug/release build.

**Invariant'lar (pass/fail):** I1 (iptal sonrası late callback yok) ·
I4 (thread/bellek sızıntısı yok).

## YAPILMAYACAK

- Gerçek libmpv, AVPlayer veya Media3 implementasyonu — **yalnız fake adapter**
- `PlaybackEngine` / `SubtitleRenderer` portlarını kaldırmak veya yeniden
  tasarlamak — portlar her iki sonuçta da kalır
- Contract test kiti yazmak → NEN-021 (ADR-0026'dan sonra)
- Spike kodunun ürüne terfisi — `core/spikes/` altında kalır

## Kanıt (DoD)

- [ ] A ve B için ölçüm tablosu, her satır bağlamıyla birlikte
- [ ] Position update: ~4 Hz ve ~60 Hz uçlarında FFI çağrı hacmi ve maliyeti
- [ ] Event ordering ve seek/seek-complete sırası test edildi
- [ ] **I1** — iptal sonrası late callback yok
- [ ] **I4** — tekrarlı start/stop/shutdown sonrası sızıntı yok
- [ ] **M7 etkisi raporlandı:** B seçilirse manuel sync için gereken position
      çözünürlüğü ne oluyor
- [ ] ADR-0026 taslağı: önerilen yön + reddedilen yön + geri dönüş maliyeti

## Kanıt kaydı

<!-- done olurken doldurulacak -->
