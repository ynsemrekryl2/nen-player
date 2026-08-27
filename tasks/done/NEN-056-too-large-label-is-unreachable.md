---
id: NEN-056
title: Resolve the unreachable "çok büyük" reason label
milestone: M3
size: S
state: done
depends_on: [NEN-025]
blocks: [NEN-026]
adr: [31, 35]
---

# NEN-056 — Resolve the unreachable "çok büyük" reason label

## Sonuç

ADR-0031 Karar 5'in kendi içindeki çelişki giderilir; `NEN-026` üretilemeyecek
bir etiketi uygulamaya çalışmaz.

## Bağlam

`NEN-025` uygulanırken bulundu. ADR-0031 Karar 5 iki madde içeriyor ve ikisi
aynı anda doğru olamaz:

- Birinci madde, **kataloğa giren ama kullanılamayan** kaynağın taşıyacağı
  kapalı etiket kümesini sayıyor: `okunamadı` · `biçim hatalı` · **`çok büyük`**.
- İkinci madde, boyut sınırını **güvenlik kapısına** koyuyor: kapıdan dönen
  dosya kataloğa **hiç girmiyor**.

Boyut aşımı kataloğa hiç girmiyorsa, katalogdaki bir kaynağın `çok büyük`
etiketini taşıması mümkün değildir. `NEN-026` bu etiketi listeliyor, yani
üretilemeyecek bir durum için UI yazmak üzere.

`NEN-025` bu çelişkiyi **kendi DoD'una göre** çözdü — orası net: boyut → kapı →
katalogda yok, ve `an_oversized_file_is_refused_without_being_opened` bunu
kanıtlıyor. Yani bugünkü kod tutarlı; tutarsız olan doküman.

## Kapsam

- ADR-0031 Karar 5'in hangi yarısının doğru olduğuna karar ver
- Karar "boyut kapıdır" ise (bugünkü kod): Karar 5'in etiket kümesinden
  `çok büyük` çıkarılır, `NEN-026`'nın kapsamı buna göre düzeltilir
- Karar "boyut kataloğa girer" ise: `security-policy.md` §4 #4'ün "okunmaz"
  ifadesiyle çakışır, o yüzden **yeni bir ADR** gerekir ve `NEN-025`'in kapıları
  değişir
- ADR düzenlenmez — karar değişiyorsa `superseded` yapılıp yenisi yazılır
  (ADR-0001)

## YAPILMAYACAK

- Menü UI'ı → `NEN-026`
- `NEN-025`'in kapılarını bu task içinde değiştirmek — önce karar

## Kanıt (DoD)

- [ ] ADR-0031 Karar 5 kendi içinde çelişmiyor (tutarlılık okuması)
- [ ] `NEN-026`'nın etiket kümesi, üretilebilen durumlarla birebir eşleşiyor
- [ ] Seçilen kapalı kümenin her elemanı için onu üreten bir test var

## Kanıt kaydı

**Karar: boyut bir güvenlik kapısıdır** (kullanıcı kararı, 2026-08-27).
Çelişkinin iki yarısından bugünkü kodun ve `NEN-025`'in kanıtladığı yarı
seçildi; `çok büyük` etiketi kapalı kümeden düştü. Karar
[`ADR-0035`](../../docs/adr/0035-subtitle-source-reason-labels.md) ile
`accepted` oldu.

