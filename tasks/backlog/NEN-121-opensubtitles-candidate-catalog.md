---
id: NEN-121
title: OpenSubtitles candidates catalogued without download
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-119, NEN-120, NEN-035]
blocks: [NEN-122]
adr: [21, 10]
---

# NEN-121 — OpenSubtitles candidates catalogued without download

## Sonuç

Medya açıldığında OpenSubtitles adayları `SubtitleSourceKind::OpenSubtitles`
kaynakları olarak — opaque public ID, dil ve rozetle — katalogda duruyor;
kataloglama sırasında hiçbir altyazı indirilmiyor ve private file ID / hash /
filename hiçbir log veya `Debug` yüzeyine düşmüyor.

## Kapsam

- `nen-providers::opensubtitles`: ADR-0021'in arama stratejisi (hash →
  doğrulanmış kimlik) için `SubtitleCandidateSearch` portu (`nen-ports`) ve
  gerçek adapter + deterministic fake; redakte fixture'lar
  (`fixtures/providers/opensubtitles/`)
- Cevap **untrusted**: bounded JSON, aday sayısı üst sınırı, dil kodu →
  `LanguageTag` (ADR-0032 emsali), geçersiz kayıt sessizce atlanır
- `nen-app::SubtitleLibrary::add_opensubtitles(candidates)` — `add_embedded`
  emsali; `SubtitleSourceId::opensubtitles(<opaque>)`; belge **yok**
  (`document()` → `None`, `is_usable` seçimde indirmeyi tetikleyecek
  `NEN-122`'ye kadar `false` değil, "indirilebilir" durumu — ADR-0021)
- Private `file_id` ↔ opaque ID eşlemesi `nen-app` içinde, FFI'ya çıkmıyor
- ADR-0010 projeksiyonu: adaylar tercih dili gruplarında, `NEN-035`'in
  sıralamasıyla; `AUTO_SELECTABLE_KINDS` **değişmiyor**
- `nen-ffi` `FfiMenuEntry`'de kind zaten `OpenSubtitles`; ek alan (ör. release
  adı gösterimi) ADR-0021'e göre

## YAPILMAYACAK

- İndirme — `NEN-122`
- macOS menü davranışı — `NEN-123`
- Otomatik seçim/indirme — `NEN-038` (ADR-0047 olmadan **yasak**)
- Anahtar yokken aramayı hataya çevirmek — sessizce aday yok

## Kanıt (DoD)

- [ ] Unit: fixture'dan adaylar kataloğa giriyor; tercih dili gruplarında
      görünüyor; belge yok
- [ ] Negatif (zorunlu): kataloglama boyunca download endpoint'ine `send`
      sayacı **0**; anahtar yokken arama `send` sayacı 0
- [ ] Negatif (zorunlu, K23): sentinel private `file_id`, hash ve filename
      `SubtitleSource`/`MenuEntryView`/`FfiMenuEntry`/hata tiplerinin
      `Debug`'ında yok; kasıtlı ikiz sızdırıyor
- [ ] Negatif: bozuk/oversize cevap tipli hata, katalog boş ama medya oynuyor
- [ ] Unit: `AUTO_SELECTABLE_KINDS` testi (`opensubtitles_is_never_selected_automatically`)
      değişmeden yeşil
- [ ] Contract kiti fake + gerçek adapter (fixture) ile

## Kanıt kaydı

<!-- done olurken doldurulacak -->
