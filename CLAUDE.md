# Nen Player — Çalışma Protokolü

Bu dosya hem insan hem agent için bağlayıcıdır. Her oturum burayı ve
`tasks/INDEX.md`'yi okuyarak başlar.

## Dil

- **Türkçe:** roadmap, milestone, task, ADR, doküman metinleri, kullanıcıyla iletişim.
- **İngilizce:** kod, tip/fonksiyon/değişken adları, dosya ve klasör adları,
  commit mesajları, test adları, kod içi yorumlar.

## Oturum başlangıcı

1. **`docs/STATUS.md`** — doğrulanmış bugün: milestone, aktif task, sıradaki
   READY, toolchain, blocker'lar.
2. `tasks/INDEX.md` — genel durum.
3. `tasks/active/` — devam eden iş var mı?
4. İlgili **milestone / task / ADR** dosyaları.

Kararların durumu için `docs/DECISIONS.md`; açık sorular için
`docs/roadmap.md` → "Açık sorular".

## Değişmez kurallar

| # | Kural |
|---|---|
| 1 | **Task dosyası olmayan kod yazılmaz.** Yeni iş → önce `tasks/backlog/NEN-###-*.md`. |
| 2 | **Aynı anda `tasks/active/` içinde en fazla bir implementation task.** |
| 3 | **Kanıt üretilmeden `done` yok.** Kanıt **task tipine göre** seçilir (`docs/testing-strategy.md` → "Kanıt formatı"): logic→test, format→golden, security→**negatif test (zorunlu)**, performans→baseline raporu, UI→screenshot veya checklist, spike→ölçüm raporu+ADR, doküman→link/tutarlılık kontrolü. Benchmark veya ekran kaydı her task için zorunlu **değildir**. |
| 4 | **Mimari karar → önce ADR.** `proposed` yazılır, kullanıcı onaylar, `accepted` olur. ADR olmadan mimari değişmez. İlgili ADR kabul edilene kadar teknoloji seçimleri **aday**dır; belgelerde kesin karar gibi yazılmaz. |
| 5 | **Unrelated refactor yok.** Yol üstünde görülen iyileştirme → yeni backlog task'ı, mevcut task'a eklenmez. |
| 6 | **Kullanıcı istemeden commit yok.** |
| 7 | **Spike kodu ürün kodu değildir.** `core/spikes/` altında kalır, terfi etmez. |
| 8 | **Testler gerçek provider kredisi/kotası kullanmaz.** Deterministic fake zorunlu. |
| 9 | `tasks/INDEX.md` elle düzenlenmez — `scripts/task-index.sh` üretir. |

## Task yaşam döngüsü

```
backlog ──▶ active ──▶ done
   ▲          │
   └── blocked ┘        (blocked ise sebep + engelleyen task/soru zorunlu)
```

Bir task'ı başlatmak:

```bash
git mv tasks/backlog/NEN-017-*.md tasks/active/    # ve frontmatter'da state: active
bash scripts/task-index.sh
```

Bir task'ı kapatmak: **Kanıt kaydı** bölümünü gerçek çıktıyla doldur, `state: done`
yap, `tasks/done/` altına taşı, index'i yeniden üret, `scripts/check-docs.sh` çalıştır.

## Commit mesajı formatı

```
feat(subtitle): indexed cue lookup (NEN-017)
fix(playback): report ended state after seek past duration (NEN-022)
docs(adr): accept SQLite + content-addressed store (ADR-0017)
```

Scope = crate veya platform. Task ID zorunlu (docs/chore hariç).

## Asla loglanmayacaklar (K23 — ihlali güvenlik hatasıdır)

medya URL'si · token içeren query · özel tam dosya yolu · subtitle diyaloğu ·
raw provider response · API key · OpenSubtitles private file ID · özel hash /
filename metadata.

Ayrıntı ve `Debug` implementasyon kuralları: `docs/security-policy.md`.

## Kapsam dışı (non-goal) — teklif edilmez

Stremio subtitle add-on · browser player · web dashboard · hosted multi-user
backend · merkezi hesap · scraping · torrent/debrid · DRM bypass · progressive
incomplete translation · tam subtitle editörü · cloud sync · sosyal özellikler ·
kullanıcıya playback engine seçtirmek · teknik ID girişi · düşük güvenli
auto-sync'i onaysız uygulamak · protected/DRM audio extraction.

## Doğrulama

```bash
bash scripts/doctor.sh          # tüm toolchain durumu (bilgilendirici, hep 0)
bash scripts/doctor.sh M1       # yalnız M1 kapısı — eksikse çıkış 1
bash scripts/task-index.sh      # INDEX.md üret
bash scripts/check-docs.sh      # task/ADR/index/STATUS tutarlılığı
```

> `doctor.sh`'ın milestone parametresi **NEN-030** ile geliyor; o task
> kapanana kadar script yalnız parametresiz çalışır.
