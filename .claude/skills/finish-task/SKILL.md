---
name: finish-task
description: Nen Player'da aktif task'ı kanıtla kapatır, commit'ler, origin/main'e push eder ve CI sonucunu izler. Eksik kanıtta done/commit/push yapmaz, başka task'a başlamaz.
---

# /finish-task

Aktif task'ı **kanıtla kapat** — eksikse kapatma.

## Akış

1. `tasks/active/` içindeki task'ı oku. Boşsa dur ve bildir.
2. "Kanıt (DoD)" bölümündeki **her maddeyi tek tek** denetle.
3. İlgili testleri çalıştır ve **gerçek çıktıyı** yakala.
4. Güvenlik/validation task'ıysa **negatif testlerin de** koştuğunu doğrula.
5. Ölçüm içeren task'ta baseline'ın **bağlamıyla** kaydedildiğini doğrula:
   fixture boyutu · cihaz · OS/toolchain · debug/release build. Bağlamsız sayı
   kanıt değildir.
6. `adr:` alanı doluysa, ADR'lerin `status: accepted` olduğunu doğrula.
7. **Eksik kanıt varsa `done` yapma.** Neyin eksik olduğunu ve nasıl
   tamamlanacağını söyle, dur.
8. Tümü karşılanıyorsa "Kanıt kaydı" bölümünü **gerçek çıktıyla** doldur —
   "test edildi" / "çalışıyor" yeterli değil.
9. `state: done` yap, dosyayı `tasks/done/` altına taşı.
10. `bash scripts/task-index.sh` çalıştır.
11. `docs/STATUS.md`'yi güncelle: son tamamlanan, sıradaki READY, toolchain
    değiştiyse o satır, çözülen blocker'lar.
12. Milestone'un son task'ıysa `docs/milestones/M#-*.md` içine **retro** yaz.
13. Karar verildiyse `docs/DECISIONS.md`'ye işle.
14. `bash scripts/check-docs.sh` ile doğrula — çıkış 0 olmalı.
15. **Commit at** — CLAUDE.md → "Commit politikası". Kapanış kendi commit'ini
    alır; bu oturumda kabul edilmiş bir ADR varsa o ayrı commit olur.
    Ön koşul: adım 14 çıkış 0 vermiş olmalı.
16. Commit'i `git push origin main` ile gönder ve GitHub CI sonucunu izle.
    Push/CI başarısızsa force, pull, rebase veya geçmiş değiştirme yapma;
    **başka task'a başlama.** Sebebi ve yerel commit durumunu raporla.
17. CI yeşilse **başka task'a başlamadan** dur ve kapanışı raporla.

## Sınırlar

- Kanıtı üretmeden yazma; testi çalıştırmadan "geçiyor" deme.
- Kapsam dışı düzeltme yapma — yeni backlog task'ı öner.
- `tasks/INDEX.md`'yi elle düzenleme; script üretir.
