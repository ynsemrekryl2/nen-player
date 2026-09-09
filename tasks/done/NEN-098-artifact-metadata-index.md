---
id: NEN-098
title: Artifact metadata index and offline lookup
milestone: M5
size: M
state: done
closed: 2026-09-09
depends_on: [NEN-096, NEN-097]
blocks: [NEN-099]
adr: [0017]
---

# NEN-098 — Artifact metadata index and offline lookup

## Sonuç

Daha önce üretilmiş bir çeviri, uygulama yeniden başlatıldıktan sonra ve **ağ
olmadan** cache identity'siyle bulunup açılıyor.

## Bağlam

Şartname §11: "restart sonrası reuse". S4 kararı (2026-09-08) bunu M5'in
DoD'una alıyor: kaydedilmiş bir artifact ağ olmadan açılıp oynatılır, ayrı bir
"offline modu" anahtarı yoktur.

S9 kararı (2026-09-08): farklı provider/model/glossary ile üretilmiş artifact'ler
diskte yan yana durur (kimlikleri zaten ayırıyor), ama hedef dil grubunda M5'te
yalnız **en yeni** olan gösterilir.

## Kapsam

- Metadata index (ADR-0017'nin kararlaştırdığı biçimde) ve kimliğe göre arama
- Hedef dil başına "en yeni artifact" projeksiyonu
- Index ile içerik deposu arasındaki tutarsızlığın (kayıt var, içerik yok)
  sessizce değil tipli hatayla karşılanması

## YAPILMAYACAK

- Katalog/menü entegrasyonu — `NEN-099`
- Artifact silme veya cache temizleme komutu — M6
- Çoklu artifact'in kullanıcıya seçtirilmesi — S9 gereği M5'te yok

## Kanıt (DoD)

- [x] Unit: yazılan artifact yeni bir process/store örneğinden kimliğiyle bulunuyor (restart reuse)
- [x] Negatif: **ağ erişimi olmayan** bir ortamda arama ve okuma çalışıyor — HTTP portu hiç çağrılmıyor (çağrı sayacı 0)
- [x] Negatif: cache identity bileşenlerinden biri değişmiş bir sorgu eski artifact'i **bulmuyor**
- [x] Negatif: index'te kayıtlı ama içeriği silinmiş artifact tipli hata veriyor, boş/yarım belge döndürmüyor
- [x] Unit: aynı hedef dil için iki artifact varken projeksiyon en yenisini veriyor, ikisi de diskte kalıyor
- [x] Guard: index sorgusu ve dosya yolu loglanmıyor (K23 #3, #7)

## Kanıt kaydı

**Kimlik artifact'in kendi bilgisi oldu.** `nen-translate::artifact::ValidatedSubtitleArtifact`
artık `assemble`'ın kendisine verdiği `BlockLayout`'un `config()`'ini
(`block_size`/`overlap`) saklıyor ve yeni `cache_identity()` metodu
`identity::CacheIdentity::of`'u tamamen kendi alanlarından çağırıyor — kullanıcı
kararı gereği yanlış kimlikli bir artifact üretmek mümkün değil, çünkü kimliğin
girdisi başka hiçbir yerden gelmiyor. `to_record()`'un imzası değişmedi
(`NEN-096`'nın yüzeyi korundu); yalnız döndürdüğü `ArtifactRecord`'a yeni
`cache_identity: CacheKey` alanı eklendi.

**Port: `CacheKey` + `ArtifactIndex`.** `nen-ports::persistence`'e
`ContentAddress`'in birebir deseniyle (`from_bytes`/`from_hex`/`to_hex`,
`Debug`/`Display` `<redacted>`) yeni `CacheKey` newtype'ı ve `ArtifactIndexEntry`
(adres · cache identity · source fingerprint · hedef dil · `created_at`)
eklendi. `nen-translate` bu crate'i göremediği port sınırını (ADR-0006)
`impl From<CacheIdentity> for CacheKey` ile aştı — kimlik iki kez hesaplanmıyor,
yalnız baytları taşınıyor. Yeni `ArtifactIndex` trait'i (`entries` · `find` ·
`latest_for_target`) ayrı bir trait: ADR-0017'nin "SQLite'a geçiş port
sınırında soğurulur" iddiasının tutması için sorgu yüzeyinin de port'ta durması
gerekiyordu. Kontrat kiti `check_index` ile genişletildi (`IndexMiss` ·
`IndexProjection`).

**Adapter: dizin taraması, yeni okuma yolu yok.** `FilesystemArtifactStore`
`ArtifactIndex`'i `artifacts/`'i tarayarak, yalnız `<64hex>.json` adlı dosyaları
aday sayıp her birini **mevcut `get()` yolundan** (hash doğrulaması, boyut
sınırı, sembolik bağ reddi dahil) geçirerek implemente ediyor —
okunamayan/bozuk/adı uymayan bir dosya kullanıcı kararı gereği **atlanıyor**,
tarama durmuyor. `wire.rs`'in dosya formatı 1→2'ye bitti (`cache_identity` alanı
eklendi); eski format dosyası sessizce yanlış okunmak yerine `Corrupt` ile
reddediliyor.

**Altı DoD maddesinin her biri gerçek testle kanıtlandı, dördü elle mutasyona
uğratıldı** (kaldırılınca tam olarak beklenen test(ler) kırmızıya döndü,
başkası etkilenmedi — kontrol sağır değil):
- Taramanın hex/uzantı filtresi kaldırılınca (adı doğrudan `from_hex`'e
  verildi) 7 test kırmızıya döndü — restart reuse, miss, deleted, projection,
  corrupt-skip, stray-file ve `the_shared_index_contract_kit_passes` — çünkü
  geçerli hiçbir artifact artık taranamıyor.
- `get()` hatasının atlanması yerine yayılması (kayıt var/içerik yok mutasyonu)
  yalnız `a_corrupt_artifact_is_skipped_rather_than_failing_the_scan`'ı
  kırmızıya döndürdü.
- `latest_for_target`'ın `pop()` yerine ilk (en eski) adayı seçmesi yalnız
  `latest_for_target_picks_the_newest_and_leaves_both_stored` ve
  `the_shared_index_contract_kit_passes`'i kırmızıya döndürdü.
- `find()`'ın istenen anahtarı yok sayıp ilk kaydı döndürmesi
  `a_query_under_a_different_identity_misses`,
  `the_shared_index_contract_kit_passes` (nen-persist) ve
  `a_query_under_a_changed_block_layout_component_does_not_find_the_old_artifact`
  (nen-app, gerçek pipeline üzerinden) — üç ayrı yerde kırmızıya döndü.

**Ağsız okuma, iddiaya değil ölçüme dayanıyor.** `nen-persist/tests/offline_lookup.rs`
sayaçlı bir `HttpClient` tutarken tam bir yaz/tara/bul/oku döngüsü çalıştırıyor
ve sayacın `0` kaldığını iddia ediyor; sağırlık kontrolü aynı sayaca doğrudan
bir `send()` çağrısı yapıp `1`'e çıktığını gösteriyor.

**Değişen bileşen ucu, gerçek pipeline'dan geçiyor.** `nen-app/tests/artifact_index_lookup.rs`
aynı belgeyi iki farklı `BlockLayoutConfig`'le (ADR-0018'in bir bileşeni) gerçek
`MockTranslationProvider` üzerinden çevirip iki gerçek artifact üretiyor; yeni
config'in artifact'i depoya yazılıyor, eski config'in kimliğiyle yapılan sorgu
**bulamıyor**, yeni kimlikle yapılan sorgu buluyor — yalnız `nen-translate`'in
saf `HashMap` testinin (`cache_identity_negative.rs`) değil, gerçek dosya
sistemi deposunun da bu kuralı tuttuğu kanıtlanıyor.

