---
id: NEN-034
title: AI-assisted release name normalization
milestone: M6
size: M
state: backlog
depends_on: [NEN-018, NEN-124, NEN-116]
blocks: []
adr: [46]
---

# NEN-034 — AI-assisted release name normalization

## Sonuç

NEN-018'in deterministik parser'ı `Unknown` döndüğünde, dosya adı bir LLM'e
normalize ettirilerek başlık/yıl/sezon/bölüm çıkarılmaya çalışılır — kullanıcıya
soru sormadan önceki kanıt katmanlarından biri olarak.

## Kapsam

> **M6 kırılımı (2026-09-11):** "Gizlilik ADR'si" ön koşulu `NEN-124`
> (ADR-0046) olarak ayrı task; LLM çağrısı `NEN-116`'nın OpenAI adapter'ının
> HTTP/credential yolunu paylaşır — ikisi de bağımlılık.

- Yalnız deterministik katmanların **hepsi** tükendiğinde tetiklenir
- Provider port üzerinden (ADR-0019), kendi typed sonucu ve `Unknown` yolu
- Deterministic fake provider
- **Gizlilik ADR'si (ön koşul):** dosya adı K23 #8 kapsamında "özel filename
  metadata". Provider'a gönderilmesi bir izin kararıdır — varsayılan davranış,
  kullanıcı onayı, hangi alanın gittiği ve ne kadarının gittiği karara bağlanmalı

## YAPILMAYACAK

- Deterministik parser yerine geçmek — bu bir **fallback**, birinci yol değil
- Kullanıcı onayı olmadan dosya adını dışarı göndermek
- Çeviri tetiklemek — kimlik çıkarımı çeviri başlatmaz

## Kanıt (DoD)

- [ ] Gizlilik ADR'si `accepted`
- [ ] Fake provider'la `Unknown` girdiden doğru başlık/yıl çıkıyor
- [ ] Deterministik parser başarılıysa AI **hiç çağrılmıyor** (test)
- [ ] Negatif: izin verilmemişken hiçbir istek çıkmıyor (test)
- [ ] Negatif: dosya adı ve provider cevabı loglanmıyor (K23 #5, #8)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
