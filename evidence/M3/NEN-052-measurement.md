# NEN-052 — Yükleme penceresi ölçümü

Gerçek libmpv adapter'ı (`MPVPlaybackEngine`), `fixtures/media/contract-clip.mkv`
üzerinde, beş ayrı `load()` çağrısı ile ölçüldü. Ölçüm kodu geçiciydi
(`ZZNEN052MeasurementTests.swift`, bu dosyayla birlikte silindi) — üründe
kalmadı.

## Yöntem

Her denemede: `load()` çağrısı hemen ardından `state()` okundu, sonra sırasıyla
`seek(toMs:)`, `positionMs()`, `durationMs()` çağrıldı (hiçbiri beklemeden), son
olarak `state()` gerçek zamanlı (1 ms'lik `Thread.sleep` ile, meşgul döngü değil)
`.ready` veya `.failed` olana kadar izlendi.

## Sonuç

```
NEN052[1]: right-after-load state=buffering seek=EngineFailure(code: -12) position=EngineFailure(code: -10) duration=OK(nil ok) settledTo=ready after 12.187 ms
NEN052[2]: right-after-load state=buffering seek=EngineFailure(code: -12) position=EngineFailure(code: -10) duration=OK(nil ok) settledTo=ready after 2.695791 ms
NEN052[3]: right-after-load state=buffering seek=EngineFailure(code: -12) position=EngineFailure(code: -10) duration=OK(nil ok) settledTo=ready after 2.561583 ms
NEN052[4]: right-after-load state=buffering seek=EngineFailure(code: -12) position=EngineFailure(code: -10) duration=OK(nil ok) settledTo=ready after 2.64225 ms
NEN052[5]: right-after-load state=buffering seek=EngineFailure(code: -12) position=EngineFailure(code: -10) duration=OK(nil ok) settledTo=ready after 2.645041 ms
```

**Bulgular:**

1. `load()` döndükten hemen sonra `state()` gerçekten `.buffering` — beş
   denemenin beşinde de.
2. Pencere **2.5–12 ms** arası sürüyor (`FILE_LOADED`'a kadar) — bir insanın
   tuşa basıp motoru yakalaması için fazlasıyla dar, dolayısıyla bu durum
   gerçek kullanımda **her zaman** oluşur, nadiren değil.
3. Bu pencerede `seek(toMs:)` mpv'den `MPV_ERROR_COMMAND` (`-12`) alıyor ve
   `check()` bunu `EngineFailure(code: -12)`'ye çeviriyor — task dosyasının
   öngördüğü tam olarak bu.
4. Aynı pencerede `positionMs()` da `EngineFailure(code: -10)`
   (`MPV_ERROR_PROPERTY_UNAVAILABLE`) fırlatıyor — `0` **dönmüyor**.
   `positionMs()`'in `try double("time-pos")` çağrısı `try?` değil `try`,
   dolayısıyla `time-pos`'un o an okunamaz olması motor hatası olarak dışarı
   sızıyor. Bu, göreli seek kararını etkiliyor (bkz. ADR-0042 Karar 5): eğer
   birisi `.loading` fazında `position()` + `seek()` bileşimini (Rust'ın
   varsayılan `seek_relative`'i) çağırsaydı, `seek()`'e hiç sıra gelmeden
   `position()` adımında patlardı.
5. `durationMs()` motor hatası **fırlatmıyor** — `nil` dönüyor, çünkü
   `durationMs()` `try?` kullanıyor (`guard let seconds = try? double("duration")`).
   Bu zaten "canlı yayının süresi yok" ile aynı, kabul edilebilir bir cevap.

## Kapsam etkisi

Bu görev yalnız **seek**'i erteliyor (task'ın DoD'u da bunu istiyor).
`position()`'ın yükleme sırasında motor hatası fırlatması **önceden var olan**,
bu task'ın kapsamına girmeyen bir davranış — dokunulmadı. Gerçek kullanıcı
yüzeyinde göreli seek (`⌘←`/`⌘→`) zaten `PlayerModel.seekRelative` üzerinden
şu anki pozisyonu **kendi tuttuğu** `displayedPositionMilliseconds`'tan alıp
mutlak `seek(to:)`'a gidiyor — Rust'ın port-seviyesi varsayılan
`seek_relative`'ini (which calls `position()` then `seek()`) hiç çağırmıyor.
Yani gerçek kullanıcı gördüğü göreli seek zaten bu task'ın düzelttiği mutlak
seek yolundan geçiyor.
