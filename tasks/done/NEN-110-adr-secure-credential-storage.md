---
id: NEN-110
title: Decide the secure credential storage map
milestone: M6
size: S
state: done
closed: 2026-09-11
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

- [x] `docs/adr/0020-*.md` yazıldı, kullanıcı onayıyla `accepted`
- [x] `docs/adr/README.md` durum sütunu, `docs/DECISIONS.md` ADR sayacı ve
      `docs/architecture.md` → Portlar `SecureCredentialStore` satırı tutarlı
      (`NEN-097`'de ölçülen ADR-0017 kusuru tekrarlanmaz)
- [x] ADR-0031'e Notlar girdisi eklendi (ayar yüzeyi genişlemesi)
- [x] `bash scripts/check-docs.sh` çıkış 0

## Kanıt kaydı

**ADR-0020 `accepted`, 2026-09-11, kullanıcı onayıyla.** Sekiz kararlı madde:
`nen-ports::credentials::SecureCredentialStore` portu (senkron, object-safe,
kapalı `CredentialKind` enum'u) · tek `ApiKey` newtype'ı (kind'dan bağımsız,
`Debug`/`Display` → `<redacted>`, `Serialize` yok) · tipli payload'suz hata
(`Unavailable`/`Denied`/`Corrupt`) · adapter Swift'te, core'a reverse-FFI ile
(`ForeignSecureCredentialStore`, `ForeignHttpClient`/ADR-0039 emsali; Rust
hiçbir platform secure-storage API'sine bağlanmaz; anahtar **tek yoldan** —
Rust portundan — geçer, ayarlar UI'ı dahil) · macOS deposu login keychain +
generic password, service `player.nen.macos`, `kSecAttrSynchronizable` yok ·
UI yalnız "kayıtlı" göstergesi verir, kayıtlı anahtar bir daha okunmaz · test
kuralı: `InMemoryCredentialStore` fake + contract kiti + K23 guard, gerçek
Keychain yalnız `NEN-112`'nin kendi testinde · diğer platformlar yalnız
harita (M9–M11), implementasyon bu ADR'nin kapsamı değil.

**Üç kullanıcı kararı** (plan onayı sırasında, `AskUserQuestion`):
anahtarın yazma/okuma yolu her zaman Rust portundan geçer (Swift adapter'ının
tek çağıranı Rust'tır) · kayıtlı anahtar yalnız "kayıtlı" göstergesiyle
gösterilir, son karakterler bile değil · macOS deposu login keychain +
generic password (data-protection keychain değil — ad-hoc imzalı geliştirme
build'inde application-identifier entitlement'ı olmadan `-34018` ile düşerdi).

**Doküman tutarlılığı** (`NEN-097`'nin ölçtüğü ADR-0017 kusuru
tekrarlanmadı): `docs/adr/README.md` 0020 satırı **Planlanan**'dan
**Yazılmış**'a taşındı, durum `✅ accepted`; `docs/DECISIONS.md` §2'ye karar
satırı, §5 **Kararlı** tablosuna "macOS Keychain" satırı, §6 sayacı 34→35;
`docs/architecture.md` → Portlar `SecureCredentialStore` satırı ve "Kabul
edilmiş teknoloji" kutusu güncellendi; `docs/adr/0031-*.md` → Notlar'a Karar
6'nın üçüncü genişlemesi (gövde ADR-0001 gereği değişmedi, `NEN-101`
emsali).

Kod değişmedi (karar/doküman task'ı; Kural 1 gereği implementasyon önce ADR
ister). `bash scripts/check-docs.sh` çıkış 0. `bash scripts/task-index.sh`
ile `tasks/INDEX.md` yeniden üretildi, `docs/STATUS.md` güncellendi.