**ADR-0031 düzenlenmedi.** Gövde olduğu gibi duruyor; Notlar'a ADR-0035'e işaret
eden bir madde eklendi (ADR-0001'in açık istisnası). ADR-0031'i bütünüyle
`superseded` yapmak elendi: ona `adr:` alanıyla referans veren **7 done task**
var (`NEN-024` · `NEN-025` · `NEN-046` · `NEN-048` · `NEN-053` · `NEN-054` ·
`NEN-055`) ve `scripts/check-docs.sh` adım 6 bunların hepsini kırmızıya
döndürürdü.

**DoD #1 — Karar 5 kendi içinde çelişmiyor.** ADR-0035 Karar 2 kümeyi iki
elemana sabitliyor, Karar 4 ADR-0031'in geri kalanını yürürlükte bırakıyor,
Karar 1 boyutu kapıda tutuyor. ADR-0031 Notlar'ı okuyucuyu oraya yönlendiriyor.

**DoD #2 — `NEN-026`'nın etiket kümesi üretilebilen durumlarla birebir.**
`NEN-026` *Kapsam*'ındaki liste `okunamadı` · `biçim hatalı` oldu; *YAPILMAYACAK*
ve negatif DoD maddesi boyut kapısını da açıkça kapsıyor. Frontmatter
`depends_on: [NEN-019, NEN-025, NEN-056]` ve `adr: [10, 31, 35]` — bu task'ın
`blocks: [NEN-026]` kenarı artık iki yönlü, grafik tutarlı.

**DoD #3 — kümenin her elemanı için üretici test var.**

| Etiket | Test | Durum |
|---|---|---|
| `biçim hatalı` | `a_malformed_subtitle_is_catalogued_and_marked_rather_than_dropped` | zaten vardı |
| `okunamadı` | `an_undecodable_subtitle_is_catalogued_and_marked_unreadable` | **yeni** |

`okunamadı`'nın üreticisi yoktu — menünün çizeceği etiket, hiçbir testin
yürümediği bir kod yoluna dayanıyordu. Yeni test gerçek bir dosyaya UTF-16LE
BOM + tek başına yüksek surrogate yazıyor: §4'ün dört kapısını geçiyor (gerçek,
düz, küçük dosya), sonra ADR-0008 gereği mojibake yerine reddediliyor — yani
sonuç `Rejected` değil `Defective(Unreadable)`, kaynak katalogda kalıyor,
`is_usable` false, document yok.

İkinci test kümeyi kapatıyor: `the_menu_reason_labels_are_a_closed_set_of_exactly_two`
(`SourceDefect` üzerinde exhaustive `match` + `FileRejection::TooLarge`'ın
karşı taraf olduğu).

**Negatif kontrol iki yönde, ikisi de geri alındı.**

- Yeni testin beklentisi `Unreadable` → `Malformed` yapılınca **1 kırmızı**:
  `left: Defective(SourceDefect { reason: "unreadable" })` /
  `right: ... "malformed"`. Test "herhangi bir kusur" değil, tam olarak bu
  kusuru ölçüyor.
- `SourceDefect`'e üçüncü bir varyant (`Truncated`, `as_str` arm'ı ile birlikte)
  eklenince guard **derlenmiyor**:
  `error[E0004]: non-exhaustive patterns: 'SourceDefect::Truncated' not covered`
  → ADR-0035 Karar 3 ("kümeye eleman eklemek bir karardır") mekanik olarak
  zorlanıyor.

**Kapılar.** Rust testleri **489 → 491**; `cargo fmt --check`,
`cargo clippy --workspace --all-targets -D warnings`,
`cargo test --workspace --no-fail-fast` (491 passed / 0 failed),
`cargo deny check` (advisories · bans · licenses · sources ok),
`bash scripts/test.sh`, `bash scripts/task-index.sh --check`,
`bash scripts/check-docs.sh` — hepsi çıkış 0. Yeni dış bağımlılık yok,
`deny.toml` değişmedi. Ürün kodu **değişmedi**: bu task davranışı değil, kaydı
ve kanıtı düzeltti.

**Yan not — `NEN-026` başlamadan önceki kapı duruyor.** ADR-0031 Notlar'ının
istediği "beyin fırtınası 2. tur" (gerçek ekrana bakarak görsel ince ayar ve
§8'in görünen metninin doğrulanması) hâlâ `NEN-026`'nın önündedir; bu task onun
yerine geçmez.
