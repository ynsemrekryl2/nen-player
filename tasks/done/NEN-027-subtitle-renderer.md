---
id: NEN-027
title: SubtitleRenderer port and libmpv injection adapter
milestone: M3
size: M
state: done
closed: 2026-09-05
depends_on: [NEN-017, NEN-026, NEN-066]
blocks: [NEN-028]
adr: [13]
---

# NEN-027 — SubtitleRenderer port and libmpv injection adapter

## Sonuç

Seçilen altyazı video üzerinde görünür ve seek sonrası doğru cue **anında**
gösterilir.

## Kapsam

- `SubtitleRenderer` portu ve capability'leri
- libmpv external subtitle injection adapter'ı
- `CueIndex` (NEN-017) ile seek sonrası anlık doğru cue
- UI'ın renderer implementasyonunu **bilmemesi**

## YAPILMAYACAK

- Custom overlay renderer → manuel sync/inspector ihtiyacında (M7)
- Stil/konum ayarları
- Sync offset uygulaması → M7

## Kanıt (DoD)

- [x] Seçim anında altyazı görünüyor (ekran kaydı)
- [x] Rastgele seek sonrası gösterilen cue, `CueIndex` sonucuyla birebir aynı
- [x] Cue'suz bir ana seek → altyazı gösterilmiyor (boş, artık cue kalmıyor)
- [x] UI kodunda renderer implementasyon adı geçmiyor (grep kanıtı)

## Kanıt kaydı

**Kod bu task'ta 2026-08-29'da tamamlandı, kabul 2026-09-05'te kapandı.** Port
(`core/crates/nen-ports/src/renderer/`), engine-native adapter, session yolu
(`core/crates/nen-app/src/renderer.rs`), FFI ve kabuk sadeleşmesi o gün
yerindeydi; DoD'nin ikinci, üçüncü ve dördüncü maddeleri de o gün karşılandı.
Karşılanamayan tek madde **"seçim anında altyazı görünüyor"** idi: belge motora
ulaşıyor, seçiliyor ve motor çizdiğini bildiriyordu (`sub-text` dolu) ama
pencerede hiçbir şey görünmüyordu.

**Kusur bu task'ın enjeksiyon yolunda değildi.** Aynı boşluk gömülü track'i de
vuruyordu; kök neden `NEN-066`'da ölçüldü — 134 pt'lik cam transport altyazının
piksel bandını örtüyordu — ve ADR-0037'nin güvenli alanıyla kapandı. `NEN-060`
aynı kök nedene katlanıp `canceled` oldu.

**Görsel kabul (2026-09-05, ad-hoc imzalı gerçek `.app`).** Altı adımın altısı
da geçti; ayrıntı `evidence/M3/NEN-027-checklist.md`. Ölçüyü belirsiz
bırakmamak için seek ±5 sn düğmeleriyle yapıldı ve adımların çoğu
**duraklatılmış** durumda koşuldu — 2026-08-29'da başarısız olan tam olarak
buydu:

- 00:05'te (gömülü track'in cue'suz anı) ekran boşken kullanıcı dosyası
  seçildi → 1. replik **oynatma gerekmeden** çizildi.
- +5 sn → 00:10: 2. replik anında geldi, 1. replik değil.
- +5 sn → 00:15 (cue'suz an): ekran boşaldı, artık replik kalmadı.
- 00:11'de — iki kaynağın da cue'su olan an — gömülü Türkçe track'e geçildi:
  **tek** replik çizildi, üst üste binme yok.
- `Kapalı`: aynı anda ekran boşaldı, CC etiketi `Kapalı` oldu.
- Ek olarak oynarken koşuldu: replik cue içinde görünüyor, boşlukta kayboluyor,
  krom gizlenince alt banda iniyor.

Kareler: `evidence/M3/NEN-027-selection.png` · `NEN-027-after-seek.png` ·
`NEN-027-gap.png` · `NEN-027-embedded.png`. Hiçbirinde tam dosya yolu, dosya
seçim diyaloğu veya özel medya metadata'sı yok; replikler uydurmadır (K23).

**Otomatik kanıt bugünün ağacında yeniden koşuldu.** Rust workspace **72 hedef ·
560 passed · 1 ignored**; macOS Swift paketi temiz koşuda **151/151**. `.app`
build'i ve strict codesign exit 0; shell testleri ve doküman kapıları yeşil.
DoD'nin dördüncü maddesinin iki grep'i hâlâ eşleşmesiz — kabuk `sub-add`,
`memory://`, `SubtitleRenderer`, `selectTrack` adlarının hiçbirini bilmiyor.

**Yol üstünde bir gözlem kaydedildi, kapsama alınmadı.** İlk tam Swift koşusu
`ContractTests` içinde bir kez kırmızı verdi; ikinci tam koşu baştan sona
yeşil. Aynı kararsızlık `NEN-070` koşusunda da kaydedilmişti ve `NEN-049` tam
olarak bu gözlemi bekleyen açık task'tır (Kural 5: mevcut task'a eklenmedi).

`NEN-028` — macOS vertical slice acceptance — artık READY.
