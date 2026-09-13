---
id: NEN-128
title: Split the generated task index into open and archive tables
milestone: M6
size: S
state: backlog
closed:
depends_on: [NEN-127]
blocks: []
adr: []
---

# NEN-128 — Üretilen task index'ini açık/arşiv olarak böl

## Sonuç

`tasks/INDEX.md` yalnız açık task'ları ve milestone başına `done x/y`
özetini taşır; done tabloları üretilen `tasks/INDEX-done.md`'de durur.

## Bağlam

`NEN-127` ölçümünde `tasks/INDEX.md` 18 KB (~4.6k token); satırların 106'sı
done task. CLAUDE.md adım 2 bu dosyayı her oturumda okutuyor. STATUS diyeti
(~54k) yanında kazanç küçük (~3.5k) olduğu için ayrı task.

## Kapsam

- `scripts/task-index.sh` iki dosya üretir: `INDEX.md` (backlog/active/
  blocked satırları + milestone başına done sayısı) ve `INDEX-done.md`
  (done + canceled tabloları)
- `scripts/check-docs.sh` denetim 8 (READY listesi) ve index tazelik
  denetimi iki dosyayı da kapsar
- `tasks/README.md` ve CLAUDE.md'deki `INDEX.md` açıklaması güncellenir

## YAPILMAYACAK

- Task dosyalarının içeriğine dokunmak
- INDEX'i elle düzenlemek (Kural 9)

## Kanıt (DoD)

- [ ] `wc -c tasks/INDEX.md` ≤ ~6 KB
- [ ] `bash scripts/task-index.sh` idempotent (ikinci koşu diff üretmez)
- [ ] `bash scripts/check-docs.sh` ve `bash scripts/test.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
