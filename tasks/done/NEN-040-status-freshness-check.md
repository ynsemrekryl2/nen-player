---
id: NEN-040
title: check-docs.sh verifies STATUS.md's "Son doğrulama" date is not stale
milestone: M3
size: S
state: done
closed: 2026-09-06
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

- [x] Bayat tarihli fixture'da `check-docs.sh` hata veriyor, mesaj hangi
      task'ın daha yeni olduğunu söylüyor
- [x] Güncel tarihli fixture'da denetim geçiyor
- [x] Gerçek `docs/STATUS.md` üzerinde çalıştırıldığında yanlış pozitif yok

## Kanıt kaydı

`check-docs.sh` denetim 9, `## Son doğrulama` bölümündeki ISO tarihi Git
committer tarihine göre en yeni done task ile karşılaştırıyor. Checkout zamanı
mtime olarak kullanılmıyor; hata mesajı daha yeni task ID'sini ve iki tarihi
birlikte veriyor.

`scripts/tests/check-docs.test.sh`: **13 senaryo / 0 failure**. T12 eşit tarihli
fixture'ı geçiriyor; T13 `2026-09-04` tarihini `NEN-901 (2026-09-05) daha yeni`
mesajıyla reddediyor. Fixture sabit tarihli geçici Git deposu kuruyor ve gerçek
repo dosyalarının parmak izini koruyor.

`bash scripts/test.sh`: **2 test dosyası geçti**. Gerçek
`bash scripts/check-docs.sh`: **9 denetim geçti**, Son doğrulama
`2026-09-06`, en yeni commit'li done task `NEN-074 2026-09-06`; yanlış pozitif
yok. `bash scripts/task-index.sh --check`: **OK**.
