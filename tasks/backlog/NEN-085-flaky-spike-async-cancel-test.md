---
id: NEN-085
title: Stabilize the flaky spike-async-cancel completion test
milestone: M1
size: S
state: backlog
closed:
depends_on: []
blocks: []
adr: []
---

# NEN-085 — Stabilize the flaky spike-async-cancel completion test

## Sonuç

`spike-async-cancel::tests::uncancelled_job_completes_and_commits_exactly_once`
artık CI'da tutarlı geçiyor; `handle.join()`'ün döndüğü an `live_jobs()`'un
gerçekten sıfır olduğu garanti.

## Bağlam

`NEN-081`'in tamamen doküman içeren bir commit'i (yalnız `docs/adr/0043-*.md`)
GitHub Actions'ta (`run 34201165481`) `cargo test` adımında kırmızı verdi:

```
thread 'tests::uncancelled_job_completes_and_commits_exactly_once' panicked
assertion `left == right` failed
  left: 1
 right: 0
```

`assert_eq!(live_jobs(), 0)` — `handle.join()` döndükten hemen sonra. Aynı
commit'in **hiçbir kod değişikliği** taşımadığı doğrulandı (diff yalnız bir
ADR dosyası); iş `gh run rerun` ile **aynı** commit üzerinde tekrar
çalıştırıldığında 2m17s'de tamamen yeşil döndü (`ci` job'ı, tüm adımlar dahil
`cargo test`). Yani kusur bir race — muhtemelen job'ın kendi thread'i
`commit_count`/`live_jobs` sayaçlarını `join()`'ün gördüğü sırayla değil,
sonrasında güncelliyor.

Bu, `NEN-081`'in kapsamı dışında bulunmuş, bağımsız bir CI kusuru
(`docs/DECISIONS.md` → Commit politikası: *"CI bağımsız bir kusur gösterirse
yeni backlog task'ı açılır"*).

## Kapsam

- `core/spikes/spike-async-cancel/src/lib.rs`'teki `join()`/`live_jobs()`
  senkronizasyonunu ölçüp yarışı kapatan sıralama (`Ordering`) veya
  senkronizasyon düzeltmesi
- Testi birkaç kez arka arkaya (`--test-threads` ve tek başına) koşturarak
  ölçülü bir doğrulama

## YAPILMAYACAK

- `spike-async-cancel`'i ürün koduna terfi ettirmek — Kural 7, spike kodu
  spike kalır
- Diğer spike testlerini veya başka bir crate'i dokunmak

## Kanıt (DoD)

- [ ] Düzeltmeden önce kusur yerel olarak yeniden üretildi (tekrarlı koşu veya
      hedefli bir gecikme enjeksiyonuyla) — bir kez CI'da görülmüş olmak
      yeterli kanıt değil
- [ ] Düzeltmeden sonra aynı test 20+ ardışık koşuda (`cargo test -p
      spike-async-cancel --lib -- --test-threads=1` ve varsayılan iş
      parçacığı sayısıyla) hep yeşil
- [ ] Negatif kontrol: düzeltme geri alındığında kusur yeniden üretilebiliyor
- [ ] `cargo test --workspace` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
