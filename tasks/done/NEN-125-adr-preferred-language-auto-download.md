---
id: NEN-125
title: Decide the preferred-language auto-download contract
milestone: M6
size: S
state: done
closed: 2026-09-14
depends_on: [NEN-035, NEN-122]
blocks: [NEN-038]
adr: [47]
---

# NEN-125 — Decide the preferred-language auto-download contract

## Sonuç

ADR-0047 `accepted`: ADR-0010 Karar 9'un ertelediği üçüncü basamağın
(`opensubtitles` otomatik seçimi) açılma sözleşmesi — güven eşiğinin sayısal
değeri, kullanıcının kapatabilmesi ve başarısızlık davranışı — karara
bağlanmış; `NEN-038` başlatılabilir.

## Bağlam

`NEN-038` kendi metninde "ADR yazılıp kabul edilmeden bu task `active`'e
alınmaz" diyor ve ADR'nin en az üç şeyi kararlaştırmasını istiyor. ADR-0010
Karar 9 basamağı reddetmedi, **erteledi**: "doğruluk ölçülünce
(NEN-033/035/036) kendi ADR'siyle açılır". Eşik ölçülmeden yazılamaz —
`NEN-035`'in fixture korpusu ölçümü bu ADR'nin girdisi.

## Kapsam

ADR-0047 en az şunları kararlaştırır:

- Güven eşiği: `NEN-035`'in raporundan sayısal değer (eşik değil **ölçülmüş
  baseline** üzerinden; `testing-strategy.md` "Baseline ve invariant ayrımı")
- Kullanıcı ayarı: varsayılan açık/kapalı, ayar yüzeyi (ADR-0031 Karar 6)
- Tetikleme: medya açılışında, yalnız tercih dilinde `embedded`/`user`
  kaynak yoksa; ikinci tercih dile düşme
- Başarısızlık: indirme reddi/kota → sessizce `Kapalı`, tekrar deneme yok;
  yanlış altyazı indiğinde kullanıcı seçimi sessizce değişmez
- Kota etkisi: her açılışta indirme → günlük kotayla ilişki
- `AUTO_SELECTABLE_KINDS`'a `OpenSubtitles`'ın **koşullu** girişi —
  `nen-catalog`'un saf fonksiyonu nasıl parametrelenir

## YAPILMAYACAK

- İmplementasyon — `NEN-038`
- `ai` kaynağını otomatik seçime sokmak — §9, ADR-0010 Karar 9 **yasak**
- Eşiği ölçmeden sabitlemek

## Kanıt (DoD)

- [x] `docs/adr/0047-*.md` yazıldı, kullanıcı onayıyla `accepted`; eşik
      `NEN-035`'in kanıt kaydına atıfla
- [x] ADR-0010'a Notlar girdisi (Karar 9 basamağı açıldı)
- [x] `docs/adr/README.md`, `docs/DECISIONS.md` sayacı tutarlı
- [x] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

### Karar

ADR-0047, kullanıcı onayıyla `accepted` oldu. OpenSubtitles otomatik indirmesi
varsayılan olarak kapalıdır; ayar açıldığında yalnız yerel kaynak yoksa birinci
ve ikinci tercih dili için, `Automatic` ve çakışmasız identity assessment ile
`ConfidenceScore >= 60` kapısından sonra çalışır. Medya başına günlük tek
deneme, hata sonrası retry yok, kullanıcı seçimini sessizce değiştirmeme ve
`ai` kaynağını otomatik seçmeme kuralları sabittir. İmplementasyon `NEN-038`'e
aittir.

### Doğrulama

- [`ADR-0047`](../../docs/adr/0047-preferred-language-auto-download.md)
  2026-09-14 kullanıcı onayıyla `accepted`; ADR-0010 Notlar, ADR README ve
  `docs/DECISIONS.md` sayacı/listesi hizalı.
- `NEN-035` baseline'ı: 10 vaka, 8 otomatik (%80), 1 aday (%10), 1 manuel
  (%10), yanlış sessiz otomatik karar 0; mevcut eşik `ConfidenceScore >= 60`.
- [`NEN-125-checklist.md`](../../evidence/M6/NEN-125-checklist.md) karar ve
  güvenlik sınırlarını, komut sonuçlarını ve CI kanıtını kaydeder.
- `bash scripts/check-docs.sh` (**10/10**),
  `bash scripts/task-index.sh --check` ve `git diff --check`: geçti.
- CI run **34886476167**: `cargo fmt --check`, `cargo clippy`, `cargo test`,
  `cargo deny check`, `bash scripts/test.sh`, iki index denetimi ve
  `bash scripts/check-docs.sh`: tamamı geçti.
