# NEN-123 — macOS OpenSubtitles menu and selection checklist

Bu kayıt NEN-123'ün gerçek `.app` kompozisyonunu ve offline FFI fixture akışını
izler. Gerçek hesap/anahtar kabul koşusu NEN-126'ya bırakılmıştır; bu task
provider testlerinde yalnız kayıtlı fixture kullanır.

## Kanıt

- [x] `bash scripts/build-macos-app.sh` — gerçek `NenPlayer.app` bağlandı.
- [x] `swift test --package-path platforms/macos --filter OpenSubtitlesSelectionTests`
      — **4 test geçti**; aday satırı download öncesi seçili değil.
- [x] Başarılı download sonrası satır gösteriliyor
      ve seçili oluyor.
- [x] Başarısız download önceki seçimi ve
      geçici hata mesajını koruyor.
- [x] Medya değişiminde geç download sonucu
      yeni kataloğa uygulanmıyor.
- [x] FFI fixture araması download çağrısı
      olmadan aday metadata'sını katalogluyor.
- [x] `swift test --package-path platforms/macos --filter MenuFixtureTests`
      — **3 test geçti**.
- [x] Anahtar yoksa production composition root provider araması bağlamıyor;
      OpenSubtitles grubu görünmez kalıyor.

## Tam suite notu

`bash scripts/test-macos.sh` çalıştırıldı. NEN-123 suite'i ve bağımsız
`NenPlaybackMPVTests` sözleşme/fixture/session/sidecar suite'leri geçerken,
mevcut `SubtitleRenderingTests` ilk gerçek render testinde libmpv
test-helper'ını **SIGSEGV / exit 11** ile sonlandırıyor. Aynı hata NEN-122
doğrulamasında da kaydedilmişti; daraltılmış koşu bunu NEN-123 kapsamından
ayırdı. Bu makine/toolchain engeli gerçek provider hesabı kullanmadı ve
NEN-126'da yapılacak tek gerçek `.app` koşusuna taşınmadı.

## Gerçek uygulama gözlemi

`bash scripts/build-macos-app.sh` ile üretilen `.app` içinde Rust FFI, Swift
shell ve `URLSessionRemoteEvidenceClient` birlikte bağlandı. Ağ hesabı veya
gerçek API anahtarı kullanılmadı; bu nedenle gerçek sağlayıcı UX kabulü
NEN-126'nın kullanıcı onaylı tek koşusudur.
