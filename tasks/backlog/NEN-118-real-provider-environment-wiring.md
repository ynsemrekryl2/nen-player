---
id: NEN-118
title: Real provider selection wired into the translation environment
milestone: M6
size: M
state: backlog
closed:
depends_on: [NEN-116, NEN-117, NEN-113, NEN-107]
blocks: [NEN-126]
adr: [19, 18]
---

# NEN-118 — Real provider selection wired into the translation environment

## Sonuç

`nen-app::TranslationEnvironment` artık kullanıcının seçtiği sağlayıcı ve
modelle, anahtarı credential store'dan alarak kuruluyor; anahtar yoksa çeviri
komutu tipli bir redle (`StartRefusal::MissingCredential`) hiç başlamıyor;
medya hash'i cache identity'ye giriyor; macOS'ta kullanıcı sağlayıcı/model
seçebiliyor.

## Kapsam

- `TranslationEnvironment::new` mock'u sabitlemekten çıkıyor: `ProviderChoice`
  (`Mock` yalnız test/`#[doc(hidden)]`, `OpenAi { model }`,
  `OpenRouter { model }`) + `Arc<dyn SecureCredentialStore>` + `Arc<dyn
  HttpClient>`; kompozisyon `nen-app`'te kalır, `nen-ffi` yalnız ilkel
  değer geçirir (ADR-0006 kural 3, `NEN-100` emsali)
- `StartRefusal::MissingCredential` ve `ProviderCapabilityMissing`
  (`NEN-117`) → `FfiTranslationStartError` düz varyantlar → Swift'te ADR-0031
  hata sunumu ("Ayarlardan anahtar girin")
- `TranslationMetadataSeed.media_hash`: `NEN-018`'in `os_hash`'i varsa
  doldurulur (ADR-0018 Karar 2'nin `media context` bileşeni artık gerçek);
  `NEN-097`'nin negatif testi iki farklı hash'in iki farklı kimlik verdiğini
  zaten kanıtlıyor — burada uçtan uca ölçülür
- macOS: `TranslationPreferenceStore`'a sağlayıcı + model satırı
  (`NEN-113`'ün ayar yüzeyinde); `canTranslateSelectedSubtitle` anahtar
  yokluğunu **kapı yapmaz** — komut basılır, tipli red gelir (kullanıcı
  ayarlara yönlendirilir); ADR-0019'un kararı farklıysa ona uyulur
- S9 kararının (ADR-0019) menü projeksiyonuna yansıması — en yeni kalıyorsa
  değişiklik yok; etiketli seçimse `NEN-098` projeksiyonu genişler

## YAPILMAYACAK

- Anahtar girişi UI'ı — `NEN-113`
- Belge-geneli ilerleme — `NEN-107` (ön koşul, burada yalnız tüketilir)
- Glossary — M6 dışı
- Testlerde gerçek sağlayıcı — Kural 8; `with_provider` seam'i mock ile kalır

## Kanıt (DoD)

- [ ] Unit (`nen-app`): anahtar yokken `start` → `MissingCredential`, provider
      `send` sayacı 0, `artifacts/` boş
- [ ] Unit: fake credential store + `FakeHttpClient` + OpenAI fixture ile
      uçtan uca iş tamamlanıyor, artifact yazılıyor
- [ ] Unit: aynı belge iki farklı `media_hash` ile iki farklı artifact
      (cache miss), aynı hash ile cache hit (provider'a gitmiyor)
- [ ] Negatif (zorunlu, K23): yazılan artifact JSON'unda ve `ArtifactRecord`
      `Debug`'ında anahtar sentinel'ı yok
- [ ] `nen-ffi`: `FfiTranslationStartError` yeni varyantları eşleniyor (mutasyon
      testi — `NEN-100` emsali)
- [ ] Swift: sağlayıcı/model ayarı kalıcı; anahtar yokken komut tipli hata
      gösteriyor, seçili altyazı değişmiyor
- [ ] `cargo test --workspace` ve `bash scripts/test-macos.sh` yeşil

## Kanıt kaydı

<!-- done olurken doldurulacak -->
