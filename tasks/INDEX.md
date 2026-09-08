# Task Index

<!-- ÜRETİLEN DOSYA — elle düzenlemeyin.
     Yenilemek için: bash scripts/task-index.sh -->

Toplam **88** task · ✅ done 80 · 🔵 active 0 · ⛔ blocked 0 · 🚫 canceled 2 · ⚪ backlog 6

Format ve kurallar: [tasks/README.md](README.md) · Milestone planı: [docs/roadmap.md](../docs/roadmap.md)

## M0 — Foundation

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-001](done/NEN-001-repo-skeleton.md) | Repository skeleton | S | ✅ done | — |
| [NEN-002](done/NEN-002-task-system.md) | Task system and index generator | M | ✅ done | NEN-001 |
| [NEN-003](done/NEN-003-adr-system.md) | ADR system | S | ✅ done | NEN-001 |
| [NEN-004](done/NEN-004-toolchain-doctor.md) | Toolchain doctor script | S | ✅ done | NEN-001 |

## M1 — Core Technical Spike

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-005](done/NEN-005-ci-skeleton.md) | CI skeleton | S | ✅ done | NEN-004 NEN-007 |
| [NEN-006](done/NEN-006-log-redaction.md) | Log redaction helpers and guard test | M | ✅ done | NEN-001 NEN-007 |
| [NEN-007](done/NEN-007-ffi-skeleton.md) | Rust workspace and UniFFI skeleton | M | ✅ done | NEN-004 |
| [NEN-008](done/NEN-008-spike-large-cue-lists.md) | Spike - large cue list across FFI | M | ✅ done | NEN-007 |
| [NEN-009](done/NEN-009-spike-async-cancellation.md) | Spike - async progress and cancellation | M | ✅ done | NEN-007 |
| [NEN-010](done/NEN-010-spike-typed-errors.md) | Spike - typed error mapping | S | ✅ done | NEN-007 |
| [NEN-011](done/NEN-011-spike-kotlin-parity.md) | Spike - Kotlin binding parity | M | ✅ done | NEN-008 NEN-009 NEN-010 |
| [NEN-012](done/NEN-012-adr-core-language.md) | Spike report and core language decision | S | ✅ done | NEN-011 NEN-029 |
| [NEN-029](done/NEN-029-playback-reverse-ffi-spike.md) | Playback/renderer reverse-FFI boundary spike | M | ✅ done | NEN-007 NEN-009 |
| [NEN-030](done/NEN-030-milestone-aware-doctor.md) | Milestone-aware doctor and STATUS consistency checks | S | ✅ done | — |
| [NEN-031](done/NEN-031-check-docs-test-fixture-independence.md) | Make check-docs test fixture independent of live repo state | S | ✅ done | — |
| [NEN-032](done/NEN-032-doctor-swift-milestone-level.md) | Move swift to M1 in doctor's milestone levels | S | ✅ done | NEN-030 |
| [NEN-085](done/NEN-085-flaky-spike-async-cancel-test.md) | Stabilize the flaky spike-async-cancel completion test | S | ✅ done | — |

## M2 — Subtitle Core

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-013](done/NEN-013-strict-srt-parser.md) | Strict SRT parser | M | ✅ done | NEN-012 |
| [NEN-014](done/NEN-014-webvtt-writer.md) | WebVTT writer | S | ✅ done | NEN-013 |
| [NEN-015](done/NEN-015-encoding-detection.md) | Encoding detection and sanitization | M | ✅ done | NEN-013 |
| [NEN-016](done/NEN-016-document-and-fingerprint.md) | SubtitleDocument and timeline fingerprint | M | ✅ done | NEN-013 |
| [NEN-017](done/NEN-017-indexed-cue-lookup.md) | Indexed cue lookup | M | ✅ done | NEN-016 |
| [NEN-018](done/NEN-018-media-evidence-and-hash.md) | Media evidence, OS-compatible hash, release name parser | L | ✅ done | NEN-012 |
| [NEN-019](done/NEN-019-source-catalog.md) | SubtitleSourceCatalog with grouping and dedup | M | ✅ done | NEN-016 NEN-018 |
| [NEN-020](done/NEN-020-language-detection.md) | Subtitle language detection with confidence threshold | S | ✅ done | NEN-013 |

