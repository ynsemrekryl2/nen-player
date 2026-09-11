---
id: NEN-035
title: Identity confidence scoring and candidate ranking
milestone: M6
size: M
state: done
closed: 2026-09-11
depends_on: [NEN-018]
blocks: [NEN-121, NEN-125]
adr: []
---

# NEN-035 — Identity confidence scoring and candidate ranking

## Sonuç

Kanıt katmanları çeliştiğinde veya hiçbiri kesin sonuç vermediğinde adaylar
skorlanır ve sıralanır; **yalnız eşiğin altında kalındığında** kullanıcıya aday
listesi gösterilir.

## Kapsam

- Skor modeli: her kanıt katmanının ağırlığı, çelişki ve teyit etkisi
- Eşik: üstünde sessizce karar, altında aday listesi
- Aday listesi projeksiyonu (UI çizimi değil, model)
- Manuel giriş **aday listesinden sonra** gelir (ADR-0009 Karar 7)

## YAPILMAYACAK

- Teknik ID giriş alanı — **non-goal**; kullanıcıya yalnız başlık/yıl/sezon/bölüm
- Aday listesini varsayılan akış haline getirmek
- Skorları ölçüm olmadan sabitlemek — gerçek aday kümesi M6'da gelir

## Kanıt (DoD)

- [x] Fixture korpusunda aday listesi gösterim oranı **ölçülüyor ve raporlanıyor**
      (ADR-0009 Karar 7'nin hedefi: büyük çoğunlukta hiç gösterilmemesi)
- [x] Çelişen kanıtlarda tanımlı ve test edilmiş sıralama
- [x] Eşik üstünde kullanıcıya hiç sorulmuyor (test)
- [x] Eşik altında aday listesi üretiliyor, manuel giriş ondan sonra

## Kanıt kaydı

Ortam: Apple M5 · arm64 · macOS 27.0 · rustc/cargo 1.98.0.
Tarih: 2026-09-11.

`MediaEvidence::assess_identity` ve `nen_identity::confidence` altında
`ConfidenceScore` (0–100), `RankedIdentityCandidate`, `IdentityAssessment` ve
`IdentityChoice` eklendi. ADR-0009 katman sırası korunuyor; uyumlu başlık ve
koordinatlar birleştiriliyor, çelişkiler ayrı aday olarak kalıyor. Doğrulanmış
hash kimliği çakışma olsa bile otomatik; diğer çakışmalar ve eşikler kullanıcı
seçimine düşüyor. Manuel giriş `choices()` sonucunda her zaman adaylardan sonra.
`resolve()` ve `identity_candidates()` geriye dönük bırakıldı.

`fixtures/media/identity-confidence.tsv` korpusu ve golden raporu ile ölçüm:
10 vaka, 8 otomatik (%80), 1 aday seçimi (%10), 1 manuel (%10), yanlış sessiz
karar 0. Sabit profil ve eşik golden raporda (`identity-confidence.golden`)
korunuyor. Çelişki, eşitlik, teyit, doğrulanmış hash, bilinmeyen giriş ve K23
redaction testleri yeşil.

Doğrulama:

- `cargo test -p nen-identity` — 183 test geçti.
- `cargo test --workspace` — tüm workspace suite'leri geçti.
- `cargo fmt --all -- --check` — çıkış 0.
- `cargo clippy --workspace --all-targets -- -D warnings` — çıkış 0.
- `cargo deny check` — advisories/bans/licenses/sources OK (mevcut duplicate
  crate uyarıları).
- `bash scripts/test.sh` — 4/4 test dosyası geçti.
