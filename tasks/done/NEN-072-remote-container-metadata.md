---
id: NEN-072
title: Parse container metadata from remote byte windows
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-036]
blocks: []
adr: []
---

# NEN-072 — Parse container metadata from remote byte windows

## Sonuç

Uzak medyanın bounded byte pencerelerinden Matroska/MP4 container metadata'sı
çıkarılır ve `MediaEvidence` içindeki container alanına taşınır.

## Kapsam

- İlk byte penceresinden Matroska/MP4 metadata ayrıştırma
- Boyut, derinlik ve malformed input sınırları
- Deterministic fixture ve negatif parser testleri

## YAPILMAYACAK

- Medyanın tamamını indirmek
- Playback demuxer'ını yeniden yazmak
- Torrent/debrid metadata'sı okumak

## Kanıt (DoD)

- [x] Matroska ve MP4 title/year fixture'ları `ContainerMetadata`'ya dönüşüyor
- [x] Bounded ve malformed byte input reddediliyor veya güvenle boş dönüyor
- [x] Negatif: oversized/deep/malformed container input paniklemiyor

## Kanıt kaydı

**Yeni bağımlılık yok.** `nen-identity/src/container.rs`'e `parse_head_window`
eklendi — magic byte'lardan (EBML `1A 45 DF A3`, ISOBMFF `ftyp`) formatı
sniff edip elle yazılmış, sınırlı bir EBML (Matroska) / ISOBMFF (MP4)
element/box walker'ına delege ediyor. Yalnız `\Segment\Info\Title` ve
`\Segment\Tags\Tag\SimpleTag` (Matroska) / `moov\udta\meta\ilst\©nam`+`©day`
(MP4) aranıyor; bulunan ham string'ler mevcut `from_tags()`'e (temizleme/
uzunluk sınırı/kontrol karakteri filtresi zaten test edilmiş) besleniyor.
Bulgular gerçek üretilmiş fixture'ların hex dökümüyle doğrulandı (element
ID'leri, VINT boyut kodlaması, MP4 box/atom yerleşimi) — tahmine dayanmadı.

**`nen-app::remote_evidence::collect_with_policy`** artık zaten çekilen
`head_window.body`'yi (`os_hash::CHUNK_BYTES` = 64 KiB, NEN-036) hash'in
yanında `container::parse_head_window`'a da veriyor; boş değilse
`MediaEvidence.with_container(...)`. Resolve/katman sırasına (`layers()`)
dokunulmadı — o zaten container'ı bekliyordu.

**Yol üstünde gerçek bir panik bulundu ve düzeltildi (task'ın kendi
gereksinimi, ayrı task açılmadı):** `read_size`'ın marker-mask hesaplaması
`0xFFu8 >> len` idi; 8 baytlık bir EBML size VINT'i (`len == 8`, gerçek
`contract-clip.mkv`-tarzı her Matroska Segment'inde görülen sıradan bir
kodlama) `u8` üzerinde 8 bit sağa kaydırma denemesiyle panikliyordu. Bu, kendi
golden fixture'ımın **kendisinde** de tetiklenirdi — adversarial input
beklemeden ilk gerçek dosya testinde yakalandı. Düzeltme: `checked_shr`.

**Golden (format):** `fixtures/media/container/valid-title.{mkv,mp4}`
(ffmpeg-üretilmiş, title+date metadata ile, `.ffmpeg.txt` üretim komutu
yanında) → `core/crates/nen-identity/tests/container_remote_window.rs` ve
`nen-app::remote_evidence::tests::container_title_and_year_from_the_head_window_reach_evidence`
title/year'ı uçtan uca (`collect_with_policy` → `MediaEvidence.container()`)
doğru çıkarıyor.

**Negatif (security, zorunlu) — üç ayrı senaryo:**
- *oversized*: `Segment` 8 baytlık VINT ile buffer'dan çok daha büyük bir
  boyut bildiriyor (`0x00FFFFFFFFFFFFFE`) → walker gerçek pencereye clamp
  ediyor, panik yok, boş sonuç.
- *deep*: `Segment` içinde 2000 seviye iç içe geçmiş `Info`-şekilli eleman
  (`Title` yerine birbirini saran) → walker yalnız tanınan tek bir çocuğa
  iniyor (şema-sabit, girdinin iddia ettiği derinliğe göre değil), sonuç boş,
  hızlı tamamlanıyor.
- *malformed*: NEN-022'nin deterministik `broken-clip.mkv`'si (tanınmayan
  rastgele 8192 bayt) ve elle kesilmiş EBML/ISOBMFF header'ları (her kesim
  noktasında) → boş sonuç, panik yok.
- Ayrıca `nen-app` tarafında: tanınmayan head window `MediaEvidence.container()`'ı
  `None` bırakıyor (entegrasyon kapısının kendisi test edildi).

**Doğrulama:** `cargo test --workspace` (yeni testler dahil tüm paket yeşil,
sayı öncekinden +19 civarı — `nen-identity` 129, `nen-app` 55 dahil), `cargo
fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` (crate
`#![deny(clippy::indexing_slicing)]` dahil temiz — tüm elle indeksleme `.get()`
tabanlı, panik yapamayan erişime çevrildi), `cargo deny check` (advisories/
bans/licenses/sources ok, yeni bağımlılık yok), `bash scripts/test.sh`,
`bash scripts/check-docs.sh` çıkış 0.

Değişiklik yalnız `nen-identity` ve `nen-app`; FFI yüzeyi ve macOS kabuğu
dokunulmadı (bu task'ın kapsamı değildi — `MediaEvidence.container()` zaten
FFI'ya bağlıydı, yeni bir alan eklenmedi).
