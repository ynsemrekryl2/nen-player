---
name: next-task
description: Nen Player'da sıradaki task'ı seçer, bağlamını okur ve kısa bir execution planı sunar. Kullanıcı onayı olmadan implementasyona geçmez, tek task uygular, push atmaz.
---

# /next-task

Sıradaki task'ı **seç, bağlamını oku, planını sun, onay bekle.**

## Akış

1. `AGENTS.md` oku — değişmez kurallar.
2. `docs/STATUS.md` oku — milestone, aktif task, blocker'lar, bekleyen kararlar.
3. `tasks/INDEX.md` ve `tasks/active/` kontrol et.
4. **Aktif task varsa onu ele al.** Yeni task seçme.
5. Aktif task yoksa: `depends_on`'ı tamamlanmış ilk uygun backlog task'ını seç
   (`INDEX.md` → "Sıradaki uygun task'lar"). Birden fazlaysa, toolchain
   gerektirmeyeni tercih et ve seçimi gerekçelendir.
6. Task'ın bağlamını oku: ilgili `docs/milestones/M#-*.md`, `docs/product-spec.md`
   içindeki ilgili bölüm, `adr:` alanındaki ADR'ler.
7. `git status` kontrol et — beklenmeyen değişiklik var mı.
8. **Önce kısa bir execution planı sun** (ne yapılacak, hangi dosyalar, kanıt ne
   olacak).
9. Mimari karar veya kullanıcı kararı gerekiyorsa **dur ve sor**. ADR gerekiyorsa
   önce ADR'yi `proposed` olarak yaz.
10. **Kullanıcı onayı olmadan implementasyona geçme.**
11. Onay gelince **yalnız tek task** uygula.
12. **Test kanıtı olmadan `done` yapma.** Kanıt tipi task tipine göre —
    `docs/testing-strategy.md` → "Kanıt formatı".
13. Task kapanınca **commit at** (AGENTS.md → "Commit politikası"): kanıt dolu,
    testler yeşil, `check-docs.sh` çıkış 0. **Push yok**, geçmiş değiştirme yok.
14. `bash scripts/task-index.sh` çalıştır, `docs/STATUS.md`'yi güncelle,
    `bash scripts/check-docs.sh` ile doğrula.
15. **Sıradaki task'a otomatik geçme.** Dur ve raporla.

## Sınırlar

- Aynı anda `tasks/active/` içinde en fazla bir task.
- Task dosyası olmayan kod yazılmaz.
- Yol üstünde görülen alakasız iyileştirme → yeni backlog task'ı, mevcut task'a
  eklenmez.
- Teknoloji seçimleri ilgili ADR kabul edilene kadar **aday**dır; belgelerde
  kesin karar gibi yazılmaz.
