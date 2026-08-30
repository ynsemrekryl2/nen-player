import NenCore

/// The narrow shell-facing surface of the Rust-owned playback session.
///
/// Keeping the UI on this protocol lets shell behaviour be tested without a
/// real libmpv process. Production still uses `FfiPlaybackSession` directly.
public protocol PlaybackSessionClient: AnyObject {
    func load(locator: String) throws
    func play() throws
    func pause() throws
    func stop() throws
    func seek(toMs: UInt64) throws
    func positionMs() throws -> UInt64
    func durationMs() throws -> UInt64?
    func state() throws -> FfiPlaybackState
    func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor]
    /// Puts the subtitle a menu row names on screen.
    ///
    /// The shell hands over its library and a token; which kind of row it is,
    /// and therefore whether the engine selects a track or is given a document
    /// to draw, is the core's decision (ADR-0013 Karar 3).
    func showSubtitle(library: FfiSubtitleLibrary, token: UInt32) throws -> FfiShowOutcome
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
