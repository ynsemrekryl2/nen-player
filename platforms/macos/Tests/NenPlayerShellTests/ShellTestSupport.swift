import AppKit
import Foundation
import NenCore
@testable import NenPlayerShell

// Shared scaffolding for the shell suites.
//
// One fake, one temp directory helper, one clock — deliberately not a copy per
// test file. A second `FakeSession` would drift from this one the first time
// the protocol grew a method, and the two suites would then be testing two
// different shells.

/// A thread-safe counter, so temp directories cannot collide.
final class Counter: @unchecked Sendable {
    private let lock = NSLock()
    private var value = 0

    func next() -> Int {
        lock.lock()
        defer { lock.unlock() }
        value += 1
        return value
    }
}

/// A real directory with real files in it.
///
/// NEN-025's gates answer questions about the filesystem, so a shell test that
/// wants a real verdict has to hand them something real to look at.
struct TempFixture {
    static let validSrt = "1\n00:00:01,000 --> 00:00:02,000\nHello there.\n"
    /// Long enough for NEN-020's detector to clear its confidence threshold,
    /// so the source lands in the Turkish group rather than `Dil Belirsiz`.
    static let turkishSrt = """
        1
        00:00:01,000 --> 00:00:04,000
        Bu akşam eve geç kalacağımı söylemiştim ama kimse dinlemedi.

        2
        00:00:05,000 --> 00:00:08,000
        Kapıyı açtığımda salonun ışıkları hâlâ yanıyordu.

        3
        00:00:09,000 --> 00:00:12,000
        Masanın üstünde soğumuş bir çay bardağı duruyordu.

        4
        00:00:13,000 --> 00:00:16,000
        Pencereden dışarı baktım, yağmur yeniden başlamıştı.

        5
        00:00:17,000 --> 00:00:20,000
        Bu şehirde geçirdiğim yılları düşünmeden edemedim.

        """

    /// Process id **and** a counter.
    ///
    /// The tag alone is not unique: swift-testing runs tests in parallel, so
    /// two tests that happen to pick the same tag would share a directory —
    /// and `init` wipes it, which means one test deletes the other's fixtures
    /// halfway through. The Rust suites number their temp directories for the
    /// same reason.
    private static let counter = Counter()

    let url: URL

    init(_ tag: String) {
        let unique = Self.counter.next()
        let pid = ProcessInfo.processInfo.processIdentifier
        url = URL(fileURLWithPath: NSTemporaryDirectory())
            .appendingPathComponent("nen-shell-\(tag)-\(pid)-\(unique)")
        try? FileManager.default.removeItem(at: url)
        try! FileManager.default.createDirectory(at: url, withIntermediateDirectories: true)
    }

    @discardableResult
    func write(_ name: String, _ contents: String) -> URL {
        let file = url.appendingPathComponent(name)
        try! contents.write(to: file, atomically: true, encoding: .utf8)
        return file
    }

    func symlink(_ name: String, to target: URL) -> URL {
        let link = url.appendingPathComponent(name)
        try! FileManager.default.createSymbolicLink(at: link, withDestinationURL: target)
        return link
    }

    func remove() {
        try? FileManager.default.removeItem(at: url)
    }
}

/// A clock a test moves by hand, so the seek guard's expiry is reachable
/// without waiting for it.
final class TestClock {
    var nanoseconds: UInt64 = 0
}

final class MemoryRecentStore: RecentMediaStoring {
    var url: URL?
    var displayName: String? { url?.lastPathComponent }

    func save(_ url: URL) throws {
        self.url = url
    }

    func resolve() throws -> URL? {
        url
    }

    func clear() {
        url = nil
    }
}

/// The session calls a test can make fail, so the shell's refusal paths run.
enum FakeSessionCall: Hashable {
    case load, play, pause, stop, seek, position, duration, state, tracks, showSubtitle,
        hideSubtitle, volume
}