## M3 — macOS Vertical Slice

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-021](done/NEN-021-playback-port-contract.md) | PlaybackEngine port contract and capability model | M | ✅ done | NEN-012 |
| [NEN-022](done/NEN-022-libmpv-adapter.md) | libmpv playback adapter for macOS | L | ✅ done | NEN-021 NEN-004 |
| [NEN-023](done/NEN-023-track-enumeration.md) | Embedded track enumeration and selection | M | ✅ done | NEN-022 |
| [NEN-024](done/NEN-024-macos-shell.md) | macOS SwiftUI shell with transport controls | M | ✅ done | NEN-022 NEN-045 |
| [NEN-025](done/NEN-025-user-subtitle-loading.md) | User subtitle loading and sidecar discovery | M | ✅ done | NEN-013 NEN-015 NEN-024 |
| [NEN-026](done/NEN-026-subtitle-menu-ui.md) | Subtitle menu UI | M | ✅ done | NEN-019 NEN-025 NEN-056 |
| [NEN-027](done/NEN-027-subtitle-renderer.md) | SubtitleRenderer port and libmpv injection adapter | M | ✅ done | NEN-017 NEN-026 NEN-066 |
| [NEN-028](done/NEN-028-slice-acceptance.md) | macOS vertical slice acceptance | S | ✅ done | NEN-027 |
| [NEN-036](done/NEN-036-remote-evidence-port.md) | Remote media evidence port (HEAD, Content-Disposition, Range) | M | ✅ done | NEN-018 |
| [NEN-037](done/NEN-037-subtitle-language-preferences.md) | Subtitle language preference setting (primary and secondary) | S | ✅ done | NEN-019 NEN-024 NEN-047 |
| [NEN-039](done/NEN-039-language-group-granularity.md) | Subtitle menu language grouping and matching use the primary subtag | S | ✅ done | NEN-019 |
| [NEN-040](done/NEN-040-status-freshness-check.md) | check-docs.sh verifies STATUS.md's "Son doğrulama" date is not stale | S | ✅ done | NEN-031 |
| [NEN-041](done/NEN-041-doctor-detects-broken-swift.md) | doctor.sh reports an installed-but-unrunnable tool as missing | S | ✅ done | NEN-032 |
| [NEN-042](done/NEN-042-recent-media-list.md) | Recent media list in the empty state | S | ✅ done | NEN-024 |
| [NEN-043](done/NEN-043-libmpv-app-bundling.md) | Bundle libmpv into the .app and notarize | M | ✅ done | NEN-022 NEN-024 |
| [NEN-045](done/NEN-045-playback-session-ffi.md) | Core playback session across the FFI boundary | M | ✅ done | NEN-021 NEN-022 |
| [NEN-046](done/NEN-046-macos-window-lifecycle.md) | macOS window lifecycle — no inert app after the window closes | S | ✅ done | NEN-024 |
| [NEN-047](done/NEN-047-keyboard-shortcut-scope.md) | Scope playback shortcuts to the player surface and echo them on screen | S | ✅ done | NEN-024 |
| [NEN-048](done/NEN-048-transient-error-class.md) | Fix the spurious transient error and prove the transient error class | S | ✅ done | NEN-024 |
| [NEN-049](done/NEN-049-serialize-real-mpv-tests.md) | Serialize real libmpv platform tests | S | ✅ done | NEN-022 NEN-045 NEN-051 |
| [NEN-050](done/NEN-050-recent-media-transient-paths.md) | Cover the recent-media store's transient error paths | S | ✅ done | NEN-048 |
| [NEN-051](done/NEN-051-seek-answered-by-load-restart.md) | A seek is never answered by the load's own playback-restart | M | ✅ done | NEN-022 NEN-045 |
| [NEN-052](done/NEN-052-seek-while-loading.md) | Decide and pin what a seek during loading does | S | ✅ done | NEN-051 |
| [NEN-053](done/NEN-053-stale-position-overrides-seek.md) | A stale position event never overrides a seek that landed | S | ✅ done | NEN-051 |
| [NEN-054](done/NEN-054-volume-change-latency.md) | A volume change is heard when it is made | M | ✅ done | NEN-024 |
| [NEN-055](done/NEN-055-transport-feedback-latency.md) | A transport command is shown without waiting for the next poll tick | S | ✅ done | NEN-053 |
| [NEN-056](done/NEN-056-too-large-label-is-unreachable.md) | Resolve the unreachable "çok büyük" reason label | S | ✅ done | NEN-025 |
| [NEN-057](done/NEN-057-filename-language-hint.md) | Read the language a subtitle filename declares | S | ✅ done | NEN-025 |
| [NEN-058](done/NEN-058-media-fails-beside-symlinked-sidecar.md) | A medium fails to load when a symlinked sidecar sits beside it | S | ✅ done | NEN-022 NEN-025 |
| [NEN-059](done/NEN-059-restore-rust-ci-gates.md) | Restore the Rust CI gates | S | ✅ done | NEN-025 NEN-051 |
| [NEN-060](canceled/NEN-060-selected-track-not-drawn-fullscreen.md) | A selected embedded track is not drawn while the window is full screen | S | 🚫 canceled | NEN-022 NEN-026 |
| [NEN-061](done/NEN-061-frameless-chrome-and-glass-bar.md) | Frameless chrome and glass transport bar | M | ✅ done | NEN-024 |
| [NEN-062](done/NEN-062-subtitle-menu-panel.md) | Subtitle menu panel above the transport bar | M | ✅ done | NEN-026 NEN-061 |
| [NEN-063](canceled/NEN-063-video-quality-badge.md) | Video presentation metadata and quality badge | L | 🚫 canceled | NEN-045 NEN-061 |
| [NEN-065](done/NEN-065-flaky-controls-timing-test.md) | Stabilize the pinned-controls timing test | S | ✅ done | NEN-061 |
| [NEN-066](done/NEN-066-subtitle-not-composited-on-surface.md) | A subtitle mpv reports drawing is missing from the video surface | M | ✅ done | NEN-024 |
| [NEN-067](done/NEN-067-ultra-thin-flush-player-chrome.md) | Ultra-thin flush player chrome | L | ✅ done | NEN-061 NEN-062 NEN-065 NEN-066 |
| [NEN-068](done/NEN-068-media-sized-aspect-locked-window.md) | Media-sized, aspect-locked player window | L | ✅ done | NEN-067 |
| [NEN-069](done/NEN-069-clear-surface-without-video.md) | Clear the video surface when the medium has no video | S | ✅ done | — |
| [NEN-070](done/NEN-070-wide-transport-layout-overflow.md) | Keep transport controls inside wide windows | S | ✅ done | NEN-067 NEN-068 |
| [NEN-071](done/NEN-071-open-panel-resolves-links.md) | Decide what the subtitle file picker does with a link | S | ✅ done | NEN-025 |
| [NEN-073](done/NEN-073-player-chrome-boundaries.md) | Player chrome boundary behaviour | M | ✅ done | — |
| [NEN-074](done/NEN-074-smooth-live-resize.md) | Smooth live video resize | M | ✅ done | NEN-073 |
| [NEN-075](done/NEN-075-sidecar-language-suffix.md) | Sidecar discovery does not see a language-suffixed subtitle | S | ✅ done | NEN-025 NEN-057 |
| [NEN-076](done/NEN-076-status-freshness-check-on-shallow-clone.md) | The STATUS freshness check reads every done task as today on CI | S | ✅ done | NEN-040 |
| [NEN-077](done/NEN-077-safe-area-aspect-minimum.md) | Keep the minimum player surface aspect-correct across the titlebar safe area | M | ✅ done | NEN-068 NEN-073 |

