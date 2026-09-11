import NenCore

/// The narrow shell-facing surface of the Rust-owned playback session.
///
/// Keeping the UI on this protocol lets shell behaviour be tested without a
/// real libmpv process. Production still uses `FfiPlaybackSession` directly.
///
/// `Sendable`: `translateSelectedSubtitle()` (`NEN-044`) calls
/// `prepareEmbeddedDocument` from inside a detached task, off the main actor,
/// for the same reason it already runs `FfiTranslationEngine.start` there —
/// a real extraction demuxes a container and must not block the UI thread.
/// `FfiPlaybackSession` is `@unchecked Sendable` in the generated bindings;
/// a test double conforms the same way (`ShellTestSupport.swift`).
public protocol PlaybackSessionClient: AnyObject, Sendable {
    func load(locator: String) throws
    func play() throws
    func pause() throws
    func stop() throws
    func seek(toMs: UInt64) throws
    func positionMs() throws -> UInt64
    func durationMs() throws -> UInt64?
    func state() throws -> FfiPlaybackState
    /// The display size of the video being played, or `nil` when there is no
    /// video (ADR-0038). Re-read after every `videoGeometryChanged` and after
    /// an `eventsLost` resync — the event carries no value.
    func videoGeometry() throws -> FfiVideoGeometry?
    func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor]
    /// Puts the subtitle a menu row names on screen.
    ///
    /// The shell hands over its library and a token; which kind of row it is,
    /// and therefore whether the engine selects a track or is given a document
    /// to draw, is the core's decision (ADR-0013 Karar 3).
    func showSubtitle(library: FfiSubtitleLibrary, token: UInt32) throws -> FfiShowOutcome
    /// Makes sure the row a token names has a document to translate,
    /// extracting one from the engine when it is an embedded track that does
    /// not have one yet (`NEN-044`). Call this before starting a translation
    /// on a row that might be an embedded track — the core has otherwise
    /// never asked the engine, and `FfiTranslationEngine.start` would refuse
    /// with `NoDocument`.
    func prepareEmbeddedDocument(
        library: FfiSubtitleLibrary,
        token: UInt32
    ) throws -> FfiPrepareOutcome
    /// §8's `Kapalı`: nothing on screen, whatever was drawing it.
    func hideSubtitle() throws
    /// The share of the surface height this shell's own chrome covers, so the
    /// subtitle is not drawn underneath it (ADR-0037).
    func setSubtitleBottomInset(fraction: Float) throws
    func setRate(rate: Float) throws
    func setVolume(volume: Float) throws
    func drainEvents() -> [FfiSessionEvent]
    func shutdown() throws
}

extension FfiPlaybackSession: PlaybackSessionClient {}
