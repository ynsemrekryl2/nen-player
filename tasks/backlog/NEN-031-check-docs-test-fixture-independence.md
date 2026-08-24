---
id: NEN-031
title: Make check-docs test fixture independent of live repo state
milestone: M1
size: S
state: backlog
depends_on: []
blocks: []
adr: []
---

# NEN-031 — Make check-docs test fixture independent of live repo state

## Sonuç

`bash scripts/test.sh`, `tasks/active/` dolu olsun boş olsun aynı sonucu verir;
denetim 8'in her iki dalı (ready listesi dolu / boş) ayrı ayrı ve deterministik
olarak doğrulanmıştır.

## Bağlam — neden açıldı

`scripts/tests/check-docs.test.sh` (NEN-030) `READY` değişkenini **canlı**
`tasks/INDEX.md`'den türetiyor:

```sh
READY="$(awk '/^## Sıradaki uygun task/{f=1;next} f' "$INDEX" ...)"
```

Ready listesi boşaldığı anda — yani **bir task `active` olduğu anda**, sistemin
normal çalışma hâlinde — üç doğrulama kırılıyor:

| Test | Neden kırılıyor |
|---|---|
| T1 | Boş satır yazılıyor; check-docs "ready listesi boş…" diyor, test "…uyumlu" arıyor |
| T3 | `henüz belirlenmedi` (ID yok) boş INDEX listesiyle **uyuşuyor** → exit 0, test 1 bekliyor |
| T6 | T1 ile aynı |

NEN-007 `active/`'e alındığında gözlendi (2026-08-24). Test yanlış davranışı
değil, **fixture'ın canlı repo durumuna bağımlılığını** yakalıyor — denetim 8'in
kendisi doğru çalışıyor (`bash scripts/check-docs.sh` → exit 0).

## Kapsam

- Test kendi task fixture'ını üretsin: geçici repo kopyasında `tasks/` içeriği
  test tarafından kurulsun, canlı `INDEX.md`'den okunmasın
- **Her iki dal** ayrı senaryo olarak doğrulansın:
  - ready listesi **dolu** → STATUS uyumlu geçer / uyumsuz hata verir
  - ready listesi **boş** (active task var) → STATUS task listelemezse geçer,
    listelerse hata verir
- T7'nin (gerçek repo dosyalarına dokunmama) korunması

## YAPILMAYACAK

- `scripts/check-docs.sh` denetim 8 mantığının değiştirilmesi — davranış doğru
- `scripts/doctor.sh` ve `doctor.test.sh` — ikisi de geçiyor
- Yeni denetim eklenmesi

## Kanıt (DoD)

- [ ] `bash scripts/test.sh` geçiyor — `tasks/active/` **dolu**ken
- [ ] `bash scripts/test.sh` geçiyor — `tasks/active/` **boş**ken
- [ ] Boş-ready dalı için negatif test: STATUS bir task listeliyorken INDEX
      listelemiyor → check-docs exit 1
- [ ] T7 hâlâ geçiyor (gerçek `STATUS.md` / `INDEX.md` parmak izi değişmiyor)

## Kanıt kaydı

<!-- Task done olurken GERÇEK çıktı ile doldurulur. -->
