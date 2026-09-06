---
id: NEN-057
title: Read the language a subtitle filename declares
milestone: M3
size: S
state: done
depends_on: [NEN-025]
blocks: []
adr: [29, 30]
---

# NEN-057 — Read the language a subtitle filename declares

## Sonuç

`Film.tr.srt` gibi bir dosya adı dilini söylüyorsa, o dil `resolve_language`'a
**metadata** olarak verilir; içerik tespiti yalnız teyit/çelişki için kullanılır.

## Bağlam

`NEN-025` uygularken ayrıldı. Bugün kullanıcı altyazısının dili yalnız
**içerikten** tespit ediliyor (`nen_subtitle::language::resolve_language`
`metadata: None` ile çağrılıyor). Bu doğru çalışıyor ama eldeki bilginin bir
kısmını kullanmıyor: `Film.tr.srt`, `Film.en.srt` gibi adlar kullanıcı
kütüphanelerinde yaygın ve dili **açıkça** söylüyor.

`resolve_language`'ın imzası bunu zaten bekliyor: metadata verildiğinde nihai
dili o belirliyor, metin yine de incelenip güvenilir bir çelişki
raporlanabiliyor (`MetadataLanguageConflict`). Yani eksik olan tek şey, dosya
adındaki alt-uzantıyı bir `LanguageTag`'e çevirmek.

`NEN-025`'in Kapsam listesinde yoktu, Kural 5 gereği oraya eklenmedi.

## Kapsam

- Dosya adının son alt-uzantısını dil etiketi olarak ayrıştırma denemesi
  (`Film.tr.srt` → `tr`, `Film.pt-BR.srt` → `pt-BR`)
- Ayrıştırılamayan alt-uzantı sessizce yok sayılır — `Film.forced.srt`,
  `Film.2019.srt` dil değildir ve hata değildir
- Bulunan etiket `resolve_language`'a metadata olarak verilir
- Gruplama ADR-0030'a göre primary subtag ile — `pt-BR` kaynağı bölgesini
  korur, grubu `pt` olur

## YAPILMAYACAK

- İçerik tespitini kaldırmak — çelişki raporu için metin yine incelenir
- Dosya adından başlık/yıl/release adı çıkarmak — o `nen-identity`'nin işi
  (`NEN-018`)
- Çelişki durumunda kullanıcıya soru sormak — bu milestone'da yüzey yok

## Kanıt (DoD)

- [x] `Film.tr.srt` Türkçe grubuna giriyor, içerik tespitine bakılmaksızın
- [x] Metin başka dilse `MetadataLanguageConflict` üretiliyor ama dil yine
      metadata'nın dediği
- [x] Negatif: `Film.forced.srt` ve `Film.2019.srt` dil ipucu üretmiyor
- [x] Negatif: dosya adı hiçbir log'a girmiyor (K23 #8)

## Kanıt kaydı

**Uygulama.** `nen_subtitle::language::from_file_name(file_name: &str) ->
Option<LanguageTag>` eklendi: `.srt` uzantısı atılır, kalan stem `.` ile
bölünür, sondan başlayarak bilinen erişilebilirlik işaretçileri
(`sdh`/`cc`/`forced`, case-insensitive) atlanır, kalan son segment
`LanguageTag::parse`'a verilir — parse başarısızsa (ör. `2019`, `zh-hant-cn`)
sessizce `None`. Adayın önünde gerçek (boş olmayan) bir segment olmalı, yoksa
`tr.srt` ve `.tr.srt` gibi adlar "dil" değil dosyanın kendisi sayılır ve
`None` döner. `hi` bilinçli olarak işaretçi kümesine **girmedi**: gerçek bir
ISO 639-1 kodu (Hintçe), ve iki harfli bir kodu erişilemez kılmaktansa
Hintçe'yi kazandırmak tercih edildi (kullanıcı kararı, 2026-09-06).

