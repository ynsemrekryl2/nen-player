# Task Sistemi

Kaynak gerçek burasıdır. `INDEX.md` üretilir — **elle düzenlenmez**.

## Dosya adı ve kimlik

```
tasks/<state>/NEN-###-kebab-case-english-slug.md
```

- ID sıralıdır, **asla yeniden kullanılmaz**, task taşınsa/iptal edilse de ölür.
- Dosya adındaki slug İngilizce, gövde metni Türkçe.

## Frontmatter

| Alan | Zorunlu | Anlam |
|---|---|---|
| `id` | ✅ | `NEN-###` |
| `title` | ✅ | Kısa İngilizce başlık |
| `milestone` | ✅ | `M0`…`M11` |
| `size` | ✅ | `S` · `M` · `L` (`XL` yasak) |
| `state` | ✅ | `backlog` · `active` · `blocked` · `done` — **bulunduğu dizinle uyumlu olmalı** |
| `depends_on` | ✅ | Önce bitmesi gereken task ID'leri (`[]` olabilir) |
| `blocks` | ✅ | Bunu bekleyen task ID'leri (`[]` olabilir) |
| `adr` | ✅ | İlgili ADR numaraları (`[]` olabilir). Doluysa, ADR `accepted` olmadan task `done` olamaz |

`blocked` durumundaki task `state: blocked` olur ama `tasks/backlog/` altında
kalır; gövdesinde **neden bloke olduğu** ve engelleyen task/soru yazılır.

## Boyut ölçeği

| Boyut | Anlamı |
|---|---|
| **S** | Tek dosya / tek kavram, ≤ ~150 satır üretim kodu |
| **M** | Birkaç dosya, tek modül sınırı içinde — **varsayılan** |
| **L** | Modül sınırı aşıyor veya platform + core'a birden dokunuyor. Planlarken bölünmeye çalışılır |
| **XL** | **Yasak.** Bölünmeden `active/`'e alınamaz |

**Bölme testi:** kanıt tek cümleyle ifade edilemiyorsa, bu en az iki task'tır.

## Zorunlu bölümler

1. **Sonuç** — tek cümle, sonuç durumu olarak yazılır
2. **Kapsam** — ne yapılacak
3. **YAPILMAYACAK** — bilinçli dışarıda bırakılanlar (kapsam kaymasına karşı)
4. **Kanıt (DoD)** — ölçülebilir maddeler, **task tipine uygun** kanıtla:

   | Task tipi | Yeterli kanıt |
   |---|---|
   | domain / logic | unit veya contract test |
   | serialization / format | golden fixture |
   | **security / validation** | **negatif test — zorunlu** |
   | performans riski taşıyan | benchmark (baseline, bağlamıyla) |
   | UI | screenshot veya kısa manuel checklist |
   | lifecycle / handoff | gerçek cihaz veya manuel acceptance checklist |
   | architecture spike | ölçüm raporu + ADR |
   | dokümantasyon / tooling | link/tutarlılık kontrolü, script çıktısı |

   Ekran kaydı veya benchmark her task için zorunlu **değildir**. Ölçmeden eşik
   yazılmaz — bkz. `docs/testing-strategy.md` → "Baseline ve invariant ayrımı".
5. **Kanıt kaydı** — kapanışta gerçek çıktı; boşsa `done` olamaz

## Durum akışı

```
backlog ──▶ active ──▶ done
   ▲          │
   └── blocked ┘
```

**Aynı anda `tasks/active/` içinde en fazla bir implementation task.**

### Başlatma

```bash
git mv tasks/backlog/NEN-017-*.md tasks/active/
# frontmatter'da: state: active
bash scripts/task-index.sh
```

### Kapatma

```bash
# 1. Kanıt kaydı bölümünü GERÇEK çıktıyla doldur
# 2. frontmatter'da: state: done
git mv tasks/active/NEN-017-*.md tasks/done/
bash scripts/task-index.sh
bash scripts/check-docs.sh
```

## Denetimler

`scripts/check-docs.sh` şunları zorlar:

1. `tasks/active/` içinde en fazla bir task
2. `state` alanı bulunduğu dizinle uyumlu
3. `done` task'ların **Kanıt kaydı** bölümü boş değil
4. `depends_on` hedefleri var olan task'lara işaret ediyor
5. Bağımlılık döngüsü yok
6. `adr` alanı dolu olan `done` task'ın ADR'si `accepted`
7. `INDEX.md` güncel

## Yeni task açma

```bash
cp tasks/_template.md tasks/backlog/NEN-029-my-new-task.md
# id, title, milestone, size, depends_on, blocks doldurulur
# ilgili docs/milestones/M#-*.md dosyasına da eklenir
bash scripts/task-index.sh
```

Task sırasında ortaya çıkan alakasız iyileştirme → **yeni backlog task'ı**,
mevcut task'a eklenmez.
