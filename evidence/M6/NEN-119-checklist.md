# NEN-119 kanıt kaydı — OpenSubtitles integration boundaries

Tarih: 2026-09-14

## Karar kanıtı

| Gereksinim | Kanıt |
|---|---|
| ADR accepted | [ADR-0021](../../docs/adr/0021-opensubtitles-integration-boundaries.md) kullanıcı onayıyla `accepted` durumunda; onay kaydı ADR metninde. |
| Search/download boundaries | ADR-0021 resmi metadata aramasını, hash → verified identity sırasını ve yalnız açık seçimde download çağrısını belirler. |
| Public/private identity | Public katalog kimliği opaque subtitle ID'dir; private `file_id`, hash ve filename metadata'sı public/FFI/persistence/log/Debug yüzeylerinden ayrılır. |
| Lazy catalog/selection | Arama yalnız ilk 16 metadata adayını üretir; aday dosyası indirilmez ve otomatik seçim yapılmaz. |
| Security numbers | Redirect en fazla 5 hop, altyazı indirmesi 10 MiB, metadata response 1 MiB; archive ve beklenmeyen content-type reddedilir. |
| Cross-doc consistency | `docs/adr/README.md`, `docs/DECISIONS.md`, `docs/security-policy.md`, ADR-0035 ve ADR-0010 ADR-0021 ile hizalıdır. |

## Çalıştırılan kapılar

```text
bash scripts/check-docs.sh PASS (10/10)
git diff --check PASS
```

ADR-only task; provider canlı ağı veya Rust/Swift kod kapıları bu task'ın
kapsamında değildir.
