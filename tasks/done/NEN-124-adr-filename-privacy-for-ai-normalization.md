---
id: NEN-124
title: Decide filename privacy for AI-assisted normalization
milestone: M6
size: S
state: done
closed: 2026-09-14
depends_on: []
blocks: [NEN-034]
adr: [46]
---

# NEN-124 — Decide filename privacy for AI-assisted normalization

## Sonuç

ADR-0046 `accepted`: deterministik parser `Unknown` döndüğünde dosya adının
bir LLM'e gönderilip gönderilmeyeceği, hangi izinle, hangi alanın ve ne
kadarının gideceği karara bağlanmış — `NEN-034`'ün kendi metninin şart
koştuğu "Gizlilik ADR'si".

## Bağlam

`NEN-034` "Gizlilik ADR'si (ön koşul): dosya adı K23 #8 kapsamında 'özel
filename metadata'. Provider'a gönderilmesi bir izin kararıdır" diyor ve
DoD'unda "Gizlilik ADR'si `accepted`" var. `security-policy.md` §1 filename
metadata'sını **asla loglanmayacaklar** arasında sayıyor; dışarı göndermek
loglamaktan daha geniş bir açıklama.

## Kapsam

ADR-0046 en az şunları kararlaştırır:

- Varsayılan: **kapalı** mı (kullanıcı açar), açık mı — gerekçeyle
- İzin biçimi: tek seferlik onay diyaloğu mu, kalıcı ayar mı (ADR-0031
  Karar 6 ayar yüzeyi)
- Ne gider: yalnız basename (dizin yolu **asla**), uzantı, boyut? URL'de
  path segmentleri (ADR-0009) gider mi?
- Hangi sağlayıcıya: kullanıcının seçtiği çeviri sağlayıcısı (`NEN-118`)
  mı, ayrı seçim mi
- Cevabın güvenilmezliği: LLM çıktısı yalnız `MediaEvidence` katmanı, kesin
  kimlik değil (ADR-0009 katman sırası nerede)
- Log/`Debug` kuralı: gönderilen dosya adı ve cevap K23 kapsamında

## YAPILMAYACAK

- İmplementasyon — `NEN-034`
- Deterministik parser'ı zayıflatmak — bu bir fallback
- Onaysız gönderim — **yasak**

## Kanıt (DoD)

- [x] `docs/adr/0046-*.md` yazıldı, kullanıcı onayıyla `accepted`
- [x] `docs/adr/README.md` (Planlanan → Yazılmış), `docs/DECISIONS.md` sayacı,
      `docs/security-policy.md` §1/§2'ye gerekirse not
- [x] ADR-0009'a Notlar girdisi (yeni evidence katmanının sırası)
- [x] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

### Karar

ADR-0046 `accepted` olarak kaydedildi: AI filename normalization yalnız
deterministik kanıtlar `Unknown` kaldığında ve medya-başına açık kullanıcı
izniyle çalışır; yalnız sanitize edilmiş basename stem'i, kullanıcının seçtiği
translation provider üzerinden gönderilir. URL/path, uzantı, boyut, hash,
metadata, içerik ve subtitle diyaloğu gönderilmez. LLM sonucu kesin kimlik
değil, düşük öncelikli untrusted evidence'tır.

### Doğrulama

- `docs/adr/0046-filename-privacy-for-ai-normalization.md` oluşturuldu ve
  kullanıcının 2026-09-14 onayıyla `status: accepted` oldu.
- `docs/adr/README.md` 0046'yı Planlanan'dan Yazılmış'a taşıyor;
  `docs/DECISIONS.md` kabul sayacı **37** ve 0046 listede.
- `docs/adr/0009-media-evidence-and-identity.md` Notlar bölümüne AI-derived
  evidence katmanının sırası eklendi.
- `docs/security-policy.md` §1/§2, ADR-0046 veri minimizasyonu ve untrusted
  LLM sonucu kurallarıyla güncellendi.
- `bash scripts/task-index.sh --check`, `bash scripts/check-docs.sh` (**10/10**)
  ve `git diff --check`: **geçti**.
