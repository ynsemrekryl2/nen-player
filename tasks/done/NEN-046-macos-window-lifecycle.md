---
id: NEN-046
title: macOS window lifecycle — no inert app after the window closes
milestone: M3
size: S
state: done
depends_on: [NEN-024]
blocks: []
adr: [31]
---

# NEN-046 — macOS window lifecycle: no inert app after the window closes

## Sonuç

Oynatma penceresi kapatıldığında uygulama ya kapanır ya da geri getirilebilir;
menüsü açılan ama hiçbir şey yapamayan bir durum kalmaz.

## Kapsam

- Sahnenin `WindowGroup` yerine **tek pencereli** bir modele geçirilmesi
  (`Window`), böylece "tek pencere" kapsam kararı kodda zorlanır
- Son pencere kapandığında uygulamanın sonlanması **veya** pencerenin
  menüden/Dock'tan geri getirilebilmesi — ikisinden biri, ikisi arası değil
- `PlayerModel.shutdown()` ile `attach(to:)` yaşam döngüsünün yeni sahneye
  göre gözden geçirilmesi: kapatma sonrası yeniden açılış çalışan bir
  oturum üretmeli, `pollTask` yeniden başlamalı
- `pendingURL`'in sahipsiz kalmaması — oturum yokken gelen `openMedia`
  ya bir yüzey bulmalı ya da kullanıcıya görünür bir sonuç üretmeli

## YAPILMAYACAK

- Çoklu pencere desteği — tek pencere M4 handoff'unun dayandığı model
- Pencere konumu/boyutu kalıcılığı — ayrı iş
- Kısayol kapsamı → `NEN-047`
- Dock ikonu / menü bar davranışının yeniden tasarlanması

## Neden ayrı task

`NEN-024` incelemesinde çalışan `.app` üzerinde ölçüldü: pencere kapatıldıktan
sonra uygulama menü çubuğunda yaşamaya devam ediyor, `⌘O` dosya seçiciyi
açıyor, dosya seçilip "Aç" tıklanınca **hiçbir şey olmuyor**, ve `Pencere`
menüsündeki her madde soluk — geri dönüş yolu yok. Sebep: `onDisappear`
oturumu kapatıyor ama `WindowGroup` sahnesi uygulamayı sonlandırmıyor ve
`CommandGroup(replacing: .newItem)` pencereyi geri getirecek komutu da
kaldırmış durumda. Kapsam dışı bir düzeltme olduğu için NEN-024'e
yamalanmıyor (CLAUDE.md kural 5).

## Kanıt (DoD)

- [x] Medya oynarken pencere kapatılıp `⌘O` ile dosya seçildiğinde medya
      oynuyor **veya** uygulama pencere kapanınca sonlanmış oluyor (checklist)
- [x] Kapanış sonrası geri gelen pencerede video yüzeyi çalışıyor — siyah
      kalmıyor (checklist)
- [x] `shutdown()` sonrası `openMedia` çağrısının sessizce yutulmadığını
      gösteren model testi
- [x] `Pencere` menüsünde uygulama penceresiz kaldığında etkin bir geri
      dönüş yolu var veya bu durum hiç oluşmuyor (checklist)

## Kanıt kaydı

- Seçilen ürün davranışı: tek `Window`; son pencere kapanınca
  `applicationShouldTerminateAfterLastWindowClosed` uygulamayı sonlandırıyor.
  Penceresiz, menüsü yaşayan süreç oluşmuyor.
- Manuel acceptance: Apple M5 · arm64 · macOS 27.0 (26A5416b) · debug,
  ad-hoc imzalı `.app` · yalnız `fixtures/media/contract-clip.mkv`.
  Oynatma → kırmızı kapatma → süreç listesinde Nen Player yok → temiz yeniden
  açılış → son fixture yeniden oynatma adımları **6/6 geçti**. Ayrıntı:
  `evidence/M3/NEN-046-checklist.md`.
- `bash scripts/test-macos.sh` çıkış 0: **31 test / 6 suite**, 0 failure.
  Yeni model testleri:
  `shutdown clears stale playback state and a later attach opens pending media`
  ve `reattaching after shutdown restarts event polling`.
- `bash scripts/build-macos-app.sh` ve
  `codesign --verify --deep --strict platforms/macos/.build/NenPlayer.app`
  çıkış 0.
- ADR-0031 `accepted`; yeni mimari karar veya dış bağımlılık yok.
