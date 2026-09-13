---
id: NEN-127
title: STATUS.md token diet and size guard
milestone: M6
size: S
state: done
closed: 2026-09-13
depends_on: []
blocks: []
adr: []
---

# NEN-127 — STATUS.md token diyeti ve boyut kapısı

## Sonuç

`docs/STATUS.md` yalnız bugünü anlatır (≤ 150 satır); geçmiş kapanış anlatısı
`docs/history/` arşivinde ve `tasks/done/`'da durur; `check-docs.sh` boyutu
mekanik olarak zorlar.

## Bağlam

Ölçüm (2026-09-13): `docs/STATUS.md` **225 KB / 3368 satır (~56k token)**.
CLAUDE.md "Oturum başlangıcı" adım 1 ve `/next-task` adım 2 bu dosyayı her
oturumda okutuyor. Dosya "yalnız doğrulanmış bugünü anlatır" diyor ama
`/finish-task` adım 11 her kapanış paragrafını **ekliyor**, öncekini
silmiyor: "Nerede duruyoruz" 2975 satır (106 task'ın kapanış anlatısı),
"Son doğrulama" + "Toolchain kapısı geçmiş kaydı" ~210 satır, "Repository'nin
gerçek durumu" 110 satırlık dosya envanteri. Hepsi zaten `tasks/done/NEN-*.md`
Kanıt kaydı ve git geçmişinde var — çift kayıt.

`check-docs.sh`'ın STATUS'tan beklediği yalnız iki şey: `**Sıradaki READY**`
satırı (denetim 8) ve `## Son doğrulama` altında en yeni done task'ın
`closed` tarihinden geri olmayan bir tarih (denetim 9). Geçmiş anlatı hiçbir
denetimde kullanılmıyor.

## Kapsam

- `docs/STATUS.md`'yi yeniden yaz: Nerede duruyoruz tablosu + yalnız **son**
  kapanışın kısa özeti · Toolchain tablosu · açık blocker'lar · kullanıcı
  kararı bekleyenler · yalnız son "Son doğrulama" girdisi · güncelleme kuralı
- Çıkarılan içeriği (eski kapanış paragrafları, eski doğrulama girdileri,
  toolchain geçmişi, repository envanteri) satırı satırına
  `docs/history/status-archive-2026-09.md`'ye taşı — kayıp yok
- "Ekle değil değiştir" kuralı: STATUS "Bu dosyayı kim günceller" bölümü ve
  `.claude/skills/finish-task/SKILL.md` adım 11 — kapanışta önceki kapanış
  özeti silinir
- `scripts/check-docs.sh` **denetim 10**: `docs/STATUS.md` > 150 satır →
  `err`; `scripts/tests/check-docs.test.sh`'a negatif vaka
- `tasks/README.md` denetim listesine 10. madde

## YAPILMAYACAK

- `tasks/INDEX.md`'yi bölmek → `NEN-128`
- `CLAUDE.md` içeriğini değiştirmek (≈1.9k token, sorun değil)
- Kod (`core/`, `platforms/`) — dokunulmaz

## Kanıt (DoD)

- [x] `wc -l docs/STATUS.md` ≤ 150; `wc -c` ≤ ~10 KB (önce 225 KB)
- [x] Eski STATUS satır sayısı ≈ yeni STATUS + arşiv (içerik kaybı yok)
- [x] `bash scripts/check-docs.sh` çıkış 0; denetim 10 çıktıda görünüyor
- [x] Negatif test: 151+ satırlık STATUS fixture'ı ile `check-docs.sh` çıkış 1
      (`scripts/tests/check-docs.test.sh`)
- [x] `bash scripts/test.sh` yeşil

## Kanıt kaydı

2026-09-13, bu makine (macOS 27.0, bash 3.2/zsh, python3).

**Boyut.** `wc -l -c docs/STATUS.md` (kapanış öncesi, NEN-115 özetiyle):

```
      98    5055 docs/STATUS.md
    3332  223240 docs/history/status-archive-2026-09.md
```

Önce: `225418 bytes / 3368 satır` (~56k token). Sonra: ~5 KB (~1.3k token).
Oturum başına kazanç ≈ 54k token.

**İçerik kaybı yok.** Eski STATUS'un 19–3363 satır aralığındaki (yeniden
yazılan başlık tablosu ve "Bu dosyayı kim günceller" bölümü dışındaki) tüm
boş olmayan satırlar `sort -u` + `comm -23` ile yeni STATUS ∪ arşivde arandı:

```
missing: 0
```

**check-docs.sh** (`bash scripts/check-docs.sh`, çıkış 0):

```
== 9. STATUS.md son doğrulama tarihi ==
  ok  Son doğrulama tarihi güncel (2026-09-11; en yeni done: NEN-115 2026-09-11)
== 10. STATUS.md boyutu ==
  ok  STATUS.md 98 satır (sınır 150)

SONUÇ: tüm denetimler geçti.
```

**Negatif test** (`scripts/tests/check-docs.test.sh` T18/T19, yeni bölüm H):

```
  T18: 150 satırlık STATUS geçiyor
    ok   T18 sınırdaki STATUS → geçiyor
  T19: 151 satırlık STATUS yakalanıyor (negatif)
    ok   T19 sınırı aşan STATUS → hata
  T7: gerçek repo dosyaları değişmedi
    ok   T7 gerçek STATUS.md / INDEX.md / task dosyaları / script'ler değişmedi
```

T19'un yakaladığı gerçek hata satırı: `HATA  docs/STATUS.md 151 satır; sınır
150.` Kapı sağır değil: dolgu yardımcısının ilk sürümü fazladan bir boş satır
ekleyince T18 153 satırla kırmızı oldu; yardımcı düzeltildi, sınır doğru.

**Shell testleri** (`bash scripts/test.sh`): `SONUÇ: 4 test dosyasının hepsi
geçti.`

**Kural değişikliği.** `.claude/skills/finish-task/SKILL.md` adım 11 ve
`docs/STATUS.md` "Bu dosyayı kim günceller": ekleme değil değiştirme;
`tasks/README.md` denetim listesine 11. madde. `NEN-128` (INDEX bölme)
backlog'a açıldı.
