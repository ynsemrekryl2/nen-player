# NEN-034 kanıt kaydı

Tarih: 2026-09-14

## Kabul ve kapsam

- ADR-0046 kabul edildi: AI filename normalization varsayılan olarak kapalı,
  medya-başına açık izinle ve yalnız sanitize basename stem'iyle çalışıyor.
- Normalization cevabı untrusted suggestion olarak kalıyor; deterministic
  identity/evidence otomatik seçilmiyor.

## Davranış kanıtı

- `Unknown` deterministic sonuç + izin + fake provider → typed title/year
  suggestion.
- Deterministic sonuç usable olduğunda normalizer çağrısı: **0**.
- İzin reddedildiğinde HTTP çağrısı: **0**.
- Remote URL path hint'i normalizer request'ine taşınmıyor; yalnız local
  basename stem'i kullanılıyor.
- OpenAI ve OpenRouter structured-output yolları, credential/HTTP/retry
  sınırları ve payload'sız hata eşlemesiyle doğrulandı; OpenRouter preflight
  cache'i korunuyor.
- Request, result, app outcome ve provider hata `Debug`/`Display` çıktıları
  filename, provider response ve credential değerlerini açığa çıkarmıyor.

## Komut kanıtı

- `cargo test --workspace --all-targets` — geçti.
- OpenAI/OpenRouter adapter integration suite — **26/26** geçti.
- Seçili-provider app suite — **3/3** geçti.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — geçti.
- `cargo deny check` — advisories, bans, licenses, sources: geçti.
- `bash scripts/test.sh` — **6/6** geçti.
- `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
  `git diff --check` — geçti.
