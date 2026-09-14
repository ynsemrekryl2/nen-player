# NEN-118 kanıt kaydı — Real provider environment wiring

Tarih: 2026-09-14

## DoD kanıtı

| Gereksinim | Kanıt |
|---|---|
| Eksik credential kapısı | `real_provider_environment::missing_real_provider_credential_refuses_before_http_and_leaves_artifacts_empty`: `MissingCredential`, HTTP çağrısı `0`, artifact dizini boş |
| Gerçek provider uçtan uca | `openai_environment_reads_secure_key_and_writes_an_artifact`: fake credential + fake HTTP + OpenAI fixture ile artifact yazıldı |
| Cache identity | `media_hash_is_part_of_environment_cache_identity`: aynı hash cache hit, farklı hash cache miss; `media_hash_helper_uses_nen018_windows` NEN-018 algoritmasını doğruladı |
| K23 negatif | Artifact JSON ve `ArtifactRecord` `Debug` çıktısı credential sentinel içermiyor; FFI guard'ında gerçek job, progress, summary ve tüm tipli varyantlar payload/path sızdırmıyor |
| FFI hata eşlemesi | `new_provider_refusals_map_to_flat_ffi_variants`: `MissingCredential`, `ProviderCapabilityMissing`, `ProviderUnavailable` düz FFI varyantlarına eşleniyor |
| Swift ayarları ve UI | Provider/model UserDefaults restart testi; credential yokken typed Türkçe hata, artifact boşluğu ve seçili subtitle token'ı değişmiyor |

## Çalıştırılan kapılar

```text
cargo test -p nen-ffi --test guard_ffi_translation_debug       PASS (5/5)
cargo test -p nen-app --test real_provider_environment          PASS (4/4)
cargo test --workspace                                      PASS
cargo fmt --all -- --check                                  PASS
cargo clippy --workspace --all-targets -- -D warnings       PASS
cargo deny check                                            PASS
bash scripts/test-macos.sh                                  PASS (284 tests)
bash scripts/check-docs.sh                                  PASS (10/10)
git diff --check                                            PASS
```

`cargo deny` yalnız mevcut duplicate `hashbrown` ve `syn` uyarılarını raporladı;
advisories, bans, licenses ve sources kontrolleri geçti. Tüm provider testleri
deterministic fake credential/HTTP fixture'larıyla çalıştı; gerçek ağa çıkılmadı.