**K23 guard'ları.** `nen-ports/tests/guard_persistence_index_debug.rs` (3 test)
`CacheKey` ve `ArtifactIndexEntry`'nin `Debug`/`Display`'inde hex/hash
sızmadığını, kasıtlı `#[derive(Debug)]` ikizinin aynı sentinel'ı sızdırdığını
gösteriyor; `nen-persist/tests/guard_persist_debug.rs` genişletildi
(`ArtifactRecord.cache_identity` de artık kapsamda).

`cargo test -p nen-ports`: **61+3(+diğer) passed**; `cargo test -p nen-persist`:
**23 lib + 5 guard + 1 offline = 29 passed**; `cargo test -p nen-app` (yeni
dosyalar): **2 (`artifact_index_lookup`) + 3 (`artifact_store_roundtrip`)
passed**; workspace **806 passed / 1 ignored** (`NEN-097` baseline 793 + bu
task'ın 13 testi). `cargo fmt --check`, `cargo clippy --workspace --all-targets
-- -D warnings`, `cargo deny check` (yeni dış bağımlılık yok — `Cargo.lock`
diff'i boş), `bash scripts/test.sh` (4/4) ve `bash scripts/check-docs.sh`
yeşil. `nen-ffi`, macOS kabuğu dokunulmadı.
