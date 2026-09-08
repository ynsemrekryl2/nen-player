# NEN-082 — Handoff locator’ını opsiyonel kimlik kanıtına bağlama

Tarih: **2026-09-08** · Apple Silicon · macOS 27.0 · Xcode 26.6 ·
Swift 6.3.3 · libmpv 2.5.0

## Uygulama

- `PlayerModel.handleHandoff` yalnız uzak `http`/`https` handoff’lar için
  detached evidence collector başlatıyor; `openMedia` çağrısı collector’ı
  beklemiyor.
- Collector’ın typed HTTP/evidence hatası task içinde tutuluyor; fatal veya
  transient playback yüzeyine taşınmıyor.
- Mevcut `collectRemoteEvidence` ve `FfiRemoteEvidenceReport` kullanıldı;
  yeni metadata alanı, yeni hata tipi, provider çağrısı veya log yüzeyi yok.
- ADR-0043 Karar 5’e göre locator’ın path segmentleri, redirect sonrası URL
  ve `Content-Disposition` adı mevcut ADR-0009 sırasına bırakıldı.

## Otomatik kanıt

Core (`nen-app::remote_evidence`):

- `a_handoff_path_is_evidence_without_server_metadata` geçti: handoff path’i
  header olmadan `Handoff Show` / `2010` / `S01E02` kimliğine çözülüyor; query,
  fragment ve host çıktıya girmiyor.
- `content_disposition_wins_over_a_url_hint` ve
  `an_opaque_locator_without_headers_resolves_to_unknown` geçti: sunucu beyanı
  URL tahmininden önce geliyor; opak locator `Unknown` olarak kalıyor.
- `redirect_basename_and_no_ranges_still_produce_evidence` geçti: redirect
  sonrası sunucu adı yokluğunda basename fallback’i korunuyor.
- `hostile_filename_is_flattened_before_identity_resolution` geçti: traversal,
  kontrol karakteri ve RFC 5987 filename playback’i bozmadan düzleştiriliyor.
- `nen-identity` golden `resolution_layers::the_url_corpus_matches_its_golden`
  ve `url_hints` limit/hostile-input testleri geçti: en fazla 8 segment, 512
  baytlık segment sınırı; query/fragment/host hiçbir zaman kimlik değil.

macOS Swift (`HandoffTests.swift`):

- `remoteHandoffCollectsEvidenceOffThePlaybackPath` geçti: remote locator
  hemen session’a yükleniyor, collector detached task’ta çağrılıyor.
- `evidenceFailureDoesNotBecomeAPlaybackFailure` geçti: collector typed hata
  verse de ilk ve ikinci handoff yükleniyor; fatal/transient mesaj yok.
- `onlyRemoteHandoffsCollectEvidence` geçti: yerel handoff ve sıradan remote
  `openMedia` evidence collector çağırmıyor.
- `bash scripts/test-macos.sh`: **235 test, 0 failure**.
- `bash scripts/build-macos-app.sh`: **çıkış 0**, ad-hoc `.app` üretildi.

HTTP adapter’ı için mevcut deterministic `URLProtocol` suite’i de geçti:
`theCoreCollectsHashEvidenceThroughURLSession`, `theAdapterDoesNotFollowRedirectsByItself`,
`theAdapterForwardsRequestHeaders` — gerçek ağ/kredi kullanılmadı.

## Negatif kontrol

`handleHandoff` içindeki evidence başlatma çağrısı geçici olarak kaldırıldığında
`remoteHandoffCollectsEvidenceOffThePlaybackPath` hedefli NEN-082 testi beklenen
şekilde kırmızı oldu (1 failure, exit 1); çağrı geri alındığında aynı test yeşil
oldu. Collector hatasının playback akışını reddetmediği ayrıca fake recorder ile
doğrulandı.

## Tam doğrulama

- `cargo test --manifest-path core/Cargo.toml --workspace --no-fail-fast` —
  yeşil.
- `cargo fmt --all --check --manifest-path core/Cargo.toml` — yeşil.
- `cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings` — yeşil.
- `cd core && cargo deny check` — `advisories ok, bans ok, licenses ok, sources ok`.
- `git diff --check` — yeşil.
