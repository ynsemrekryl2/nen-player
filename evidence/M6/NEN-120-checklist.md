# NEN-120 kanıt kaydı — Verified identity lookup app akışı

Tarih: 2026-09-14

## DoD kanıtı

| Gereksinim | Kanıt |
|---|---|
| `nen-app` exact match ve revision guard | `identity` unit testleri fake credential/provider ile eşleşmeyi evidence'a uygular; macOS `staleIdentityResultIsDiscarded` eski worker cevabının yeni medyaya yazılmadığını doğrular. |
| Credential/hash negatifleri | `missing_credential_does_not_call_lookup` ve `missing_hash_does_not_read_or_call_provider`; FFI `missing_credential_is_typed_and_does_not_call_http` ile `missing_hash_returns_before_credential_or_http_access`. |
| FFI sonucu ve K23 | `nen-ffi/tests/identity_lookup.rs` exact match eşlemesini ve identity/error Debug çıktılarında key/hash sentinel bulunmadığını doğrular. |
| Playback bağımsızlığı | macOS `identityTransportErrorDoesNotAffectPlayback`: lookup hatası sonrası fake session playback'e devam eder, fatal state oluşmaz. |
| Apple binding | `bash scripts/build-apple.sh` geçti; generated Swift'te `FfiVerifiedMediaIdentity` ve `lookupVerifiedIdentityByHash` mevcut. |

## Çalıştırılan kapılar

```text
cargo test --workspace                                      PASS
cargo fmt --all -- --check                                  PASS
cargo clippy --workspace --all-targets -- -D warnings       PASS
cargo deny check                                            PASS
bash scripts/build-apple.sh                                 PASS
bash scripts/test-macos.sh                                  PASS (287 tests / 35 suites)
bash scripts/test.sh                                        PASS (6/6)
bash scripts/check-docs.sh                                  PASS (10/10)
git diff --check                                            PASS
```

Tüm provider testleri deterministic fake credential/HTTP fixture'larıyla
çalıştı; canlı provider ağı kullanılmadı.
