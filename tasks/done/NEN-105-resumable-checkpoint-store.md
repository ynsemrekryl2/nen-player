---
id: NEN-105
title: Resumable checkpoint store for interrupted translation runs
milestone: M6
size: M
state: done
closed: 2026-09-11
depends_on: [NEN-096]
blocks: []
adr: [0017]
---

# NEN-105 — Resumable checkpoint store for interrupted translation runs

## Sonuç

Çeviri ortasında kapanan uygulama yeniden açıldığında `NEN-093`'ün ürettiği
`BlockCheckpoints`'i diskten okuyup kaldığı bloktan devam ediyor; hiçbir
koşulda yarıda kalmış bir resume kaydı okunabilir bir artifact olarak
görünmüyor.

## Bağlam

ADR-0017 Karar 5 (2026-09-09, `NEN-095`) checkpoint'in kalıcı olacağını
kararlaştırdı ama implementasyonu bilerek M6'ya erteledi: M5 mock provider ile
çalışıyor, baştan başlamanın maliyeti bugün ölçülemeyecek kadar düşük. M6'da
gerçek sağlayıcı geldiğinde yarıda kalan bir çeviri gerçek kredi/kota kaybı
demek — erteleme burada biter.

`NEN-093`'ün `BlockCheckpoints::into_completed`'ı bugün "her blok
checkpoint'li değilse `CompletedBlocks` üretilmez" değişmezini tip düzeyinde
zaten garanti ediyor; bu task o garantiyi bozmadan checkpoint'lerin **diske**
yazılmasını ekliyor.

## Kapsam

- `BlockCheckpoints`'in ADR-0017 Karar 5'in tanımladığı **ayrı resume
  alanına** (artifact deposunun dışında) yazılması ve okunması
- Aynı çeviri run'ının kimliğiyle (cache identity, `NEN-097`) resume alanının
  eşleştirilmesi
- Yeniden başlatmada zaten checkpoint'li blokların provider'a hiç
  gönderilmemesi (mevcut bellek-içi davranışın disk üzerinde de korunması)
- Tamamlanan run'ın resume kaydının temizlenmesi (artifact commit edildikten
  sonra resume alanı çöp bırakmaz)

## YAPILMAYACAK

- Artifact deposunun kendisi — `NEN-096` (zaten done/yapılıyor)
- Kullanıcıya "yarım çeviriler" listesi veya UI — M6/UI işi, burada yalnız
  API sınırı
- Cache identity hesabı — `NEN-097` (zaten var)

## Kanıt (DoD)

- [x] Unit: kaydedilen `BlockCheckpoints` aynı run kimliğinden birebir
      okunuyor (resume)
- [x] Unit: zaten checkpoint'li bloklar yeniden başlatmada provider'a
      gitmiyor (çağrı sayacıyla ölçülür)
- [x] Negatif (zorunlu — security/validation satırı): yarıda kesilmiş bir
      resume yazımı sonrası **okunabilir hiçbir kısmi artifact yok** — ne
      resume alanından ne artifact deposundan
- [x] Negatif: atomik yazım düz yazımla değiştirildiğinde yukarıdaki test
      kırmızıya dönüyor — kontrol sağır değil
