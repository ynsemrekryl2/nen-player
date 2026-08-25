# Task Index

<!-- ÜRETİLEN DOSYA — elle düzenlemeyin.
     Yenilemek için: bash scripts/task-index.sh -->

Toplam **36** task · ✅ done 21 · 🔵 active 1 · ⛔ blocked 0 · ⚪ backlog 14

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

## M2 — Subtitle Core

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-013](done/NEN-013-strict-srt-parser.md) | Strict SRT parser | M | ✅ done | NEN-012 |
| [NEN-014](done/NEN-014-webvtt-writer.md) | WebVTT writer | S | ✅ done | NEN-013 |
| [NEN-015](done/NEN-015-encoding-detection.md) | Encoding detection and sanitization | M | ✅ done | NEN-013 |
| [NEN-016](done/NEN-016-document-and-fingerprint.md) | SubtitleDocument and timeline fingerprint | M | ✅ done | NEN-013 |
| [NEN-017](done/NEN-017-indexed-cue-lookup.md) | Indexed cue lookup | M | ✅ done | NEN-016 |
| [NEN-018](active/NEN-018-media-evidence-and-hash.md) | Media evidence, OS-compatible hash, release name parser | L | 🔵 active | NEN-012 |
| [NEN-019](backlog/NEN-019-source-catalog.md) | SubtitleSourceCatalog with grouping and dedup | M | ⚪ backlog | NEN-016 NEN-018 |
| [NEN-020](backlog/NEN-020-language-detection.md) | Subtitle language detection with confidence threshold | S | ⚪ backlog | NEN-013 |

## M3 — macOS Vertical Slice

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-021](backlog/NEN-021-playback-port-contract.md) | PlaybackEngine port contract and capability model | M | ⚪ backlog | NEN-012 |
| [NEN-022](backlog/NEN-022-libmpv-adapter.md) | libmpv playback adapter for macOS | L | ⚪ backlog | NEN-021 NEN-004 |
| [NEN-023](backlog/NEN-023-track-enumeration.md) | Embedded track enumeration and selection | M | ⚪ backlog | NEN-022 |
| [NEN-024](backlog/NEN-024-macos-shell.md) | macOS SwiftUI shell with transport controls | M | ⚪ backlog | NEN-022 |
| [NEN-025](backlog/NEN-025-user-subtitle-loading.md) | User subtitle loading and sidecar discovery | M | ⚪ backlog | NEN-013 NEN-015 NEN-024 |
| [NEN-026](backlog/NEN-026-subtitle-menu-ui.md) | Subtitle menu UI | M | ⚪ backlog | NEN-019 NEN-025 |
| [NEN-027](backlog/NEN-027-subtitle-renderer.md) | SubtitleRenderer port and libmpv injection adapter | M | ⚪ backlog | NEN-017 NEN-026 |
| [NEN-028](backlog/NEN-028-slice-acceptance.md) | macOS vertical slice acceptance | S | ⚪ backlog | NEN-027 |
| [NEN-036](backlog/NEN-036-remote-evidence-port.md) | Remote media evidence port (HEAD, Content-Disposition, Range) | M | ⚪ backlog | NEN-018 |

## M6 — Real Providers

| ID | Başlık | Boyut | Durum | Bağımlılık |
|---|---|---|---|---|
| [NEN-033](backlog/NEN-033-opensubtitles-hash-lookup.md) | OpenSubtitles hash-based identity lookup | M | ⚪ backlog | NEN-018 |
| [NEN-034](backlog/NEN-034-ai-release-name-normalization.md) | AI-assisted release name normalization | M | ⚪ backlog | NEN-018 |
| [NEN-035](backlog/NEN-035-identity-confidence-and-candidates.md) | Identity confidence scoring and candidate ranking | M | ⚪ backlog | NEN-018 |

## Sıradaki uygun task'lar

Bağımlılıkları tamamlanmış, henüz başlanmamış task'lar:

- **NEN-020** — Subtitle language detection with confidence threshold
- **NEN-021** — PlaybackEngine port contract and capability model
