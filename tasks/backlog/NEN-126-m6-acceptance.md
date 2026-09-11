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
anahtarlarıyla ve onayıyla gerçek koşuda** doğrulanmış; varsayılan OpenRouter
Luna ile S3'ün EN→TR yayın kalitesi ölçülmüş; K23 negatifleri
(log/artifact/crash'te anahtar yok) ölçülmüş; milestone retro'su yazılmış.

## Bağlam

Kural 8 **testlerin** gerçek kredi/kota kullanmasını yasaklar; kabul
task'ında kullanıcının kendi anahtarıyla, küçük bir fixture ile gerçek koşu
yapılır. Bu koşu test değil, kabul checklist'idir ve `evidence/M6/` altına
yazılır. Varsayılan canlı yol için OpenSubtitles + seçili gerçek çeviri
sağlayıcısının anahtarı yeterlidir; ChatGPT Plus veya canlı OpenAI bakiyesi
önkoşul değildir. M5'te mock'un anlık bitişi
yüzünden canlı gözlenemeyen ilerleme/iptal (`NEN-101`/`102`/`104` kayıtları)
burada ilk kez insan-zamanlı gözlenir.

## Kapsam

- Checklist (`evidence/M6/NEN-126-checklist.md`), `evidence/M5/NEN-104-checklist.md`
  biçiminde:
  1. OpenSubtitles ve seçili gerçek çeviri sağlayıcısının anahtarı ayarlardan
     girildi; uygulama yeniden açılınca "kayıtlı"
  2. `contract-clip.mkv` (veya hash'i OpenSubtitles'ta olan küçük bir gerçek
     medya — kullanıcı sağlar) açıldı: kimlik şeridi/menüde OpenSubtitles
     adayları görünüyor; **hiçbir indirme olmadı** (ağ isteği sayımı —
     `nen-app` sayaç veya proxy log)
  3. Bir aday seçildi → indirildi → ekranda çizildi
  4. Telif-temiz 24 cue'luk EN→TR kabul belgesi varsayılan OpenRouter/
     `openai/gpt-5.6-luna` ile çevrildi; belge-geneli ilerleme (`NEN-107`)
     okunabilir arttı; iptal bir kez denendi (gerçek gecikme var) ve yarım
     artifact kalmadı
  5. S3 dilsel inceleme: 24/24 cue zorunlu düzeltmesiz; anlam tersine dönmesi,
     atlama/ekleme ve isim-terim tutarsızlığı ayrı ayrı 0. Bu koşul yalnız
     EN→TR için "yayın kalitesi" iddiasını destekler
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
- Aynı task içinde birden fazla model/sağlayıcıyı karşılaştırmak — Luna
  başarısızsa ayrı düzeltme task'ı Terra ile aynı kabulü yeniler
- Kod değişikliği — kusur çıkarsa ayrı task (Kural 5), kabul kırmızı kalır

## Kanıt (DoD)

- [ ] Checklist 7/7, `evidence/M6/NEN-126-checklist.md`, kullanıcı onayıyla;
      varsayılan OpenRouter/Luna koşusu ve gerekirse ayrı Terra düzeltme koşusu
- [ ] Negatif 6 ve 7 gerçek ölçümle (komut çıktısı kayıtta)
- [ ] `cargo test --workspace` ve `bash scripts/test-macos.sh` baseline ile
      aynı (kod değişmedi)
- [ ] Retro yazıldı; `docs/STATUS.md`, `docs/roadmap.md` M6 kapandı

## Kanıt kaydı

<!-- done olurken doldurulacak -->
