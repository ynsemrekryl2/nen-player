---
id: NEN-021
title: PlaybackEngine port contract and capability model
milestone: M3
size: M
state: done
closed: 2026-08-25
depends_on: [NEN-012]
blocks: [NEN-022]
adr: [11, 26]
---

# NEN-021 — PlaybackEngine port contract and capability model

## Sonuç

`PlaybackEngine` portu, tüm adapter'ların geçmek zorunda olduğu bir contract
test kitiyle birlikte tanımlıdır; capability'si olmayan operasyon typed error
döner.

> **Ön koşul: ADR-0026 (ownership yönü) kabul edilmiş olmalı.** Port'un imzası,
> session'ın sahibinin core mu platform shell mi olduğuna göre değişir
> (NEN-029'un A/B karşılaştırması). Bu karar verilmeden contract yazmak, büyük
> olasılıkla yeniden yazılacak bir contract üretir.

## Kapsam

- Port trait'i: load · play · pause · stop · absolute/relative seek · position ·
  duration · state (buffering/ready/ended/failure) · rate · volume · audio &
  subtitle track enumeration · track selection · embedded text extraction
  capability · external subtitle injection capability · event stream · shutdown
- `Capability` kümesi ve capability sorgulama
- Contract test kiti (`nen-ports` içinde, paylaşılan)
- Fake adapter — kiti geçen referans implementasyon

## YAPILMAYACAK

- Gerçek motor bağlama → NEN-022
- Renderer → NEN-027
- Motor adının UI'a sızması — **yasak** (§4)

## Kanıt (DoD)

- [x] Fake adapter contract kitinin tamamını geçiyor
- [x] Negatif: capability'si olmayan operasyon typed error dönüyor (panik/no-op yok)
- [x] Application katmanında motor **adına** göre dallanma yok (grep kanıtı)
- [x] Event stream sıralama garantisi test edildi

## Kanıt kaydı

**ADR-0011 kabul edildi** (dört karar: bounded+coalescing teslimat · reentrancy
yasağı · capability yalnız farklılaşanı sayar · contract senaryoları veridir).
Ön koşul ADR-0026 zaten `accepted`'dı. Kod ADR onaylanmadan yazılmadı (kural 4).

Tamamı `core/crates/nen-ports/` içinde, **yeni dış bağımlılık yok**:

```
$ cargo tree -p nen-ports --edges normal
nen-ports v0.1.0
└── nen-domain v0.1.0                    → tek kenar, ADR-0006 grafiği korundu
```

Test sayısı **337 → 390** (+53): 30 unit · 6 `contract_fake` ·
6 `event_ordering` · 7 `guard_playback_debug` · 4 `guard_no_engine_names`.

### DoD #1 — fake adapter contract kitini geçiyor

Kit **23 senaryo** taşıyor; tam capability'li motora **19**'u, taban-only
motora **18**'i uygulanıyor (`applicable_count`). İkisi de sıfır failure.

```
$ cargo test -p nen-ports --test contract_fake
a_fully_capable_engine_passes_every_applicable_scenario ... ok
a_base_only_engine_passes_every_applicable_scenario ... ok
every_partial_capability_set_passes ... ok      (16 alt kümenin hepsi)
every_capability_is_covered_in_both_directions ... ok
the_run_is_not_vacuous ... ok
the_fake_declares_what_it_was_built_with ... ok
6 passed, 0 failed
```

**Koşunun boşta dönmediği ayrıca test ediliyor.** Hepsi atlanan bir kit de
"geçer" — `the_run_is_not_vacuous` senaryo sayılarına alt sınır koyuyor ve
her senaryonun en az bir uçta uygulandığını doğruluyor;
`every_capability_is_covered_in_both_directions` her capability için hem
**var** hem **yok** senaryosunun bulunduğunu şart koşuyor.

`every_partial_capability_set_passes` 4 capability'nin **16 alt kümesini de**
tek tek koşuyor — capability'lerin bağımsız olduğu (birini bildirmenin
diğerinin davranışını değiştirmediği) böyle kanıtlanıyor.

### DoD #2 — capability'si olmayan operasyon typed error dönüyor

Dört capability'nin her biri için iki senaryo: `WithoutCapability` →
`PlaybackError::Unsupported`, `WithCapability` → başarı. Panik yok, sessiz
no-op yok — `Outcome::Error(ErrorKind::Unsupported)` ile açıkça beklenen bir
dönüş değeri.

Taban (load · play · pause · stop · absolute seek · position · duration ·
state · track enumeration/selection · event stream · shutdown) capability
**değil** ve sorgulanmıyor (ADR-0011 Karar 3). Relative seek de capability
değil: `position` + `seek` üzerinden default implementasyon, sıfırda
clamp'liyor (`relative seek moves from the current position and clamps at zero`).

Lifecycle refuse'ları da tipli: `NotLoaded` (medya yokken), `ShutDown`
(kapanıştan sonra, shutdown idempotent), `UnknownTrack` (olmayan track — ve
**audio id'si subtitle id'si değil**, ayrı senaryo).

### DoD #3 — motor adına göre dallanma yok

Grep, elle bir kez değil **test olarak** koşuyor
(`tests/guard_no_engine_names.rs`): `nen-ports` ve `nen-app` altındaki tüm
`.rs` dosyalarında 8 motor adı (`mpv` · `avplayer` · `avkit` · `media3` ·
`exoplayer` · `vlc` · `gstreamer` · `avfoundation`) aranıyor, satır yorumları
çıkarılarak — port'un kendi dokümantasyonu ve ADR referansları bu adları
serbestçe anıyor, yasak olan adın **koda** girmesi.

