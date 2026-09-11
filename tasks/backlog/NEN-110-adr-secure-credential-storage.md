---
id: NEN-110
title: Decide the secure credential storage map
milestone: M6
size: S
state: backlog
closed:
depends_on: []
blocks: [NEN-111]
adr: [20]
---

# NEN-110 — Decide the secure credential storage map

## Sonuç

ADR-0020 `accepted`: kullanıcının üç API anahtarının (OpenSubtitles · OpenAI ·
OpenRouter) hangi port üzerinden, hangi platform deposunda ve hangi yaşam
döngüsüyle saklanacağı karara bağlanmış; M6'nın üç kolu (kimlik sorgusu,
OpenSubtitles indirme, gerçek çeviri sağlayıcısı) tek bir credential kapısına
bağlanabiliyor.

## Bağlam

`NEN-033`'ün OpenSubtitles kimlik sorgusu Rust'ta var ama uygulamaya bağlı
değil: `nen-ffi`'de çağıran yok, macOS'ta anahtar girişi yok, Keychain yok.
`OpenSubtitlesApiKey`'in kendi yorumu "credentials are supplied by the
platform secure-storage adapter later" diyor — o adapter bu ADR'nin konusu.
Şartname §15 platform başına depoyu (Apple → Keychain) ve secret kurallarını
(loglanmaz · UI'da tam gösterilmez · artifact'e girmez) zaten sabitliyor;
`docs/security-policy.md` §5 yaşam döngüsünü yazıyor. Karar verilmemiş olan
**sınır**: port nerede durur, adapter hangi tarafta (Swift/Rust), anahtar
core'a nasıl ulaşır.

## Kapsam

ADR-0020 en az şunları kararlaştırır:

- `SecureCredentialStore` portu `nen-ports`'ta (`docs/architecture.md` →
  Portlar tablosuyla tutarlı); anahtar kimlikleri kapalı bir enum
  (`OpenSubtitles` · `OpenAi` · `OpenRouter`), serbest string değil
- macOS adapter'ı **Swift** tarafında Keychain (`SecItem*`), core'a
  reverse-FFI ile (`#[uniffi::export(with_foreign)]` — `ForeignHttpClient`
  emsali, ADR-0039'un aynı deseni); Rust core hiçbir platform secure-storage
  API'sine doğrudan bağlanmaz
- Secret taşıyan tipin `Debug`/`Display`/`Serialize` kuralı (K23,
  `OpenSubtitlesApiKey`'in `<redacted>` emsali) ve bellekte tutma süresi
- UI sözleşmesi: giriş alanı maskeli, kayıtlı anahtar bir daha tam
  gösterilmez (yalnız "kayıtlı" göstergesi / son karakterler), silme var
- ADR-0031 Karar 6'nın ayar yüzeyi kapsamı üçüncü kez genişliyor (M5'te
  `NEN-101` emsali) — ADR-0031'e Notlar girdisi, gövde değişmez
- Testlerde deterministic in-memory fake; gerçek Keychain yalnız macOS
  adapter'ının kendi contract testinde, ayrı bir test service adıyla

## YAPILMAYACAK

- Port ve fake implementasyonu — `NEN-111`
- Keychain adapter'ı — `NEN-112`; ayarlar UI'ı — `NEN-113`
- Diğer platformların depoları (Keystore · Credential Manager · Secret
  Service) — M9–M11; ADR yalnız haritayı çizer, implementasyon vermez
- Anahtarı plaintext config/`UserDefaults`'a yazmak — şartname §15 **yasak**

## Kanıt (DoD)

- [ ] `docs/adr/0020-*.md` yazıldı, kullanıcı onayıyla `accepted`
- [ ] `docs/adr/README.md` durum sütunu, `docs/DECISIONS.md` ADR sayacı ve
      `docs/architecture.md` → Portlar `SecureCredentialStore` satırı tutarlı
      (`NEN-097`'de ölçülen ADR-0017 kusuru tekrarlanmaz)
- [ ] ADR-0031'e Notlar girdisi eklendi (ayar yüzeyi genişlemesi)
- [ ] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

<!-- done olurken doldurulacak -->
