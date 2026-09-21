---
id: NEN-141
title: Stop playback when the player window closes
milestone: M6
size: S
state: done
closed: 2026-09-20
depends_on: [NEN-046]
blocks: []
adr: []
---

# NEN-141 — Stop playback when the player window closes

## Sonuç

Oynatıcı penceresi kırmızı düğmeyle veya `⌘W` ile kapatıldığında oynatma durur;
aynı uygulama süreci yaşar ve Dock/`⌘O` pencereyi temiz boş durumla geri getirir.

## Bağlam

`NEN-046`'nın ürün sözleşmesi: kırmızı düğme ve `⌘W` oynatmayı temizleyip
pencereyi kapatır, PID yaşar, yalnız `⌘Q` çıkar. Bugün bu temizliğin tek yolu
`PlayerRootView.onDisappear → model.shutdown()`; ancak macOS'ta SwiftUI `Window`
sahnesi kapatıldığında `onDisappear` güvenilir biçimde çağrılmıyor (bilinen
SwiftUI/AppKit sınırı). Çağrılmayınca oturum kapanmıyor, mpv arka planda
çalmaya devam ediyor; Dock dönüşü `resume()` çağırıyor ama `session != nil`
olduğu için no-op kalıyor ve pencere eski video ile geri geliyor. Kullanıcı
2026-09-20'de bu davranışı gerçek `.app` üzerinde bildirdi.

## Kapsam

- Yeni `WindowLifecycleWriter` (`NenPlayerShell`): bağlı `NSWindow`'un
  `willCloseNotification`'ını dinler, kapanışta `model.shutdown()` çağırır;
  pencere değişiminde yeniden bağlanır, `dismantleNSView`'da observer bırakır.
- `PlayerRootView`'e writer'ın bağlanması; `.onDisappear` idempotent ikinci yol
  olarak korunur.
- Kapanış sonrası yeniden açılışın yeni oturumla çalıştığının testi.
- Gerçek `.app` üzerinde kapanış/dönüş checklist'i.

## YAPILMAYACAK

- `applicationShouldTerminateAfterLastWindowClosed` /
  `applicationShouldHandleReopen` sözleşmesini değiştirmek (uygulama yaşamaya
  devam eder; `NEN-046`).
- Pencere konumu/boyutu kalıcılığı, çoklu pencere, Olaylar/Ayarlar
  pencerelerinin yaşam döngüsü.
- Oynatma motoru, FFI oturum kapatma sözleşmesi veya `onDisappear` davranışını
  yeniden tasarlamak.
- Ekran görüntüsünde medya adı/yolu gibi K23 verisi göstermek.

## Kanıt (DoD)

- [x] Deterministic macOS testi: gerçek `NSWindow` +
      `NSHostingView(PlayerRootView)` + `FakeSession`; `window.performClose` /
      `close()` sonrası oturum kapandı (`shutdownCount == 1`), model durumu
      temiz; `resume()` yeni oturum açıyor.
- [x] Negatif kontrol: düzeltme olmadan test kırmızı (veya harness
      `onDisappear`'ı zaten tetikliyorsa ayrım gerçek `.app` gözlemiyle
      kanıtlanır).
- [x] Gerçek `.app`: X ve `⌘W` sesi anında durdurur, pencere kapanır, PID
      yaşar; Dock boş durumla geri getirir; son medyayla yeniden oynatma
      çalışır; `⌘Q` PID'i bitirir.
- [x] `bash scripts/test-macos.sh`, `bash scripts/build-macos-app.sh`,
      `bash scripts/check-docs.sh`, `bash scripts/task-index.sh --check` ve
      `git diff --check` çıkış 0.

## Kanıt kaydı

- **Kırmızı kanıt (düzeltme öncesi):** `swift test --package-path
  platforms/macos --filter PlayerWindowLifecycleTests` → 2 test, 6 issue.
  `window.close()` sonrası `session.shutdownCount == 0`, `model.hasMedia ==
  true`, `model.mediaName` temizlenmemiş; `resume()` yeni oturum açmamış.
- **Negatif kontrol (düzeltme sonrası, `WindowLifecycleWriter` geçici devre
  dışı):** aynı 6 issue yeniden üretildi; test düzeltmeyi gerçekten ayırt
  ediyor.
- **Düzeltme sonrası odak testi:** `swift test --package-path platforms/macos
  --filter PlayerWindowLifecycleTests` → 2/2 geçti
  (`closing the player window shuts the playback session down`,
  `reopening after a close starts a fresh session and plays again`).
  Deterministik test gerçek bir `NSWindow` + `NSHostingView(PlayerRootView)` +
  `FakeSession` kullanır; kapanış `NSWindow.willCloseNotification` üzerinden
  `model.shutdown()`'a ulaşır, ardından `resume()` yeni oturum açar ve aynı
  fixture yeni oturumda yüklenir.
- **Tam paket:** `bash scripts/test-macos.sh` → exit 0; 267 shell/player (27
  suite), 57 playback/contract ve 4 Keychain testi geçti.
- **Uygulama derlemesi:** `bash scripts/build-macos-app.sh` → exit 0; güncel
  debug ad-hoc `NenPlayer.app` üretildi.
- **Gerçek `.app` manuel kabulü:** kullanıcı 2026-09-20'de X ve `⌘W` için
  sesin anında durduğunu, pencerenin kapandığını ve PID'in yaşadığını; Dock
  dönüşünün temiz boş durum getirdiğini; "Son Açılanlar"dan yeniden oynatmanın
  çalıştığını doğruladı. Ayrıntı: `evidence/M6/NEN-141-checklist.md`.
- **Kapılar:** `bash scripts/check-docs.sh` 10/10, `bash
  scripts/task-index.sh --check` ve `git diff --check` çıkış 0.
