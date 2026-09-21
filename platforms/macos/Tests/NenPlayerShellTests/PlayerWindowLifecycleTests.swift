import AppKit
import NenPlaybackMPV
import SwiftUI
import Testing

@testable import NenPlayerShell

/// `NEN-141`: the product contract `NEN-046` wrote down — the red button and
/// `⌘W` stop playback and close the window, the process stays alive, and
/// Dock/`⌘O` bring the window back with a fresh session.
///
/// Asserted against a **real window**, because that is exactly where the
/// contract was breaking: `PlayerRootView.onDisappear` is not reliably called
/// when a SwiftUI `Window` scene is closed, so the session stayed alive and mpv
/// kept playing behind an invisible window. A hosting view with no window would
/// not reproduce that AppKit lifecycle at all.
@Suite("Player window lifecycle")
@MainActor
struct PlayerWindowLifecycleTests {
    /// The app's own window shape: full-size content under a hidden title bar,
    /// closable, so `performClose`/`close` exercise the real close path.
    private func window() -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1_024, height: 576),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: true
        )
        window.titleVisibility = .hidden
        window.titlebarAppearsTransparent = true
        // Programmatic windows default to releasing themselves on close; the
        // test keeps its own reference to inspect the outcome, so AppKit must
        // not over-release it from under ARC.
        window.isReleasedWhenClosed = false
        return window
    }

    /// SwiftUI installs representable views on a later AppKit turn. Poll only
    /// until the requested state exists instead of holding the shared main
    /// actor for a fixed delay; other timing tests run in parallel.
    @discardableResult
    private func waitUntil(
        timeout: TimeInterval = 1,
        _ condition: () -> Bool
    ) -> Bool {
        let deadline = Date().addingTimeInterval(timeout)
        while !condition(), Date() < deadline {
            RunLoop.current.run(mode: .default, before: Date().addingTimeInterval(0.01))
        }
        return condition()
    }

    private func videoSurface(in view: NSView) -> MPVVideoView? {
        if let surface = view as? MPVVideoView { return surface }
        for child in view.subviews {
            if let found = videoSurface(in: child) { return found }
        }
        return nil
    }

    private func host(_ model: PlayerModel, in window: NSWindow) -> NSHostingView<PlayerRootView> {
        let host = NSHostingView(rootView: PlayerRootView(model: model))
        window.contentView = host
        host.frame = window.contentLayoutRect
        host.layoutSubtreeIfNeeded()
        return host
    }

    @Test("closing the player window shuts the playback session down")
    func closingTheWindowShutsTheSessionDown() {
        let session = FakeSession()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            sessionFactory: { _ in session }
        )

        // Held strongly for the whole test: the app's SwiftUI scene keeps its
        // hosting view alive after the window closes (that is why
        // `onDisappear` cannot be relied on here), so the surface the model
        // resumes onto later must survive the close the same way.
        let window = window()
        let host = host(model, in: window)
        #expect(waitUntil { videoSurface(in: host) != nil })
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        #expect(session.loadedLocators.count == 1)
        #expect(model.hasMedia)

        window.close()

        // `NSWindow.willClose` reaches the writer through `queue: .main`, so
        // `waitUntil` rather than a bare assertion keeps this honest whether
        // the queue delivers on this turn or the next.
        #expect(
            waitUntil { session.shutdownCount == 1 },
            "the window closed but the session kept playing"
        )
        #expect(!model.hasMedia)
        #expect(model.mediaName == nil)
        #expect(model.positionMilliseconds == 0)
    }

    @Test("reopening after a close starts a fresh session and plays again")
    func reopeningAfterACloseStartsAFreshSession() throws {
        let spy = SessionFactorySpy()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            sessionFactory: { _ in spy.make() }
        )

        let window = window()
        let host = host(model, in: window)
        #expect(waitUntil { videoSurface(in: host) != nil })
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        #expect(spy.made.count == 1)
        let first = try #require(spy.made.first)
        #expect(first.loadedLocators.count == 1)

        window.close()
        #expect(waitUntil { first.shutdownCount == 1 })

        // `applicationShouldHandleReopen`'s path (`NEN-046`): the surface never
        // went away, so `resume()` must build a new session on it.
        model.resume()
        #expect(spy.made.count == 2)
        let second = try #require(spy.made.dropFirst().first)
        #expect(second.shutdownCount == 0)

        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
        #expect(second.loadedLocators.count == 1)
        #expect(model.hasMedia)
    }
}

/// Records every session the model builds, so "a fresh session after reopen"
/// is a fact rather than an inference from state.
@MainActor
private final class SessionFactorySpy {
    private(set) var made: [FakeSession] = []

    func make() -> FakeSession {
        let session = FakeSession()
        made.append(session)
        return session
    }
}
