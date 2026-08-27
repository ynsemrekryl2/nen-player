---
name: next-task
description: Nen Player'da sıradaki task'ı seçer, onaydan sonra tek task uygular, kanıtla kapatır, commit/push eder ve CI sonucunu izler.
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
13. Kanıt kaydını doldur, task'ı `done` yap, `bash scripts/task-index.sh`
    çalıştır ve `docs/STATUS.md`'yi güncelle.
14. `bash scripts/check-docs.sh` çıkış 0 olduktan sonra **commit at**
    (AGENTS.md → "Commit politikası").
15. Commit'i `git push origin main` ile gönder ve GitHub CI sonucunu izle.
    Push/CI başarısızsa force, pull, rebase veya geçmiş değiştirme yapma.
16. **Sıradaki task'a otomatik geçme.** CI sonucuyla birlikte dur ve raporla.

## Sınırlar

- Aynı anda `tasks/active/` içinde en fazla bir task.
- Task dosyası olmayan kod yazılmaz.
- Yol üstünde görülen alakasız iyileştirme → yeni backlog task'ı, mevcut task'a
  eklenmez.
- Teknoloji seçimleri ilgili ADR kabul edilene kadar **aday**dır; belgelerde
  kesin karar gibi yazılmaz.
