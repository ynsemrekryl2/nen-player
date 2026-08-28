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
| 6 | **Doğrulanmış kapanış commit edilir ve hemen push edilir** — yarım işe, kırmızı teste, eksik kanıta commit/push yok. Her doğrulanmış commit `origin/main`'e gider ve CI sonucu izlenir; geçmiş değiştirme kullanıcıya aittir → "Commit politikası". |
| 7 | **Spike kodu ürün kodu değildir.** `core/spikes/` altında kalır, terfi etmez. |
| 8 | **Testler gerçek provider kredisi/kotası kullanmaz.** Deterministic fake zorunlu. |
| 9 | `tasks/INDEX.md` elle düzenlenmez — `scripts/task-index.sh` üretir. |

## Task yaşam döngüsü

```
backlog ──▶ active ──▶ done
   │          │
   ├── blocked ┘        (blocked ise sebep + engelleyen task/soru zorunlu)
   └──▶ canceled         (iptal kaydı: tarih + gerekçe zorunlu)
```

Bir task'ı başlatmak:

```bash
git mv tasks/backlog/NEN-017-*.md tasks/active/    # ve frontmatter'da state: active
bash scripts/task-index.sh
```

Bir task'ı kapatmak: **Kanıt kaydı** bölümünü gerçek çıktıyla doldur, `state: done`
yap, `tasks/done/` altına taşı, index'i yeniden üret, `scripts/check-docs.sh` çalıştır.

Bir task'ı iptal etmek: özgün kapsamı ve DoD'u tarihsel kayıt olarak koru,
`state: canceled` yap, tarih ve gerekçe içeren **İptal kaydı** ekle,
`tasks/canceled/` altına taşı ve index'i yeniden üret. İptal edilen task
`done` sayılmaz ve bağımlılıkları tamamlamaz.

## Commit politikası

Commit **bir kapanış işaretidir**, ara kayıt değil. Ölçü şu: commit'in gösterdiği
ağaçta `bash scripts/check-docs.sh` çıkış 0 vermeli ve o iş kendi başına anlamlı
olmalı.

### Sormadan atılır

Aşağıdakiler kendi commit'lerini alır — her biri ayrı:

| Durum | Ön koşul |
|---|---|
| Bir task `done` oldu | Kanıt kaydı gerçek çıktıyla dolu · DoD testleri geçiyor · `check-docs.sh` çıkış 0 |
| Bir ADR `accepted` oldu | Kullanıcı onayı alınmış |
| Doküman / tooling düzeltmesi | Kendi başına tutarlı; task gerektirmeyen türden |

Hepsinde ortak zorunluluk: **staged içerikte yarım iş, kırmızı test veya
`.gitignore`'lu üretilmiş dosya yok.** Bir commit'e sığmayan iki ayrı iş varsa
iki commit atılır.

### Sormadan push edilir

Yukarıdaki koşullarla oluşturulan **her doğrulanmış commit**, oluşturulur
oluşturulmaz ayrı ayrı `git push origin main` ile gönderilir. Push yalnız mevcut
`main` dalından mevcut `origin/main` hedefine normal fast-forward teslimattır.
Her push'tan sonra GitHub Actions CI sonucu izlenir; CI yeşil olmadan iş
tamamlanmış raporlanmaz ve başka task'a geçilmez.

- Push bağlantı, kimlik doğrulama veya non-fast-forward nedeniyle reddedilirse
  force/pull/rebase yapılmaz; yerel commit korunur ve teslimatın beklediği
  bildirilir.
- CI aynı commit'ten kaynaklanan bir kusurla kırılırsa kusur giderilir, tüm
  kapılar yeniden doğrulanır, yeni commit normal push edilir ve CI yeniden
  izlenir.
- CI bağımsız bir kusur gösterirse yeni backlog task'ı açılır; mevcut kırmızı
  durum ve engel raporlanır.

### Sorulmadan yapılmaz

- **Normal `origin/main` push'u dışındaki uzağa giden işler:** PR açma, remote
  branch oluşturma veya başka remote/branch'e push
- **Geçmişi değiştiren her şey:** `--amend` · `rebase` · `reset --hard` ·
  force push · tag · branch silme
- **Kırmızı testle veya eksik kanıtla commit** (WIP kaydı) — kullanıcı açıkça
  isterse olur
- Branch açmak — bu depo tek geliştiricili ve `main` üstünde çalışıyor;
  branch'e geçmek kullanıcının kararıdır

**Tek istisna:** aynı oturumda kendi attığın ve **henüz push edilmemiş** bir
commit'in hatasını düzeltmek serbesttir (`reset --soft` + yeniden commit).
Burada kullanıcının geçmişine dokunulmuyor, kendi hatanın izi siliniyor —
ama yapıldığı **söylenir**, sessizce geçilmez.

Kural 6'nın koruduğu şey "kullanıcı her seferinde onay versin" değil, **kayıt
altına alınanın doğrulanmış olması**. Doğrulama sağlanıyorsa commit beklemez.

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
auto-sync'i onaysız uygulamak · protected/DRM audio extraction ·
Mac App Store dağıtımı (ADR-0034).

## Doğrulama

```bash
bash scripts/doctor.sh          # tüm toolchain durumu (bilgilendirici, daima 0)
bash scripts/doctor.sh M1       # yalnız M1 kapısı — blocker eksikse çıkış 1
bash scripts/test.sh            # shell testleri (doctor + check-docs)
bash scripts/task-index.sh      # INDEX.md üret
bash scripts/check-docs.sh      # task/ADR/index/STATUS tutarlılığı
```

> Milestone gereksinimleri **kümülatif**: `doctor.sh M3` core araçlarını da
> ister. Gereklilik seviyeleri: `blocker` (milestone'a başlanamaz) ·
> `soon` (milestone içinde bir task'tan önce, çıkış kodunu etkilemez) · `info`.
