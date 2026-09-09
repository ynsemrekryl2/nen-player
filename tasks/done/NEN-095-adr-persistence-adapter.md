---
id: NEN-095
title: Decide the persistence adapter for translated artifacts
milestone: M5
size: S
state: done
closed: 2026-09-09
depends_on: [NEN-094]
blocks: [NEN-096]
adr: [0017]
---

# NEN-095 — Decide the persistence adapter for translated artifacts

## Sonuç

Doğrulanmış artifact'lerin nerede ve nasıl saklandığı `accepted` bir ADR ile
sabitlendi.

## Bağlam

`docs/DECISIONS.md` → "Ertelenmiş kararlar" bu kararı adıyla **ADR-0017**'ye
bağlıyor ve `docs/architecture.md` → Portlar tablosunda `Persistence` satırı
"aday: SQLite + CAS" diyor — yani bugün **karar değil aday** (Kural 4).
Şartname §11 de "değerlendirilebilir" diyor ve seçimin ADR ile
gerekçelendirilmesini istiyor.

Karar `NEN-094`'ten sonra verilir, çünkü ne saklandığı bilinmeden nasıl
saklanacağı seçilemez.

## Kapsam

- ADR-0017: index + içerik saklama modeli, atomik commit mekanizması, dosya
  düzeni, veritabanı yolunun platformdan enjekte edilmesi
- Offline garantisinin kararda yazılı olması (S4, 2026-09-08: kaydedilmiş
  artifact ağ olmadan açılır; ayrı bir offline modu yok)
- Ephemeral session ile persistent artifact ayrımının nerede durduğu (§11)
- En az bir reddedilen alternatifin gerekçesi (ADR şablonu bu bölümü boş
  bırakmayı yasaklıyor)

## YAPILMAYACAK

- Implementasyon — `NEN-096` · `NEN-098`
- Cache identity bileşenleri — `NEN-097` (ADR-0018)
- Cloud sync — non-goal (S8)

## Kanıt (DoD)

- [x] ADR-0017 `accepted` (kullanıcı onayı alınmış, 2026-09-09)
- [x] `docs/architecture.md` → `Persistence` satırındaki "aday" ifadesi karara
      çevrildi; `docs/DECISIONS.md` → "Ertelenmiş kararlar" satırı kapandı ve
      "Teknoloji karar statüsü" tablosuna taşındı
- [x] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

**ADR-0017 `accepted` — 2026-09-09.** Kullanıcı üç kararı verdi: (1) index
teknolojisi yalnız dosya sistemi — SQLite/redb reddedildi, yeni dış bağımlılık
yok; (2) artifact tek kanonik dosya — metadata + normalize cue'lar + WebVTT
bir arada, ayrı `.vtt` yazılmıyor; (3) blok checkpoint'i kalıcı olacak ama
implementasyonu M6'ya bırakıldı (yeni `tasks/backlog/NEN-105-*.md`). ADR gövdesi
altı numaralı Karar maddesi + beş satırlık Reddedilen alternatifler tablosu
(SQLite/rusqlite · redb · merkezi tek `index.json` · metadata+ayrı `.vtt` ·
M5'te kalıcı checkpoint) ile dolduruldu.

**"Aday" ifadeleri karara çevrildi:** `docs/architecture.md` (giriş kutusu,
ASCII diyagram, `Persistence`/`nen-persist` satırları, adaptör-seçim
paragrafı), `docs/DECISIONS.md` ("Ertelenmiş kararlar"dan çıkarıldı,
"Kararlı" tablosuna ADR-0017 linkiyle eklendi, "Aday" tablosundan satır
silindi), `docs/testing-strategy.md` (`Persistence` kiti paragrafı),
`docs/milestones/M5-translation-core.md` (Kapsam + ADR tablosu + kapanış
paragrafı — 0015/0016/0045'in de zaten `accepted` olduğu bu vesileyle
düzeltildi). `docs/product-spec.md` §11 dokunulmadı — orijinal şartname
metni, ADR-0016 kapanışında §10'un da değişmediği emsalle aynı.

`grep -rn "aday: SQLite\|SQLite + content-addressed"` sonrası yalnız
ADR-0017'nin kendi Bağlam/Reddedilen alternatifler bölümü, `docs/STATUS.md`'nin
2026-09-08 tarihli tarihsel kaydı, `docs/product-spec.md`'nin orijinal
şartname metni ve bu task dosyasının kendi Bağlam bölümü eşleşiyor — hiçbiri
güncel bir aday iddiası taşımıyor.

**Karardan doğan iki takip kaydı** (Kural 5): `tasks/backlog/NEN-105-resumable-checkpoint-store.md`
(M6, kalıcı checkpoint implementasyonu, `docs/milestones/M6-real-providers.md`'e
eklendi) ve `NEN-098-sqlite-artifact-index.md` → `NEN-098-artifact-metadata-index.md`
yeniden adlandırması (`git mv`, içerik değişmedi — dosya adı artık reddedilmiş
bir teknolojiyi anmıyor).

`bash scripts/task-index.sh` sonrası `tasks/INDEX.md` **105 task** sayıyor
(88 → 89 done, backlog 14 → 14: `095` done oldu, `105` yeni açıldı).
`bash scripts/check-docs.sh` çıkış **0**. Kod değişmedi — karar/doküman
task'ı, `cargo`/`swift` kapıları koşulmadı (emsal: `NEN-079`/`NEN-103`).
