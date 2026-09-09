---
id: NEN-096
title: Content-addressed artifact store with atomic commit
milestone: M5
size: M
state: done
closed: 2026-09-09
depends_on: [NEN-095]
blocks: [NEN-098]
adr: [0017]
---

# NEN-096 — Content-addressed artifact store with atomic commit

## Sonuç

Bir artifact'in içeriği içeriğinden türeyen bir adresle ve **atomik** olarak
yazılıyor; yarım yazım hiçbir koşulda okunabilir olmuyor.

## Bağlam

Şartname §11: "Final artifact: yalnız validation sonrası · **atomik commit** ·
restart sonrası reuse". Atomiklik burada teorik bir incelik değil: yarım yazılmış
bir WebVTT dosyası, M5'in "yarım/progressive çıktı yayınlanmaz" kriterini
doğrudan ihlal eder.

`nen-persist` bugün üç satırlık boş bir iskelet.

## Kapsam

- İçerik adresli yazma ve okuma (ADR-0017'nin kararlaştırdığı biçimde)
- Atomik commit: geçici yazım + yerine koyma; kısmi dosya görünür olmaz
- Aynı içeriğin ikinci kez yazılmasının yeni kopya üretmemesi
- Depo kökünün dışarıdan (platformdan) enjekte edilmesi

## YAPILMAYACAK

- Metadata index ve sorgulama — `NEN-098`
- Cache identity hesabı — `NEN-097`
- Kullanıcıya "cache'i temizle" komutu — M6/UI işi, burada yalnız API sınırı
- Şifreleme — kapsam dışı; artifact kullanıcının kendi diskinde

## Kanıt (DoD)

- [x] Unit: yazılan artifact aynı adresten birebir okunuyor
- [x] Unit: aynı içerik iki kez yazıldığında tek kopya kalıyor
- [x] Negatif: yazım ortasında kesilen bir commit sonrası depoda **okunabilir
      hiçbir kısmi artifact yok** (kasıtlı olarak kesilen yazımla ölçülür)
- [x] Negatif: atomik yerine koyma düz yazımla değiştirildiğinde yukarıdaki test
      kırmızıya dönüyor — kontrol sağır değil
- [x] Negatif: depo kökünün dışına çıkmaya çalışan bir adres reddediliyor
- [x] Guard: depo yolu ve dosya adı hiçbir log yüzeyine düşmüyor (K23 #3)

## Kanıt kaydı

### Ne yapıldı

**Port (`nen-ports::persistence`, yeni).** ADR-0006 `nen-persist`'i
`domain + ports` ile sınırlıyor — `ValidatedSubtitleArtifact`'i (nen-translate)
ve fingerprint tiplerini (nen-subtitle) göremez. Bu yüzden depoya giren şekil
portta tanımlandı: `ContentAddress`, `ArtifactRecord`, `ArtifactStore`,
`ArtifactStoreError`, `MAX_ARTIFACT_BYTES` ve `persistence::contract` kiti
(`docs/architecture.md`'nin zaten söz verdiği kit).

**İzdüşüm tek yönlü.** `ValidatedSubtitleArtifact::to_record()` eklendi; tersi
**bilerek yok**. Okuma bir `ArtifactRecord` üretir, hiçbir zaman bir
`ValidatedSubtitleArtifact` — `NEN-094`'ün "yalnız `assemble` doğrulanmış
artifact üretir" değişmezi tip düzeyinde bozulmadan kalıyor, diskten gelen
baytlar doğrulanmış bir çeviri gibi davranamıyor. `ArtifactId` de kayda
girmiyor: depodaki kimlik içerik adresidir, dışarıdan verilen bir id içeriğe
girseydi aynı çeviri iki id altında iki adres alırdı.

**Adapter (`nen-persist`).** `FilesystemArtifactStore`, düz yerleşimle
`<root>/artifacts/<64hex>.json`. Yazım ADR-0017 Karar 3'ün tam dizisi: aynı
dizinde benzersiz geçici ad → `fsync` → `rename` → dizin `fsync`; her hata
yolunda geçici dosya siliniyor. Okuma `symlink_metadata` ile düzenli dosya
şartı, `MAX_ARTIFACT_BYTES` sınırı ve **içerik hash'inin adrese eşitliği**
kontrolünden geçiyor. Serileştirme `wire` modülünde izole: `serde` derive'ları
yalnız orada, domain tipleri serde'den habersiz, geri dönüş her alanda o tipin
kendi public constructor'ından geçiyor.

**Kök dışına çıkma iki katmanda kapalı** (ADR-0017 Karar 4): (1)
`ContentAddress::from_hex` yalnız tam 64 küçük-harf hex kabul ediyor, yani
`..`, `/`, yüzde kaçışı veya büyük harf bir adres olamıyor — bir yola hiç
ulaşamıyor; (2) `resolve()` önce sözdizimsel olarak tek bir normal bileşen
şartı koyuyor, sonra hedefin dizinini `canonicalize` edip kökün altında
kaldığını doğruluyor — sembolik bağla kaçırılmış bir `artifacts` dizinini
yalnız bu ikinci katman yakalıyor.

### Sayılar

- `cargo test -p nen-persist`: **21 passed** (16 unit + 5 guard)
- `cargo test -p nen-ports`: lib **61 passed** (4'ü yeni), tüm hedefler yeşil
- `cargo test -p nen-translate`: lib **42 passed** (2'si yeni), tüm hedefler yeşil
- `cargo test -p nen-app --test artifact_store_roundtrip`: **3 passed**
- `cargo test --workspace`: **768 passed / 1 ignored**, 0 failed
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo deny check` (advisories/bans/licenses/sources ok) — hepsi yeşil
- `bash scripts/test.sh` **4/4**, `bash scripts/task-index.sh --check` çıkış 0,
  `bash scripts/check-docs.sh` çıkış 0

### Yeni dış bağımlılık getirilmedi (ADR-0017 Karar 2)

`serde` workspace dep'i olarak beyan edildi, ama `Cargo.lock` diff'i **tek bir
yeni `[[package]]` göstermiyor** — yalnız `nen-persist`'in kendi bağımlılık
listesine `blake3`, `serde`, `serde_json` eklendi. `serde` 1.0.229 ve
`serde_derive` grafikte `serde_json` ve `uniffi` üzerinden zaten duruyordu;
`cargo deny` yüzeyi ve `NEN-043`'ün bundle kapanışı büyümedi.

### Mutasyon ölçümleri — her kapı ayrı ayrı, sağırlık yok

Her kapı elle kaldırıldı, ölçüldü, geri alındı. **Onunda da tam olarak bir
test kırmızıya döndü:**

| Kaldırılan kapı | Kırmızıya dönen |
|---|---|
| Atomik commit (temp+`rename` → hedefe düz yazım) | `an_interrupted_commit_leaves_no_readable_artifact` (11 passed / 1 failed) |
| `resolve()` sözdizimsel tek-bileşen kapısı | `a_name_that_leaves_the_root_is_refused` |
| `new()` kanonik containment kontrolü | `a_root_whose_artifacts_directory_escapes_is_refused` |
| `get()` içerik-hash doğrulaması | `bytes_that_do_not_match_their_address_are_refused` |
| Düzenli-dosya / symlink reddi | `a_symlink_at_an_address_is_not_read_through` |
| `put()` dedup erken dönüşü | `the_same_content_written_twice_keeps_one_copy` |
| `wire` format sürüm kapısı | `an_unknown_format_version_is_refused` |
| `ContentAddress::from_hex` uzunluk kapısı | `only_exactly_64_lowercase_hex_characters_parse` |
| `ContentAddress` `Debug`/`Display` → gerçek hex | `no_content_address_surface_prints_the_file_name` |
| `ArtifactRecord` `Debug` → `#[derive(Debug)]` | `no_record_debug_output_leaks_dialogue_or_a_fingerprint` |
| `FilesystemArtifactStore::Debug` → kökü basıyor | `no_store_debug_output_prints_the_store_root` |

Atomiklik kontrolü ayrıca **kalıcı** bir karşı-kontrol taşıyor:
`a_plain_write_leaves_a_readable_partial_artifact`, aynı kesilen yazımı naif
biçimde (hedefe doğrudan) yaparak, bir önceki testin yasakladığı şeyin gerçekten
oluştuğunu — adreste görünür ve çözülemeyen bir kısmi dosya — her koşuda
gösteriyor. Derive'lı `Debug` mutasyonu sentinel diyaloğu (`Zzqxvunlogged`) ve
ham digest'i (`171, 171, …`) aynen bastı; guard sağır değil.

**Dedup kapısında bir düzeltme yapıldı.** İlk yazımda `the_same_content_written_
twice_keeps_one_copy` yalnız dosya *sayısını* sayıyordu, ve dedup erken dönüşü
kaldırıldığında **yeşil kalıyordu** — çünkü atomik `rename` yeniden yazımı
zaten idempotent yapıyor, ikinci yazım aynı adrese birebir aynı dosyayı koyuyor.
Ölçülen bu sonuç üzerine test, dosyanın **aynı dosya** kaldığını (inode
değişmemiş) iddia edecek şekilde güçlendirildi; kapı artık gerçekten bağlayıcı.

### Uçtan uca

`nen-app/tests/artifact_store_roundtrip.rs`, gerçek `MockTranslationProvider`
ile bir çeviri koşusu → `assemble` → `to_record` → `put` → `get` zincirini
sürüyor ve cue ID/sıra/zamanların baştan sona kaynak belgeyle birebir kaldığını
doğruluyor. Çok bloklu (95 cue) hâli ayrı bir testte. Bu test `nen-app`'te
duruyor çünkü iki yarıyı birden gören tek crate orası — ADR-0006 sınırı
bozulmadı.

### Yol üstünde bulunan bağımsız kusur (Kural 5) — `NEN-106`

Uçtan uca test yazılırken ölçüldü: `translate_checkpointed` bütün bloklar için
tek bir `TranslationCall` kullanıyor, `TranslationCall::progress` ise aynı çağrı
içinde `total`'ın değişmesini haklı olarak `Permanent` ile reddediyor. Ama bir
sağlayıcı yalnız kendi bloğunu görür, `total` olarak blok cue sayısını bildirir
— iki bloğun çıktı penceresi farklı boyda olduğu anda (50 cue → `37` + `13`)
ikinci blok düşüyor. Ölçüm: `MockTranslationProvider` ile 50 cue
`Block(Provider(Permanent))`, **aynı kod** 30 cue'da (tek blok) yeşil.
`nen-translate`'in kendi testleri bunu görmemişti çünkü fixture sağlayıcısı hiç
`progress` çağırmıyor. Kusur bu task'ın kapsamı değil; `NEN-106` olarak
dosyalandı ve `NEN-099`'u blokluyor. Bu task'ın uçtan uca testi kusuru
gizlemiyor: çok bloklu senaryo ilerleme bildirmeyen bir sağlayıcıyla sürülüyor
ve bunun sebebi test dosyasında `NEN-106`'ya referansla yazılı.

### Kapsam dışı bırakılanlar

- `persistence::fake` (in-memory store) yazılmadı — hiçbir DoD maddesi
  gerektirmiyor; `NEN-099`'un orkestrasyon testleri gerektirdiğinde kendi
  task'ında açılır (Kural 5).
- Metadata index ve sorgulama `NEN-098`'de, cache identity `NEN-097`'de
  (ADR-0018 henüz `accepted` değil, Kural 4 — bu task hiçbir kimlik semantiği
  kararlaştırmadı).
- Depo kökünü macOS kabuğunun `~/Library/Application Support/…` altında
  vermesi `NEN-101`'in işi; core kökü enjekte edilmiş bir yol olarak alıyor.
- Test `TempDir` yardımcısının üçüncü özel kopyası yazıldı; paylaşılan bir
  dev-crate'e refactor yapılmadı (Kural 5).
