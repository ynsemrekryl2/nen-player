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
    func setVolume(volume: Float) throws
    func drainEvents() -> [FfiSessionEvent]
    func shutdown() throws
}

extension FfiPlaybackSession: PlaybackSessionClient {}
