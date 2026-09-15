---
id: NEN-130
title: Restore one-shot local subtitle auto-selection
milestone: M6
size: S
state: done
closed: 2026-09-16
depends_on: [NEN-038]
blocks: [NEN-129]
adr: []
---

# NEN-130 — Restore one-shot local subtitle auto-selection

## Sonuç

Playback `ready` olduktan sonra bulunan tercih dilindeki sidecar menüye eklenir
ama kendiliğinden açılmaz; buna karşılık sidecar `ready` öncesi geldiyse yerel
öncelikle açılır ve opt-in OpenSubtitles otomatik indirme kapıları bozulmaz.

## Bağlam

`NEN-038`, opt-in OpenSubtitles otomatik indirmesini yerel kaynaklardan sonra
çalıştırmak için `applyAutoSelectionIfNeeded()` çağrısını sidecar taraması
bitene kadar erteledi. Bu değişiklik ADR-0031 Karar 4.3'ün tek-seferlik yerel
seçim sınırını yanlışlıkla genişletti: playback zaten `ready` olduktan sonra
gelen sidecar, mevcut `SubtitleMenuTests.aLateSidecarNeverRetriggersAutomaticSelection`
negatifini kırarak ekrana açılıyor.

Regresyon NEN-129 dosyalarında değildir; NEN-129'un zorunlu macOS kapısı bu
mevcut main kusurunu izole testte yeniden üretmiştir. Kullanıcı 2026-09-16'da
NEN-129'u geçici bloke edip bu ayrı düzeltmeyi yapmayı onayladı.

## Kapsam

- Yerel otomatik seçim fırsatı ile OpenSubtitles otomatik indirme kararının
  yaşam döngüsünü birbirinden ayır:
  - playback `ready` olduğunda o anda mevcut embedded/user kaynaklar tam bir
    kez değerlendirilir,
  - daha sonra gelen sidecar menüyü büyütür fakat seçimi değiştirmez,
  - geç gelen yerel kaynak provider indirmesini bastırır; kendisi de otomatik
    açılmaz,
  - sidecar `ready` öncesi geldiyse mevcut yerel öncelik korunur,
  - uygun ve yerel kaynaksız opt-in OpenSubtitles yolu tarama/identity/candidate
    kapıları tamamlanınca çalışmaya devam eder.
- Yeni yaşam döngüsü alanı her medya açılışında ve katalog resetinde sıfırlanır.
- Var olan kırmızı sidecar testi korunur; opt-in yolun geç sidecar durumunu da
  kapsayan regresyon testi eklenir.

## YAPILMAYACAK

- NEN-129 çeviri kodu, prompt'u veya resume sözleşmesini değiştirmek
- ADR-0031 ya da ADR-0047 kararlarını değiştirmek
- Menü sırası, tercih sırası, günlük indirme bütçesi veya identity güven
  kapısını değiştirmek
- Test beklentisini mevcut yanlış davranışa göre gevşetmek

## Kanıt (DoD)

- [x] İzole kırmızı test düzeltme öncesi aynı iki assertion ile yeniden
      üretilmiş; düzeltme sonrası geçiyor
- [x] Geç sidecar `ready` sonrası menüye giriyor fakat seçilmiyor/çizilmiyor
- [x] Opt-in açıkken geç sidecar provider indirmesini bastırıyor ve yine
      kendiliğinden açılmıyor
- [x] Sidecar `ready` öncesi geldiyse yerel kaynak açılıyor; yerel kaynaksız
      opt-in OpenSubtitles otomatik indirmesi çalışıyor
- [x] `bash scripts/test-macos.sh`, `bash scripts/check-docs.sh`,
      `bash scripts/task-index.sh --check` ve `git diff --check` geçiyor

## Kanıt kaydı

- Kırmızı kanıt: `aLateSidecarNeverRetriggersAutomaticSelection` izole koşuda
  `selectedSubtitleToken == nil` ve `drawnSubtitles.isEmpty` beklentilerini
  aynı anda kırdı; token `1`, çizilen çıktı `.document(1)` idi.
- Düzeltme sonrası odak testleri:
  - `aLateSidecarNeverRetriggersAutomaticSelection` — 1/1 geçti,
  - `lateLocalSourceSuppressesAutomaticDownloadWithoutOpening` — 1/1 geçti;
    geç sidecar menüde, seçim/çizim yok, provider download isteği 0,
  - `localSourceWinsBeforeAutomaticDownload` — 1/1 geçti,
  - `automaticDownloadUsesSecondaryPreference` — 1/1 geçti.
- `bash scripts/test-macos.sh` — çıkış 0: player-shell **241/241**, gerçek
  libmpv **57/57**, Keychain **4/4** geçti; yalnız mevcut OpenGL ve deployment
  target uyarıları kaldı.
- `bash scripts/check-docs.sh` — 10/10; `bash scripts/task-index.sh --check` ve
  `git diff --check` — çıkış 0.
