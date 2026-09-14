# NEN-109 kanıt kaydı — remote embedded text extraction

Tarih: 2026-09-14
Ortam: macOS 27.0 (26A5416b), arm64e, Apple Swift 6.3.3, libmpv 2.5.0,
FFmpeg libavformat/libavcodec/libavutil 63.1.101/63.1.101/61.1.101.
Derleme: SwiftPM debug test ürünü; Rust workspace debug testleri.

## DoD kanıtları

1. **Gerçek Stremio HTTP akışı:** Kurulu Stremio EngineFS server süreci
   doğrulandı; canlı EngineFS proxy akışı `200`, `73482` byte ve
   `video/x-matroska` döndürdü. URL ve medya metni kanıta yazılmadı. Aynı
   akış, `NEN_STREMIO_M6_URL` env girdisiyle çalışan
   `aConfiguredStremioHTTPStreamExtractsEmbeddedText` testinde geçti; test
   çıktısı `0.033 seconds`.
2. **Gerçek iptal ölçümü:** `cancellingAReadStopsTheUnderlyingURLSessionTask`
   gerçek `URLSessionDataTask` üzerinde çalışan kontrollü HTTP transport ile
   okuma durdurma callback'ini bekledi ve extraction thread'inin tamamlanmasını
   `1.0 s` altında assert etti; test çıktısı `0.020 seconds`.
3. **Tipli byte-cap reddi:** `anOversizedRemoteResponseIsATypeRefusalNotTruncatedText`
   Content-Length `8388609` ile `FfiPlaybackError.RemoteResponseTooLarge`
   bekledi; response truncation veya subtitle payload kabul edilmedi.
   Bounded başarılı yol ayrıca `Range: bytes=0-8388607` başlığını assert etti.
4. **NEN-044 regresyonu:** `theRealAdapterPassesTheSharedContractKit` ve
   yerel İngilizce/Türkçe text extraction, bitmap negatif testi ve tüm ilgili
   playback/session testleri dahil `bash scripts/test-macos.sh` sonucu
   `271 tests in 33 suites passed`.

## Ek doğrulamalar

- `cargo fmt --all -- --check` → çıkış 0.
- `cargo test --workspace --quiet` → tüm workspace suite'leri geçti.
- `cargo clippy --workspace --all-targets -- -D warnings` → çıkış 0.
- `cargo deny check` → `advisories ok, bans ok, licenses ok, sources ok`.
- `bash scripts/build-macos-app.sh` → gerçek `NenPlayer.app` debug bundle'ı
  tamamlandı.
- `bash scripts/test.sh` → 4 doğrulama dosyasının hepsi geçti.
- `bash scripts/doctor.sh M3` → tüm M3 blocker'ları hazır.
- `git diff --check` → çıkış 0.
- `bash scripts/check-docs.sh` → tüm 10 denetim geçti.
- ADR-0046 `status: accepted`; byte bütçesi, cancellation, typed refusal ve
  payload-free hata sınırı bu kayıttaki uygulamanın kararıdır.
