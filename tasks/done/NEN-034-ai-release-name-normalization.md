---
id: NEN-034
title: AI-assisted release name normalization
milestone: M6
size: M
state: done
closed: 2026-09-14
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

- [x] Gizlilik ADR'si `accepted`
- [x] Fake provider'la `Unknown` girdiden doğru başlık/yıl çıkıyor
- [x] Deterministik parser başarılıysa AI **hiç çağrılmıyor** (test)
- [x] Negatif: izin verilmemişken hiçbir istek çıkmıyor (test)
- [x] Negatif: dosya adı ve provider cevabı loglanmıyor (K23 #5, #8)

## Kanıt kaydı

### Uygulama

- `nen-ports` içinde bounded basename-stem request'i, typed normalization
  sonucu/`Unknown` yolu ve payload'sız hata portu eklendi.
- `nen-app` yalnız deterministik çözüm `Unknown`, medya-başına izin açık ve
  sanitize edilmiş local basename stem mevcutsa normalizer çağırıyor; öneri
  mevcut kanıtı otomatik olarak değiştirmiyor.
- Deterministic fake provider ile birlikte OpenAI Responses ve OpenRouter
  structured-output adapter'ları, mevcut credential/HTTP/retry yollarını
  paylaşacak şekilde eklendi. OpenRouter capability preflight cache'i korunuyor.

### Doğrulama

- `cargo test --workspace --all-targets` — geçti.
- `cargo test -p nen-providers --test openai_translation --test openrouter_translation`
  — 26 provider adapter testi geçti.
- Seçili-provider app testleri — 3/3 geçti; deterministic success ve izin
  reddi yollarında provider çağrısı 0 olarak doğrulandı.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — geçti.
- `cargo deny check` — advisories, bans, licenses ve sources kontrolleri geçti.
- `bash scripts/test.sh` — 6/6 geçti; docs/task-index/diff gates geçti.

Ayrıntılı kayıt: [NEN-034 checklist](../../evidence/M6/NEN-034-checklist.md).
