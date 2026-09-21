# NEN-141 kanıt kaydı — Pencere kapanınca oynatma durur

Tarih: **2026-09-20** · Apple Silicon · macOS 27.0 (26A428) · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0 · Debug, ad-hoc imzalı `NenPlayer.app`

Kanıt medyası yalnız telif-temiz sentetik fixture'dır:
`fixtures/media/contract-clip.mkv`.

## Otomatik kanıt

| Kanıt | Sonuç |
|---|---|
| `swift test --package-path platforms/macos --filter PlayerWindowLifecycleTests` (düzeltme öncesi) | FAIL — 6 issue; `window.close()` sonrası `shutdownCount == 0`, `hasMedia == true`, `mediaName` temizlenmedi |
| Aynı test, `WindowLifecycleWriter` geçici devre dışı (negatif kontrol) | FAIL — 6 issue; oturum kapanmadı, `resume()` yeni oturum açmadı |
| Aynı test, düzeltmeyle | PASS — 2/2: `closing the player window shuts the playback session down`, `reopening after a close starts a fresh session and plays again` |
| `bash scripts/test-macos.sh` | PASS — çıkış 0; 267 shell/player (27 suite), 57 playback/contract ve 4 Keychain testi |
| `bash scripts/build-macos-app.sh` | PASS — çıkış 0; debug ad-hoc `NenPlayer.app` üretildi |

Kök neden ölçüldü: macOS'ta SwiftUI `Window` sahnesi kapatıldığında
`onDisappear` güvenilir biçimde çağrılmıyor; sahnenin içeriği bağlı kalıyor, bu
yüzden `PlayerRootView.onDisappear → model.shutdown()` hiç çalışmıyor ve oturum
pencereyle birlikte ölmüyordu. `WindowLifecycleWriter` aynı pencereyi
`NSWindow.willCloseNotification` üzerinden dinleyip `model.shutdown()` çağırıyor;
`.onDisappear` ikinci, idempotent yol olarak korundu.

## Elle kabul

1. Gerçek `NenPlayer.app`, `contract-clip.mkv` ile açıldı; video oynadı.
2. Oynatma sürerken kırmızı kapatma düğmesine basıldı: ses anında durdu,
   pencere kapandı, aynı `NenPlayer` PID'i yaşamaya devam etti.
3. Dock'tan uygulama etkinleştirildi: tek oynatıcı penceresi temiz boş
   durumla geri geldi; eski süre/transport kalmadı.
4. "Son Açılanlar"dan aynı fixture yeniden açıldı; yeni oturumda oynadı.
5. Oynatma sürerken `⌘W` basıldı: aynı davranış — ses durdu, pencere kapandı,
   PID yaşadı.

Sonuç: **5/5 geçti**; kullanıcı 2026-09-20'de doğruladı.

## Doküman kapıları

```text
bash scripts/check-docs.sh          PASS — 10/10 denetim
bash scripts/task-index.sh --check  PASS
git diff --check                    PASS
```

Üç komutun da çıkışı 0 oldu.