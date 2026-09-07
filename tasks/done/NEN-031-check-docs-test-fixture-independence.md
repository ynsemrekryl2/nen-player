---
id: NEN-031
title: Make check-docs test fixture independent of live repo state
milestone: M1
size: S
state: done
closed: 2026-08-24
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

- [x] `bash scripts/test.sh` geçiyor — `tasks/active/` **dolu**ken
- [x] `bash scripts/test.sh` geçiyor — `tasks/active/` **boş**ken
- [x] Boş-ready dalı için negatif test: STATUS bir task listeliyorken INDEX
      listelemiyor → check-docs exit 1
- [x] T7 hâlâ geçiyor (gerçek `STATUS.md` / `INDEX.md` parmak izi değişmiyor)

## Kanıt kaydı

Kanıt tipi: **tooling → script çıktısı** (`docs/testing-strategy.md` → "Kanıt
formatı — task tipine göre"). 2026-08-24, macOS 27.0 · arm64.

### Bağımsızlığın mekanik kanıtı — eski/yeni test, aynı repo durumunda

Kırılma koşulu (**ready listesinin boşalması**) sahte bir repo'da üretildi: her
backlog task'ının `depends_on`'una active bir task eklendi → `task-index.sh`
ready listesini `_(yok — …)_` olarak üretti. `check-docs.sh`'ın kendisi bu
durumda **doğru** çalışıyor:

```
$ bash <sahte-repo>/scripts/check-docs.sh
  ok  ready listesi boş, STATUS.md de task listelemiyor
SONUÇ: tüm denetimler geçti.               → exit 0
```

Aynı sahte repo'da iki test sürümü:

```
$ bash <sahte-repo>/scripts/tests/check-docs.test.sh   # ESKİ (92fdbd0)
    FAIL T1 uyumlu → geçiyor — çıktıda 'STATUS.md ready listesi INDEX ile uyumlu' yok
    FAIL T3 boş liste → hata → beklenen exit 1, gelen 0
    FAIL T6 düzeltme sonrası geçiyor — çıktıda '…INDEX ile uyumlu' yok
                                           → exit 1

$ bash <sahte-repo>/scripts/tests/check-docs.test.sh   # YENİ
  11 doğrulamanın hepsi ok, FAIL yok       → exit 0
```

### Her iki dal da artık ayrı ayrı doğrulanıyor

```
$ bash scripts/tests/check-docs.test.sh
  Fixture: ready listesi dolu (NEN-902 backlog)
    ok   fixture ready listesi = 'NEN-902'
    ok   T1 uyumlu → geçiyor
    ok   T2 uyumsuz → hata
    ok   T3 boş liste → hata
    ok   T4 satır yok → hata
    ok   T5 dosya yok → hata
    ok   T6 düzeltme sonrası geçiyor
  Fixture: ready listesi boş (NEN-902 active, backlog boş)
    ok   fixture ready listesi = '<boş>'
    ok   T8 iki taraf da boş → geçiyor
    ok   T9 boş INDEX + dolu STATUS → hata      ← negatif test
    ok   T7 gerçek STATUS.md / INDEX.md / task dosyaları / script'ler değişmedi
                                           → exit 0 (11 doğrulama)
```

`T9` DoD'un istediği negatif test: INDEX ready listesi boşken STATUS bir task
listeliyor → `check-docs.sh` exit 1.

### `test.sh` — `tasks/active/` dolu ve boş, aynı sonuç

```
$ bash scripts/test.sh          # NEN-031 tasks/active/ içindeyken
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (24 doğrulama)
SONUÇ: 2 test dosyasının hepsi geçti.      → exit 0

$ bash scripts/test.sh          # NEN-031 tasks/done/'a taşındıktan sonra
  check-docs.test.sh ✓ (11 doğrulama)
  doctor.test.sh     ✓ (24 doğrulama)
SONUÇ: 2 test dosyasının hepsi geçti.      → exit 0
```

Bağımsızlık ayrıca **birbirinin zıddı iki repo durumunda** diff'lendi:

```
A) sahte repo   — tasks/active/ DOLU, ready listesi BOŞ   → exit 0
B) gerçek repo  — tasks/active/ BOŞ,  ready listesi DOLU  → exit 0

$ diff A.log B.log
(fark yok)                                 → çıktılar birebir aynı
```

Fixture canlı `tasks/` ve `INDEX.md`'yi hiç okumadığı için iki uçtaki repo
durumu testin çıktısını değiştirmiyor.

### Yan etki: koşu süresi 16.4 s → 1.6 s

Eski fixture tüm repo'yu tarlıyordu (1.2 GB — `core/target` 893 MB +
`platforms/apple-shared/generated` 155 MB). Yeni sandbox yalnız
`scripts/` + sentetik `docs/STATUS.md` + sentetik `tasks/` kuruyor:

```
ESKİ: bash scripts/tests/check-docs.test.sh   16.372 total
YENİ: bash scripts/tests/check-docs.test.sh    1.640 total
```

NEN-005 bunu her push'ta koşacak.

### Yan bulgu — STATUS.md B4'ün gerekçesi hatalıydı

B4 "bir task `active` olduğu anda kırılıyor" ve "paket bir sonraki task
başlatıldığında yeniden kırmızıya dönecek" diyordu. **Doğru değil**: tetikleyici
bir task'ın active olması değil, **ready listesinin boşalması**. NEN-031
`active/`'e alındığında ready listesinde NEN-005/006/008/009/010 kaldı ve
`test.sh` yeşil kaldı (bu oturumda ölçüldü). NEN-007'de kırılmasının sebebi, o
an her backlog task'ının NEN-007'ye bağlı olmasıydı. `docs/STATUS.md` bu
kapanışta düzeltildi.