Üç kontrol testi taramanın kendisini doğruluyor: gerçek bir ihlal
(`if engine.name() == "mpv"`) yakalanıyor · motor adı geçen prose
yakalanmıyor · satır sonu yorumu kodu gizleyemiyor.

### DoD #4 — event stream sıralama garantisi

`tests/event_ordering.rs` ADR-0026'nın ölçtüğü backgrounding şeklini
tekrarlıyor — kimse drain etmiyor, motor üretmeye devam ediyor:

- 200 ardışık seek sonrası **tek** `PositionChanged` hayatta ve değeri
  **en yenisi** (20 000 ms), ama üç seek'in **üç** `SeekCompleted`'ı da duruyor
  — coalescing kritik olayı almıyor.
- Kapasite 8'lik kuyruğa 64 kritik olay: teslimat `EventsLost { dropped }` ile
  **başlıyor**, ve `dropped + teslim edilen = 64` — hiçbir şey hesapsız
  kaybolmuyor. Bir taşma patlaması için **tek** marker.
- `EventsLost` sonrası motor resync'e cevap verebiliyor (state, position,
  duration doğru) — kayıp teslimatta, motorda değil.
- `Ended` state'i `EndReached` marker'ından **önce** geliyor: tüketici
  marker'ı gördüğünde durum zaten güncel.

Reentrancy (Karar 2) aynı akışta: `deliver_all` teslimat boyunca thread'i
işaretliyor, `guard_reentrancy` o sırada yapılan senkron çağrıyı
`ReentrantCall` ile geri çeviriyor. Contract kiti bunu **7 operasyon için
ayrı ayrı** koşuyor ve ardından motorun durumunun değişmediğini doğruluyor.
Panik ile çıkan bir callback thread'i işaretli bırakmıyor
(`a_panicking_callback_does_not_leave_the_thread_marked`), iç içe scope'lar
önceki işareti geri veriyor.

### Negatif kontrol — testler boşta dönmüyor

Üç ayrı kusur koda sokuldu, koşuldu, geri alındı:

| Kusur | Kırmızıya dönen |
|---|---|
| `FakeEngine::require` capability kontrolünü atlıyor | 2 contract testi (`a_base_only_engine…`, `every_partial_capability_set…`) |
| `is_replaced_by` kritik olayları da düşürüyor | 3 unit + 3 contract + 1 ordering = **7** |
| `guard_reentrancy` her zaman `Ok` dönüyor | 1 unit + 3 contract = **4** |

Geri alındıktan sonra 53/53 yeşil, `cargo fmt --check` ve
`cargo clippy -D warnings` temiz.

### Güvenlik (K23)

Port'tan geçen üç tip özel veri taşıyor ve üçü de elle yazılmış `Debug`
kullanıyor: `MediaSource` (medya URL / özel tam yol — K23 #1, #2, #3) yalnız
uzunluk basıyor; `TrackDescriptor` (release adı / özel dosya adı — #8) başlığı
hiç basmıyor, yerine `has_title` koyuyor; `PlaybackError`'ın **hiçbir varyantı
String taşımıyor** — NEN-010'un ölçtüğü sebeple: FFI'yı geçen bir hatayı host
dili kendi basar, Rust `Debug`'ını hiç görmez, yani yalnız Rust'ın basma
biçimi sayesinde güvenli olan bir değer güvenli değildir.

`tests/guard_playback_debug.rs` 10 yasak parçayı arıyor ve **negatif kontrolü
kalıcı**: `#[derive(Debug)]`'lı üç ikiz (media source, track descriptor, String
taşıyan hata) aynı değerlerle kurulup gerçekten sızdırdıkları doğrulanıyor —
guard bir derive hatasını yakalayacağını böyle kanıtlıyor.

### Kayda geçen tasarım kararları

1. **Coalescing eski örneği kaldırıp yenisini sona ekliyor**, yerinde
   güncellemiyor. Yerinde güncelleme, sonradan üretilmiş bir pozisyonu daha
   önce üretilmiş kritik olayların önünde gösterirdi; bu biçimde kuyruk
   üretim sırasını koruyor, yalnız bayat değer düşüyor.
2. **Coalescing kayıpları `EventsLost` sayacına girmiyor.** Ara pozisyonun
   düşmesi tasarım gereği; saymak tüketiciyi hiç olmamış bir kayıp için
   resync'e gönderirdi.
3. **`SeekCompleted` `PositionChanged` ile coalesce olmuyor.** "Benim seek'im
   yerine oturdu mu" sorusunu en yeni pozisyon cevaplayamaz.
4. **`shutdown` idempotent.** Platform shell'i kendi teardown'ının çalışıp
   çalışmadığını her zaman bilemez; ikinci çağrının hata olması gereksiz bir
   hata yolu üretirdi.
5. **Track id'leri kind başına ayrık** (fake'de subtitle 0–1, audio 2), böylece
   contract bir audio id'sinin subtitle olarak sessizce kabul edilmediğini
   kanıtlayabiliyor.
6. **`ContractInputs` kitin tek adapter-özel noktası** ve o da veri: fake bir
   medya ile gerçek bir medya farklı, ama senaryolar aynı kalıyor.
