---
id: NEN-040
title: check-docs.sh verifies STATUS.md's "Son doğrulama" date is not stale
milestone: M3
size: S
state: backlog
depends_on: [NEN-031]
blocks: []
adr: []
---

# NEN-040 — check-docs.sh verifies STATUS.md's "Son doğrulama" date is not stale

## Sonuç

`docs/STATUS.md`'nin "Son doğrulama" tarihi, en son `done` olan task'ın
tarihinden eski kaldığında `check-docs.sh` bunu yakalar.

## Kapsam

- Yeni denetim (9.): "Son doğrulama" başlığındaki tarih, `tasks/done/`
  içindeki en yeni `date`'ten (veya en yeni dosya mtime'ından, hangisi task
  formatında güvenilir kalıyorsa) eski olamaz
- `scripts/tests/check-docs.test.sh`'e iki senaryo — NEN-031'in kurduğu
  kendi fixture'ını kuran desene uyarak: (a) güncel tarih → geçer, (b) bayat
  tarih → hata mesajıyla yakalanır

## YAPILMAYACAK

- Bloktaki test sayılarının veya komut çıktılarının doğruluğunu denetlemek —
  yalnız **tarih** taze/bayat ayrımı, içerik denetimi değil
- STATUS.md'nin diğer bölümlerini denetlemek

## Kanıt (DoD)

- [ ] Bayat tarihli fixture'da `check-docs.sh` hata veriyor, mesaj hangi
      task'ın daha yeni olduğunu söylüyor
- [ ] Güncel tarihli fixture'da denetim geçiyor
- [ ] Gerçek `docs/STATUS.md` üzerinde çalıştırıldığında yanlış pozitif yok

## Kanıt kaydı

<!-- done olurken doldurulacak -->
