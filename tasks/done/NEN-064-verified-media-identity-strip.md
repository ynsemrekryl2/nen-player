---
id: NEN-064
title: Verified media identity in player chrome
milestone: M6
size: L
state: done
closed: 2026-09-14
depends_on: [NEN-120, NEN-061]
blocks: [NEN-126]
adr: [9, 31]
---

# NEN-064 — Oynatıcı kromunda doğrulanmış medya kimliği

## Sonuç

Resmî API'den kesin hash eşleşmesiyle doğrulanan başlık, yıl ve varsa
sezon/bölüm bilgisi playback'i bekletmeden üst medya şeridinde görünür;
doğrulanmış kimlik yoksa mevcut basename kalır.

## Kapsam

> **Bağımlılık düzeltmesi (M6 kırılımı, 2026-09-11):** `NEN-033` kimlik
> sorgusunu Rust'ta bitirdi ama uygulamaya bağlamadı — `nen-ffi`'de çağıran,
> macOS'ta anahtar yok. Kimlik kabuğa ancak `NEN-120` ile ulaşır; bu task
> onu bekler.

- Yalnız `NEN-033`'teki resmî OpenSubtitles API'sinin kesin hash `Match`
  sonucu zengin kimlik sayılır. `NoMatch`, `Ambiguous` ve hata sonucu kimlik
  değişikliği üretmez.
- Application katmanı optional
  `VerifiedMediaIdentity { title, year, season, episode }` sunar; tek
  `nen-ffi` kapısından Swift'e geçer. Playback portu genişletilmez.
- Kimlik sorgusu playback'i bloklamaz. Sonuç gelene kadar basename görünür;
  medya değişmişse eski revision'a ait geç sonuç yok sayılır.
- Sunum tek satırdır: film `Inception (2010)`, dizi
  `Breaking Bad (2008) · S01E02`. Eksik optional alanlar sessizce atlanır.
- Kimlik değişimi mevcut 0,24 sn ease-out geçişini kullanır; shutdown ve
  yeni medya stale sonucu etkisiz kılar.

## YAPILMAYACAK

- Filename/klasör tahminini "doğrulanmış" kimlik gibi göstermek
- Aday seçimi veya manuel başlık/yıl/sezon/bölüm düzeltme UI'ı
- Çözünürlük, codec, bitrate, SDR/HDR veya engine metadata'sı
- Tam yol, URL/query, hash, private provider ID veya filename metadata'sını
  log ya da kanıt yüzeyine çıkarmak

## Kanıt (DoD)

- [x] Kesin fake hash eşleşmesi film ve dizi alanlarını FFI'dan eksiksiz
      geçiriyor ve kompakt etiketi üretiyor
- [x] `NoMatch`, `Ambiguous` ve typed hata basename fallback'ini koruyor;
      playback kesilmiyor ve kullanıcıya hata gösterilmiyor
- [x] Medya revision'ı değiştikten sonra tamamlanan eski sorgu şeridi
      değiştirmiyor; shutdown sonrası late update yok
- [x] Negatif redaction testinde yol, query, hash, private ID ve özel filename
      hiçbir `Debug`/log/kanıt çıktısında yok
- [x] Basename → doğrulanmış film/dizi etiketi 0,24 sn geçişle gerçek
      `.app` üzerinde ve fixture medyayla doğrulanıyor
- [x] Workspace, macOS testleri ve doküman denetimleri yeşil

## Kanıt kaydı

<!-- done olurken gerçek test ve acceptance çıktısıyla doldurulur. -->

2026-09-14: `VerifiedMediaIdentityPresentation` film/dizi kompakt etiketini,
optional alan düşümünü, basename fallback'ini ve 0,24 sn ease-out geçiş
kontratını doğrulayan testlerle kapandı. macOS model testleri exact match,
`NoMatch`/`Ambiguous`, typed hata, eski revision ve shutdown sonrası late
sonuç davranışlarını doğruladı. K23 negatif guard'ı hassas değerlerin
Debug/log/kanıt yüzeyine çıkmadığını doğruladı.

Kapılar: `bash scripts/build-macos-app.sh` PASS; gerçek libmpv fixture
testleri dahil `bash scripts/test-macos.sh` PASS (295 test / 36 suite);
`cargo test --workspace`, `cargo fmt --all -- --check`, `cargo clippy
--workspace --all-targets -- -D warnings`, `cargo deny check`, `bash
scripts/test.sh` (6/6), `bash scripts/check-docs.sh` (10/10) ve `git
diff --check` PASS. `cargo deny` yalnız mevcut duplicate crate uyarılarını
raporladı; advisories/bans/licenses/sources geçti.