## M4 — Stremio Handoff (macOS)

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-078](done/NEN-078-stremio-launch-measurement.md) | Measure how Stremio launches an external player on macOS | S | ✅ done | — |
| [NEN-079](done/NEN-079-adr-handoff-surface.md) | Decide the macOS handoff receiving surface | S | ✅ done | NEN-078 |
| [NEN-080](done/NEN-080-receive-opened-medium.md) | Receive a medium opened by another application | M | ✅ done | NEN-079 |
| [NEN-081](done/NEN-081-handoff-start-position.md) | Apply the start position a handoff carries | S | ✅ done | NEN-080 |
| [NEN-082](done/NEN-082-handoff-metadata-evidence.md) | Treat handoff metadata as optional evidence | M | ✅ done | NEN-080 |
| [NEN-083](done/NEN-083-handoff-never-logged.md) | Handoff input never reaches a log surface | S | ✅ done | NEN-080 |
| [NEN-084](done/NEN-084-m4-acceptance.md) | Stremio handoff acceptance | S | ✅ done | NEN-081 NEN-082 NEN-083 |
| [NEN-086](done/NEN-086-correct-stremio-mpv-launcher-measurement.md) | Correct Stremio MPV launcher measurement | S | ✅ done | — |
| [NEN-087](done/NEN-087-install-reversible-stremio-mpv-bridge.md) | Install reversible Stremio MPV bridge | M | ✅ done | NEN-086 |
| [NEN-088](done/NEN-088-stremio-to-nen-player-acceptance.md) | Stremio to Nen Player acceptance | S | ✅ done | NEN-087 |

## M5 — Translation Core

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-044](backlog/NEN-044-embedded-text-extraction.md) | Embedded subtitle text extraction | M | ⚪ backlog | NEN-023 |
| [NEN-072](backlog/NEN-072-remote-container-metadata.md) | Parse container metadata from remote byte windows | M | ⚪ backlog | NEN-036 |

## M6 — Real Providers

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-033](done/NEN-033-opensubtitles-hash-lookup.md) | OpenSubtitles hash-based identity lookup | M | ✅ done | NEN-018 |
| [NEN-034](backlog/NEN-034-ai-release-name-normalization.md) | AI-assisted release name normalization | M | ⚪ backlog | NEN-018 |
| [NEN-035](backlog/NEN-035-identity-confidence-and-candidates.md) | Identity confidence scoring and candidate ranking | M | ⚪ backlog | NEN-018 |
| [NEN-038](backlog/NEN-038-preferred-language-auto-download.md) | Auto-download subtitles for the preferred language | M | ⚪ backlog | NEN-019 NEN-033 NEN-035 NEN-036 |
| [NEN-064](backlog/NEN-064-verified-media-identity-strip.md) | Verified media identity in player chrome | L | ⚪ backlog | NEN-033 NEN-061 |

## Sıradaki uygun task'lar

Bağımlılıkları tamamlanmış, henüz başlanmamış task'lar:

- **NEN-034** — AI-assisted release name normalization
- **NEN-035** — Identity confidence scoring and candidate ranking
- **NEN-044** — Embedded subtitle text extraction
- **NEN-064** — Verified media identity in player chrome
- **NEN-072** — Parse container metadata from remote byte windows
