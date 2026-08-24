---
id: NEN-002
title: Task system and index generator
milestone: M0
size: M
state: done
depends_on: [NEN-001]
blocks: [NEN-004]
adr: []
---

# NEN-002 — Task system and index generator

## Sonuç

Task'lar tek bir dosya formatında yaşar, `tasks/INDEX.md` bu dosyalardan
üretilir, ve tutarsızlık (kırık bağımlılık, kanıtsız `done`, birden fazla aktif
task, bayat index) script ile yakalanır.

## Kapsam

- `tasks/_template.md` — frontmatter + zorunlu 5 bölüm
- `tasks/README.md` — format, boyut ölçeği, durum akışı, denetimler
- `scripts/task-index.sh` — frontmatter'dan `INDEX.md` üretir; `--check` bayrağı
- `scripts/check-docs.sh` — 7 değişmez denetimi
- İlk 28 task dosyası (`NEN-001` … `NEN-028`)

## YAPILMAYACAK

- GitHub Issues senkronizasyonu — repo içi markdown kaynak gerçek olarak seçildi
- M4 sonrası milestone'ların task kırılımı — her milestone kendi task'larını bir
  önceki kapanırken üretir
- CI'a bağlama → NEN-005

## Kanıt (DoD)

- [x] `scripts/task-index.sh` 28 task dosyasını okuyup `INDEX.md` üretiyor
- [x] `scripts/task-index.sh --check` bayat index'te çıkış kodu 1 veriyor
- [x] `check-docs.sh` temiz repoda çıkış kodu 0 veriyor
- [x] Negatif: kırık `depends_on` → hata + çıkış kodu 1
- [x] Negatif: ikinci aktif task → hata + çıkış kodu 1
- [x] Negatif: kanıt kaydı boş `done` task → hata + çıkış kodu 1

## Kanıt kaydı

Tarih: 2026-08-24 · Kanıt tipi: script çıktısı + 4 negatif test

Pozitif:

```
$ bash scripts/task-index.sh
Üretildi: tasks/INDEX.md
# → Toplam 28 task · done 3 · active 0 · blocked 0 · backlog 25
# → "Sıradaki uygun task'lar" bölümü doğru tek sonucu verdi: NEN-004

$ bash scripts/task-index.sh --check
OK: tasks/INDEX.md güncel.
```

Negatif testler (hepsi uygulandı ve geri alındı):

| # | Senaryo | Sonuç | Çıkış kodu |
|---|---|---|---|
| 1 | NEN-014'ün `depends_on`'una `NEN-999` eklendi | `HATA tasks/backlog/NEN-014-webvtt-writer.md: depends_on içinde 'NEN-999' var ama böyle bir task yok.` | 1 |
| 2 | `tasks/active/` altına iki task konuldu | `HATA tasks/active/ içinde 2 task var, en fazla 1 olabilir` + iki dosya adı listelendi | 1 |
| 3 | `INDEX.md`'ye fazladan satır eklendi | `HATA: tasks/INDEX.md bayat.` + diff çıktısı | 1 |
| 4 | ADR-0001 `status: proposed` yapıldı | `HATA tasks/done/NEN-003-adr-system.md: done, ama ADR-0001 durumu 'proposed'` | 1 |
| 5 | Bu üç done task'ın kanıt kaydı boşken | `HATA ... done ama 'Kanıt kaydı' bölümü boş.` ×3 | 1 |

Her testten sonra `git status` temiz kaldı; `check-docs.sh` yeniden 0 döndü.

Not: NEN-005 (CI) ve NEN-006 (redaction) M0'dan **M1'e taşındı** — ikisi de Rust
crate'i gerektirdiği için `nen-ffi` iskeleti (NEN-007) olmadan çalıştırılamıyor.
