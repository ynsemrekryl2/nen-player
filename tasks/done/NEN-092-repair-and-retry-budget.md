---
id: NEN-092
title: Targeted repair and full-block retry budget
milestone: M5
size: M
state: done
closed: 2026-09-08
depends_on: [NEN-091]
blocks: [NEN-093]
adr: []
---

# NEN-092 — Targeted repair and full-block retry budget

## Sonuç

Doğrulamadan geçemeyen bir blok en fazla **iki targeted repair** ve **bir
full-block retry** ile onarılmaya çalışılıyor; bütçe tükenirse blok başarısız
olur ve yarım çıktı üretmez.

## Bağlam

Şartname §10 bütçeyi sayıyla veriyor: "en fazla **iki targeted repair**, en
fazla **bir full-block retry**". ADR-0016 hangi ihlalin targeted repair'e uygun
olduğunu kararlaştırır (`NEN-091`).

Bütçenin asıl işlevi maliyet değil **sonlanma garantisi**: onarım döngüsünün
provider'ı sınırsız çağırmaması, M5'in "yarım çıktı yayınlanmaz" kriterinin ön
koşulu.

## Kapsam

- Targeted repair isteğinin kurulması (yalnız ihlal eden cue'lar)
- Bütçe sayacı ve tükenme davranışı
- Full-block retry'ın targeted repair'den sonra tek sefer denenmesi
- Blok başarısızlığının tipli hata olarak yukarı taşınması

## YAPILMAYACAK

- Ağ kaynaklı transient retry — bu provider portunun işi (`NEN-090`), doğrulama
  bütçesiyle karıştırılmaz
- Bütçe sayılarının kullanıcı ayarı yapılması — şartname sabitliyor
- Checkpoint ve iptal — `NEN-093`

## Kanıt (DoD)

- [x] Unit: ilk denemede geçen blok hiç repair çağrısı yapmıyor (sayaç 0)
- [x] Unit: tek ihlal eden cue için targeted repair yalnız o cue'yu istiyor
- [x] Unit: iki başarısız targeted repair'den sonra tam bir full-block retry deneniyor
- [x] Negatif: bütçe tükendiğinde blok tipli hata ile başarısız oluyor ve **kısmi sonuç teslim etmiyor**
- [x] Negatif: sürekli bozuk provider ile çağrı üst sınırı ölçülüyor; döngü 4 çağrıda sonlanıyor
- [x] Guard: repair isteği, provider cevabı ve hata yüzeyleri cue metni loglamıyor (K23 #4, #5)

## Kanıt kaydı

`nen-translate::repair::translate_block_with_repair`, ilk cevabı
`validate_block` ile kontrol ediyor; `repair_cue_ids` yalnız beklenen ve
onarılabilir cue'lara daraltılmış targeted istek üretiyor. Kabul edilmiş
kısmi cue'lar yalnız crate içinde birleştiriliyor. İki targeted deneme sonrası
veya repair kümesi boş olduğunda özgün tam istekle tek full-block retry yapılıyor;
sonuç yine geçersizse yalnız tipli `ValidationBudgetExhausted` hatası dönüyor.
Provider hataları validation bütçesinden bağımsız olarak doğrudan taşınıyor.

Targeted isteklerin daha küçük progress toplamları için `TranslationCall::fork`
eklendi; fork cancellation durumunu ve sink'i paylaşırken her provider
denemesinde progress dizisini yeniden başlatıyor.

Kanıt:

- `valid_initial_response_uses_no_repair`: geçerli ilk cevap tek çağrıda,
  repair çağrısı olmadan tamamlandı.
- `targeted_repair_requests_only_the_missing_cue_and_merges_it`: ikinci istek
  yalnız `CueId(2)` taşıdı; source/target dil, context cue'ları ve context
  terimleri korundu; birleşim tam doğrulanmış blok üretti.
- `unknown_only_response_skips_targeted_repair_and_uses_full_retry`:
  repair kümesi boş olduğunda targeted deneme yapılmadan tam retry kullanıldı.
- `exhausted_budget_is_bounded_and_never_returns_partial_output`: sürekli
  bozuk deterministic provider çağrı dizisi `full → targeted → targeted → full`
  oldu; toplam **4 çağrı** sonrasında tipli hata ve kısmi çıktı yok.
- `provider_failure_is_not_retried_as_validation`: transient provider hatası
  tek çağrıda aynı hata sınıfıyla taşındı.
- `repair_error_surfaces_never_leak_translated_text` ve
  `fork_resets_progress_sequence_but_shares_cancellation`: yeni hata/istek/
  cevap yüzeyleri sentinel metin sızdırmadı; retry fork cancellation'ı
  paylaştı ve progress toplamını güvenle yeniledi.

Doğrulama çıktıları:

- `cargo test -p nen-translate`: **38 passed, 0 failed** (30 unit, 1 golden,
  3 context guard, 4 validation guard).
- `cargo test --workspace --quiet`: **712 passed, 0 failed, 1 ignored**.
- `cargo fmt --all -- --check`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo deny check`: advisories/bans/licenses/sources ok.
- `bash scripts/test.sh`: **4/4** shell test dosyası geçti.
- `bash scripts/check-docs.sh`: 9/9 denetim geçti.

Yeni dış bağımlılık, provider ağı, fixture, FFI veya macOS değişikliği yok.
