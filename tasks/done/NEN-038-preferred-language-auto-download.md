---
id: NEN-038
title: Auto-download subtitles for the preferred language
milestone: M6
size: M
state: done
closed: 2026-09-14
depends_on: [NEN-019, NEN-033, NEN-035, NEN-036, NEN-125, NEN-122]
blocks: []
adr: [47]
---

# NEN-038 — Auto-download subtitles for the preferred language

## Sonuç

Tercih edilen dilde yerel kaynak yoksa OpenSubtitles adayı otomatik indirilir —
ancak kimlik çıkarımının doğruluğu **ölçülmüş** biçimde yeterliyse.

## Kapsam

> **M6 kırılımı (2026-09-11):** "kendi ADR'si" ön koşulu `NEN-125`
> (ADR-0047); indirme mekanizması `NEN-122`'de — ikisi de bağımlılık.

- ADR-0010 Karar 9'un tür önceliğindeki **üçüncü basamağın** açılması:
  `embedded` → `user` → `opensubtitles`
- Birinci tercih dilinde aday yoksa ikinci tercihe düşme
- Kullanıcının bu davranışı kapatabilmesi
- Yanlış altyazı indiğinde davranış (geri alma / sessizce değiştirmeme)

## Ön koşul — kendi ADR'si

ADR-0010 Karar 9 bu basamağı **reddetmedi, erteledi**. Açılması iki şeye bağlı:

1. Kimlik çıkarımının ölçülmüş doğruluğu — `NEN-033` · `NEN-035` (ölçülebilir
   kabul kriteri) · `NEN-036`
2. Açılışta otomatik ağ isteği ve indirme kullanıcı-görünür bir davranış
   sözleşmesidir → **yeni bir ADR** gerekir. En az şunları karara bağlamalı:
   güven eşiğinin sayısal değeri · kullanıcının kapatabilmesi · başarısızlık
   davranışı.

**ADR yazılıp kabul edilmeden bu task `active`'e alınmaz.**

## YAPILMAYACAK

- `ai` kaynağını otomatik seçime sokmak — §9 açık komut şartı, ADR-0010 Karar 9
- Tercih edilmeyen dillerde otomatik seçim veya indirme
- Kullanıcı seçimini indirilen altyazıyla sessizce değiştirmek

## Kanıt (DoD)

- [x] Eşik altı güvende indirme **yapılmıyor** (negatif test)
- [x] Kullanıcı kapattığında hiçbir otomatik istek çıkmıyor (negatif test)
- [x] Birinci tercihte aday yoksa ikinciye düşülüyor, ikisi de yoksa `Kapalı`
- [x] Deterministic fake provider ile — gerçek kota kullanılmıyor (Kural 8)

## Kanıt kaydı

### Uygulama

- `nen-catalog` OpenSubtitles basamağını varsayılan yerel seçimden ayrı,
  açıkça parametrelenen saf bir seçim yolu olarak sunuyor; `ai` otomatik
  seçime girmiyor.
- macOS ayarı varsayılan kapalı ve `UserDefaults` ile kalıcı. Açıkken hazır
  oynatma, sidecar taraması, aday araması ve kesin doğrulanmış hash eşleşmesi
  birlikte sağlanmadan otomatik indirme başlamıyor.
- Birinci tercih dili bulunamazsa ikinci tercih dili deneniyor; iki dilde aday
  yoksa sonuç `Closed`. Medya ve tam takvim günü için yalnızca özetlenmiş
  deneme anahtarı tutuluyor; başarısız indirme aynı gün tekrarlanmıyor.
- İndirme sürerken kullanıcının altyazıyı kapatması veya başka bir kaynak
  seçmesi korunuyor; başarılı provider sonucu kullanıcı seçimini sessizce
  değiştirmiyor.

### DoD kanıtı

- `unqualifiedIdentityDoesNotDownload`: otomatik kapının altındaki `.noMatch`
  sonucu provider satırı görünse bile indirme yapmıyor.
- `disabledAutomaticDownloadDoesNotDownload`: ayar kapalıyken otomatik
  indirme isteği çıkmıyor.
- `automaticDownloadUsesSecondaryPreference`: birinci tercih `fr` yoksa
  ikinci tercih `en` seçiliyor ve deterministic fake provider indirmesi
  tamamlanıyor.
- `automaticDownloadClosesWithoutCandidate`: her iki tercih için boş fake
  provider sonucu `Closed` kalıyor.
- `automaticDownloadPreservesUserChoice` ve günlük bütçe testi, kullanıcı
  seçiminin korunmasını ve aynı medya/günde tek denemeyi kanıtlıyor.

### Doğrulama

- `swift test --package-path platforms/macos --filter OpenSubtitlesSelectionTests`
  — **11/11 geçti**.
- `swift test --package-path platforms/macos --filter SubtitlePreferenceStoreTests`
  — **8/8 geçti**.
- `cargo test --workspace --quiet`, `cargo fmt --all -- --check` ve
  `cargo clippy --workspace --all-targets -- -D warnings` — geçti.
- `cargo deny check` — advisories, bans, licenses ve sources geçti; yalnız
  mevcut duplicate dependency uyarıları var.
- `bash scripts/test.sh` — **6/6 geçti**.
- `bash scripts/build-macos-app.sh` — macOS uygulama derleme/link aşaması
  geçti; yalnız mevcut deployment-target linker uyarıları var.

K23 kapsamında özel URL, credential, tam yol, provider payload'ı, altyazı
diyaloğu veya dosya metadata'sı bu kayda alınmamıştır.