/// What ended up on screen, as the fake session saw it.
enum DrawnSubtitle: Equatable {
    /// One of the medium's own tracks, by engine id.
    case track(UInt32)
    /// A document, by the row that named it.
    case document(UInt32)
    /// Nothing — §8's `Kapalı`.
    case off
}

final class FakeSession: PlaybackSessionClient {
    /// Errors keyed by call: every listed call throws instead of succeeding.
    var errors: [FakeSessionCall: FfiPlaybackError] = [:]

    var loadedLocators: [String] = []
    var playCount = 0
    var pauseCount = 0
    var seekTargets: [UInt64] = []
    var volumes: [Float] = []
    /// Every bottom inset the shell declared, in order (ADR-0037).
    var subtitleBottomInsets: [Float] = []
    var currentPosition: UInt64 = 0
    var currentDuration: UInt64? = 30_008
    var currentState: FfiPlaybackState = .ready
    var requestedTrackKinds: [FfiTrackKind] = []
    /// What `tracks(kind: .subtitle)` reports. Empty by default so every test
    /// written before NEN-026 keeps meaning what it meant.
    var subtitleTracks: [FfiTrackDescriptor] = []
    /// Everything the shell asked to be put on screen, in order.
    ///
    /// The fake makes the same decision the core session makes — an embedded
    /// row is a track, anything else is a document — because a fake that
    /// merely recorded the token would let the shell stop distinguishing them
    /// without any test noticing.
    var drawnSubtitles: [DrawnSubtitle] = []
    var events: [FfiSessionEvent] = []
    var shutdownCount = 0

    private func refuse(_ call: FakeSessionCall) throws {
        if let error = errors[call] { throw error }
    }

    func load(locator: String) throws {
        try refuse(.load)
        loadedLocators.append(locator)
    }
    func play() throws {
        try refuse(.play)
        playCount += 1
        currentState = .playing
        // The real adapter appends the transition before the command returns
        // and the bridge pulls right after it, so it is queued by the time the
        // caller gets control back. Measured at 50-180 us (NEN-055).
        events.append(.stateChanged(state: .playing))
    }
    func pause() throws {
        try refuse(.pause)
        pauseCount += 1
        currentState = .paused
        events.append(.stateChanged(state: .paused))
    }
    func stop() throws {
        try refuse(.stop)
        currentState = .idle
    }
    func seek(toMs: UInt64) throws {
        try refuse(.seek)
        seekTargets.append(toMs)
        currentPosition = toMs
    }
    func positionMs() throws -> UInt64 {
        try refuse(.position)
        return currentPosition
    }
    func durationMs() throws -> UInt64? {
        try refuse(.duration)
        return currentDuration
    }
    func state() throws -> FfiPlaybackState {
        try refuse(.state)
        return currentState
    }
    func tracks(kind: FfiTrackKind) throws -> [FfiTrackDescriptor] {
        try refuse(.tracks)
        requestedTrackKinds.append(kind)
        return kind == .subtitle ? subtitleTracks : []
    }
    func showSubtitle(library: FfiSubtitleLibrary, token: UInt32) throws -> FfiShowOutcome {
        try refuse(.showSubtitle)
        guard library.isUsable(token: token) else { return .unusable }
        if let track = library.embeddedTrackOf(token: token) {
            drawnSubtitles.append(.track(track))
        } else {
            drawnSubtitles.append(.document(token))
        }
        return .shown
    }
    func hideSubtitle() throws {
        try refuse(.hideSubtitle)
        drawnSubtitles.append(.off)
    }
    func setSubtitleBottomInset(fraction: Float) throws {
        subtitleBottomInsets.append(fraction)
    }
    func setVolume(volume: Float) throws {
        try refuse(.volume)
        volumes.append(volume)
    }
    func drainEvents() -> [FfiSessionEvent] {
        defer { events.removeAll() }
        return events
    }
    func shutdown() throws { shutdownCount += 1 }
}
