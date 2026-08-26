---
id: NEN-046
title: macOS window lifecycle — no inert app after the window closes
milestone: M3
size: S
state: backlog
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

- [ ] Medya oynarken pencere kapatılıp `⌘O` ile dosya seçildiğinde medya
      oynuyor **veya** uygulama pencere kapanınca sonlanmış oluyor (checklist)
- [ ] Kapanış sonrası geri gelen pencerede video yüzeyi çalışıyor — siyah
      kalmıyor (checklist)
- [ ] `shutdown()` sonrası `openMedia` çağrısının sessizce yutulmadığını
      gösteren model testi
- [ ] `Pencere` menüsünde uygulama penceresiz kaldığında etkin bir geri
      dönüş yolu var veya bu durum hiç oluşmuyor (checklist)

## Kanıt kaydı

<!-- done olurken doldurulacak -->