`subtitle_files::load` artık `from_file_name(&admitted.file_name)`'i
`resolve_language`'a metadata olarak veriyor (ADR-0029 Karar 5: metadata
kazanır, metin yine incelenip güvenilir çelişki taşınır). Çelişki taşınmıyor —
sunacak bir yüzey yok (task'ın YAPILMAYACAK maddesi).

**İkinci ISO tablosu yok.** `LanguageTag::parse`'ın ADR-0032 kanonikleştirmesi
bedava geldi: `Film.eng.srt` → `en`, `Film.pt-BR.srt` → `pt-br` (grup `pt`,
ADR-0030).

**Negatif kontrol üç yönde ve ayrık**, her biri tek başına geri alınıp yalnız
kendi testinin kırmızıya döndüğü ölçüldü:

1. `load`'da ipucu tekrar `None`'a sabitlenince yalnız
   `a_language_declared_in_the_filename_wins_over_the_text_it_names` kırmızı
   (1/25) — `a_filename_language_hint_is_ignored_when_it_is_not_a_real_sub_extension`
   yeşil kaldı, çünkü o test zaten içerik tespitine düşüyor.
2. İşaretçi atlama kaldırılınca yalnız `a_bare_marker_or_unparseable_suffix_produces_no_hint`
   ve `a_trailing_accessibility_marker_is_skipped_for_the_language_beneath_it`
   kırmızı (2/11) — `Film.sdh.srt` artık "sdh" dil etiketi üretiyordu.
3. "önünde gerçek segment olmalı" koşulu kaldırılınca yalnız
   `a_language_shaped_filename_with_no_real_prefix_produces_no_hint` kırmızı
   (1/11) — `.tr.srt` artık `tr` üretiyordu.

Üçü de düzeltmeyle birlikte geri getirildi ve tekrar yeşil doğrulandı.

**K23 #8 (dosya adı loglanmaz).** Ayrı bir fixture eklenmedi:
`guard_subtitle_file_debug.rs`'in var olan fixture'ı
(`Sevgilimle.Tatil.2019.tr.srt`) zaten bu task'ın ürettiği bir ipucu
(`tr`) taşıyor — yani mevcut guard, ipucunun kendisi hiçbir `Debug` çıktısına
girmediğini bu task'ın koduyla birlikte de kanıtlıyor. `SubtitleLibrary`'nin
`Debug`'ı yalnız sayaç basıyor; dosya adı ne kataloğa ne loga giriyor.

**Kapsam sınırı bulundu ve ayrı task'a dosyalandı (`NEN-075`).**
`subtitle_files::sidecar_of` yalnız medyanın tam basename'ini `.srt` ile
arıyor (`Film.mkv` → `Film.srt`), yani `Film.tr.srt` bugün sidecar olarak
**bulunmuyor** — bu task'ın etkisi yalnız `⇧⌘O` ile açıkça seçilen dosyalar.
Sidecar taramasını genişletmek güvenlik kapılarının dizin listelemeye
açılması demek, kendi kararını isteyen ayrı bir iş (Kural 5).

### Doğrulama çıktıları

```
$ cargo test --manifest-path core/Cargo.toml --workspace
594 passed, 0 failed, 1 ignored           → exit 0 (bu task ile 586 → 594)
  nen-subtitle lib +6 (from_file_name tablosu + metadata-çelişki testi)
  nen-app subtitle_file_gates +2 (dosya adı kazanıyor · gerçek olmayan
  alt-uzantı ipucu üretmiyor)

$ cargo fmt --check                          → exit 0
$ cargo clippy --all-targets -- -D warnings  → exit 0, uyarı yok
$ cargo deny check                           → advisories ok, bans ok,
                                                licenses ok, sources ok
                                                (yeni dış bağımlılık YOK)

$ bash scripts/test.sh                       → SONUÇ: 2 test dosyasının hepsi geçti
$ bash scripts/task-index.sh                 → INDEX.md yeniden üretildi
$ bash scripts/check-docs.sh                 → SONUÇ: tüm denetimler geçti
```

Not: `568 passed` `docs/STATUS.md`'nin `NEN-042` girişindeki tarihsel sayıydı;
bu task'tan **önceki** temiz `HEAD` (`e1cb3b2`) zaten **586 passed** veriyordu
(aradaki fark `NEN-033`'ün OpenSubtitles testleri) — baseline `git stash` ile
ayrıca ölçüldü.

Değişiklik yalnız Rust çekirdeğinde (`nen-subtitle`, `nen-app`); FFI yüzeyi ve
Swift kabuğu dokunulmadı, macOS paketi bu task'ın kapsamı dışında.

### Kapsam dışı bırakılanlar

- **Sidecar taramasının `Film.tr.srt` desenini görmesi** → `NEN-075`
  (backlog'a dosyalandı).
- **Çelişki durumunun kullanıcıya gösterilmesi** — bu milestone'da yüzey yok
  (task'ın kendi YAPILMAYACAK maddesi).
