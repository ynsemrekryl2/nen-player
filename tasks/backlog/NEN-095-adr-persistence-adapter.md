---
id: NEN-095
title: Decide the persistence adapter for translated artifacts
milestone: M5
size: S
state: backlog
closed:
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

- [ ] ADR-0017 `accepted` (kullanıcı onayı alınmış)
- [ ] `docs/architecture.md` → `Persistence` satırındaki "aday" ifadesi karara
      çevrildi; `docs/DECISIONS.md` → "Ertelenmiş kararlar" satırı kapandı ve
      "Teknoloji karar statüsü" tablosuna taşındı
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