- [x] Unit: artifact commit edildikten sonra resume kaydı temizleniyor
- [x] Guard: resume alanının dosya yolu hiçbir log yüzeyine düşmüyor (K23 #3)

## Kanıt kaydı

### Uygulama

- `nen-ports::persistence` içine payload'suz `ResumeStoreError`, redaction-safe
  `ResumeRecord`/blok/cue DTO'ları ve senkron, object-safe `ResumeStore`
  portu eklendi. `FilesystemArtifactStore` aynı enjekte edilen kökün ayrı
  `resume/` alanında bu portu uyguluyor; artifact taraması bu alanı hiç
  görmüyor.
- Resume snapshot'ı cache identity ile adlandırılıyor ve sürümlü JSON olarak
  aynı dizindeki benzersiz geçici dosyaya yazılıyor; dosya `fsync` → atomik
  `rename` → dizin `fsync` sırası artifact commit'iyle aynı güvenceyi taşıyor.
  Okuma 16 MiB sınırı, regular-file kontrolü, wire sürümü ve gömülü cache-key
  eşitliğini doğruluyor.
- `translate_resumable`, diskten gelen metni güvenilir saymıyor: cache key,
  blok sayısı, sıralı önek, cue ID/kapsam ve boş olmayan metin mevcut
  `validate_block` üzerinden yeniden doğrulanıyor. Bozuk kayıt kullanıcı
  kararıyla silinmeden `ResumeStoreError::Corrupt` veriyor ve provider çağrısı
  sıfır kalıyor.
- Her doğrulanmış blok, `TranslationCall::commit` delivery gate'i içinde tam
  snapshot olarak kalıcılaşıyor; yazım başarısızsa bellek checkpoint'i de
  ilerlemiyor. Artifact commit'inden sonra resume siliniyor; commit ile silme
  arasındaki süreç kesintisinden kalmış kayıt sonraki cache hit'te temizleniyor.

### Test kanıtı

- `a_new_process_resumes_from_disk_without_retranslating_the_validated_prefix`:
  70 cue / 2 blok. İlk provider iki çağrı gördü (blok 0 doğrulandı, blok 1
  bilinçli kalıcı hatayla kesildi), diskte 1 resume ve 0 artifact kaldı. Yeni
  `FilesystemArtifactStore` örneğiyle ikinci süreç yalnız 1 provider çağrısıyla
  kalan bloğu tamamladı; sonuçta 1 artifact ve 0 resume vardı.
- `a_corrupt_resume_stops_before_any_provider_call`: bozuk wire kaydı tipli
  resume hatası verdi, provider çağrısı **0**, kayıt silinmeden kaldı.
- `resume_restore_rejects_wrong_identity_shape_order_and_cues`: yanlış cache
  key, blok sayısı, önek sırası ve cue ID ayrı ayrı `Corrupt` ile reddedildi.
- `an_interrupted_resume_commit_leaves_nothing_readable`: yarım yazan gerçek
  atomik commit yolu `ResumeStoreError::Io` döndürdü; `load == Ok(None)` ve
  artifact listesi boş kaldı. Sağır olmayan ikiz
  `a_plain_resume_write_exposes_the_partial_file` aynı kesintinin düz yazımda
  okunabilir hedef adı altında `Corrupt` bıraktığını gösterdi.
- Gerçek mutasyon: üretim `commit_resume_with` geçici dosya yerine doğrudan
  hedefe yazacak şekilde değiştirildiğinde atomiklik testi beklenen biçimde
  kırmızı oldu (`left: Err(Corrupt)`, `right: Ok(None)`); mutasyon geri alındı
  ve aynı test yeşil geçti.
- `a_cache_hit_never_calls_the_provider_while_a_changed_block_layout_does`
  içine eklenen kesinti aralığı, geçerli artifact yanında kalmış resume'ın
  sonraki cache hit'te temizlendiğini doğruladı.
- K23 guard'ı resume metni, cache-key dosya adı ve store kökü sentinel'lerinin
  `Debug`/`Display` ve hata yüzeylerine düşmediğini; derived ikizin aynı
  sentinel'i gerçekten sızdırdığını doğruladı.

### Kapılar

- `cargo fmt --all -- --check` — geçti.
- `cargo clippy --workspace --all-targets -- -D warnings` — geçti.
- `cargo test --workspace` — geçti; yalnız mevcut `lookup_bench` ignored.
- `cargo deny check` — geçti (`advisories/bans/licenses/sources ok`; yalnız
  mevcut `hashbrown`/`syn` duplicate uyarıları).
- `bash scripts/build-apple.sh` — `nen-ffi` build ve Swift binding üretimi
  geçti.
- `bash scripts/test.sh` — 4/4 test dosyası geçti.
