---
name: plan-milestone
description: Bir Nen Player milestone'unu ölçülebilir task'lara böler, dependency graph ve acceptance criteria üretir. Mevcut task'ları yeniden numaralandırmaz, implementasyon yapmaz, kullanıcı onayı bekler.
---

# /plan-milestone

Bir milestone'u **task'lara böl** — kod yazma.

## Girdi

Milestone kimliği (ör. `M5`). Verilmemişse `docs/STATUS.md`'deki mevcut
milestone'un **bir sonrakini** öner ve teyit iste.

## Akış

1. `docs/roadmap.md` → milestone tablosu ve bağımlılıklar.
2. `docs/milestones/M#-*.md` → amaç, kapsam, çıkış kriterleri.
3. `docs/product-spec.md` → milestone'un dayandığı şartname bölümleri.
4. `docs/architecture.md` → etkilenen portlar ve crate'ler.
5. `docs/DECISIONS.md` → bu milestone'u bekleten açık soru veya ertelenmiş karar
   var mı. **Varsa önce onu sor.**
6. `tasks/INDEX.md` → en yüksek mevcut NEN numarası.
7. Task'ları üret: her biri **tek cümlelik sonuç** + ölçülebilir kanıt + açık
   "YAPILMAYACAK" bölümü.
8. Dependency graph çıkar; döngü olmadığını doğrula.
9. Milestone'un çıkış kriterlerini task kanıtlarıyla eşleştir — karşılığı olmayan
   kriter varsa eksik task vardır.
10. **Planı sun, onay bekle.** Onaysız dosya oluşturma.

## Kurallar

- **Mevcut task'ları yeniden numaralandırma.** Yeni task'lar sıradaki numaradan
  başlar; ID'ler asla yeniden kullanılmaz.
- Boyut: `S` / `M` / `L`. **`XL` yasak** — bölünmeden `active/`'e alınamaz.
  Bölme testi: kanıt tek cümleyle ifade edilemiyorsa en az iki task'tır.
- Kanıt tipi task tipine göre seçilir (`docs/testing-strategy.md`). Güvenlik ve
  validation task'larında **negatif test zorunlu**.
- Ölçüm gerektiren task'ta **eşik değil baseline** yaz: fixture boyutu, cihaz,
  build tipi ile birlikte. Ölçmeden pass/fail sınırı konmaz.
- Mimari karar gerekiyorsa ADR öner (`proposed`), task'ın `adr:` alanına ekle.
- **Implementasyon yapma.** Çıktı yalnız task dosyaları ve milestone güncellemesi.
- Onaydan sonra: `bash scripts/task-index.sh` + `bash scripts/check-docs.sh`.
