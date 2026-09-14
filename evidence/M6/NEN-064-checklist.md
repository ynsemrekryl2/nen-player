# NEN-064 kanıt kaydı — Oynatıcı kromunda doğrulanmış medya kimliği

Tarih: 2026-09-14

## DoD kanıtı

| Gereksinim | Kanıt |
|---|---|
| Exact match ve kompakt etiket | macOS presentation testleri film/dizi başlığı, yıl, sezon/bölüm ve eksik alanları; PlayerModel testi doğrulanmış sonucu kapsıyor. |
| Fallback ve playback bağımsızlığı | `NoMatch`, `Ambiguous` ve typed hata sonuçlarında basename korunuyor; lookup worker'ı playback akışını kesmiyor. |
| Revision/shutdown guard | Eski medya revision'ına ait ve shutdown sonrasında gelen geç sonuçlar model state'ine uygulanmıyor. |
| K23 redaction | Negatif guard; yol, query, hash, private ID ve özel filename değerlerinin Debug/log/kanıt yüzeyine çıkmadığını doğruluyor. |
| Chrome geçişi | `PlayerRootView` doğrulanmış kimliği basename yerine kompakt başlık olarak gösteriyor; 0,24 sn ease-out transition kontratı ve gerçek app build'i geçti. |

## Çalıştırılan kapılar

```text
bash scripts/build-macos-app.sh                             PASS
bash scripts/test-macos.sh                                  PASS (295 tests / 36 suites)
cargo test --workspace                                      PASS
cargo fmt --all -- --check                                  PASS
cargo clippy --workspace --all-targets -- -D warnings       PASS
cargo deny check                                            PASS
bash scripts/test.sh                                        PASS (6/6)
bash scripts/check-docs.sh                                  PASS (10/10)
git diff --check                                            PASS
```

Testler deterministic fake/provider fixture'larıyla çalıştı; canlı provider
ağı kullanılmadı.
