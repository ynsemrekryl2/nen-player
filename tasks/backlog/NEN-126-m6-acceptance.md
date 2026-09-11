---
id: NEN-126
title: Real providers acceptance
milestone: M6
size: S
state: backlog
closed:
depends_on: [NEN-118, NEN-123, NEN-113]
blocks: []
adr: []
---

# NEN-126 — Real providers acceptance

## Sonuç

M6'nın altı çıkış kriteri gerçek `.app` üzerinde, kullanıcının **kendi
anahtarlarıyla ve onayıyla tek bir gerçek koşuda** doğrulanmış; S3'ün dilsel
kalite maddesi ilk kez gerçek bir modelle işaretlenmiş; K23 negatifleri
(log/artifact/crash'te anahtar yok) ölçülmüş; milestone retro'su yazılmış.

## Bağlam

Kural 8 **testlerin** gerçek kredi/kota kullanmasını yasaklar; kullanıcı
kararı (2026-09-11): kabul task'ında, kullanıcının kendi anahtarıyla, küçük bir
fixture ile **bir** gerçek koşu yapılır. Bu koşu test değil, kabul
checklist'idir ve `evidence/M6/` altına yazılır. M5'te mock'un anlık bitişi
yüzünden canlı gözlenemeyen ilerleme/iptal (`NEN-101`/`102`/`104` kayıtları)
burada ilk kez insan-zamanlı gözlenir.

## Kapsam

- Checklist (`evidence/M6/NEN-126-checklist.md`), `evidence/M5/NEN-104-checklist.md`
  biçiminde:
  1. Üç anahtar ayarlardan girildi; uygulama yeniden açılınca "kayıtlı"
  2. `contract-clip.mkv` (veya hash'i OpenSubtitles'ta olan küçük bir gerçek
     medya — kullanıcı sağlar) açıldı: kimlik şeridi/menüde OpenSubtitles
     adayları görünüyor; **hiçbir indirme olmadı** (ağ isteği sayımı —
     `nen-app` sayaç veya proxy log)
  3. Bir aday seçildi → indirildi → ekranda çizildi
  4. Küçük fixture (`fixtures/subtitles/languages/english.srt` veya 3 cue'luk
     gömülü track) OpenAI **veya** OpenRouter ile çevrildi; belge-geneli
     ilerleme (`NEN-107`) okunabilir arttı; iptal bir kez denendi (gerçek
     gecikme var) ve yarım artifact kalmadı
  5. S3 dilsel inceleme: çıktı "anlaşılır" mı — kullanıcı işaretler; iddia
     bundan öteye gitmez
  6. Negatif: `~/Library/Logs`, `Console` çıktısı, `artifacts/*.json`,
     `UserDefaults` içinde anahtar sentinel'ı (anahtarın son 4 karakteri)
     **yok** — `grep` ölçümü
  7. Negatif: anahtar silinip komut verildiğinde tipli red, istek yok
- Retro `docs/milestones/M6-real-providers.md`'ye (süre, yanlış çıkan
  varsayımlar, ADR revizyonları, M7 kırılımı işaretçisi)
- Kabul için kullanılan anahtarlar koşu sonrası **silinir** (kullanıcı
  isterse kalır — checklist'te kayıt)

## YAPILMAYACAK

- Otomatik testte gerçek ağ — Kural 8
- Birden fazla model/sağlayıcıyı karşılaştırmak — tek koşu
- Kod değişikliği — kusur çıkarsa ayrı task (Kural 5), kabul kırmızı kalır

## Kanıt (DoD)

- [ ] Checklist 7/7, `evidence/M6/NEN-126-checklist.md`, kullanıcı onayıyla
- [ ] Negatif 6 ve 7 gerçek ölçümle (komut çıktısı kayıtta)
- [ ] `cargo test --workspace` ve `bash scripts/test-macos.sh` baseline ile
      aynı (kod değişmedi)
- [ ] Retro yazıldı; `docs/STATUS.md`, `docs/roadmap.md` M6 kapandı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
