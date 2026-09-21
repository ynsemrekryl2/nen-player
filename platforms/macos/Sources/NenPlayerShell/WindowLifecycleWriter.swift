import AppKit
import SwiftUI

/// Calls back when the window this view is attached to closes (`NEN-141`).
///
/// SwiftUI's `onDisappear` is not reliably called when a `Window` scene's
/// window is closed: measured on the real app, the scene's content stays
/// mounted, so `PlayerRootView`'s `onDisappear` never ran and the playback
/// session outlived the window it belonged to — mpv kept playing behind an
/// invisible window, and the Dock brought the same, still-playing session
/// back. `NSWindow.willClose` is the AppKit fact that actually happens, so
/// the cleanup hangs off it.
///
/// The same shape as `WindowGeometryWriter`, for the same reason: SwiftUI has
/// no vocabulary for the window's own lifecycle, so the one piece of AppKit
/// needed is reached through a backing `NSView` rather than by giving the
/// model a window reference it would then have to keep alive.
///
/// **Observation is synchronous.** Unlike the geometry writer, nothing here
/// resizes a window from inside SwiftUI's layout pass, so deferring the
/// registration by a run-loop turn would only widen the window in which a
/// close lands before the observer exists — the exact case this type exists
/// for.
///
/// **It decides nothing.** The callback belongs to the root view; what runs
/// there is `PlayerModel.shutdown()`.
struct WindowLifecycleWriter: NSViewRepresentable {
    let onWindowClose: @MainActor () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context _: Context) -> AttachmentView {
        AttachmentView()
    }

    func updateNSView(_ nsView: AttachmentView, context: Context) {
        let onClose = onWindowClose
        let observe = { (window: NSWindow) in
            context.coordinator.observe(window, onClose: onClose)
        }
        nsView.onWindowChange = observe
        if let window = nsView.window {
            observe(window)
        }
    }

    static func dismantleNSView(_ nsView: AttachmentView, coordinator: Coordinator) {
        nsView.onWindowChange = nil
        coordinator.stopObserving()
    }

    @MainActor
    final class AttachmentView: NSView {
        var onWindowChange: ((NSWindow) -> Void)?

        override func viewDidMoveToWindow() {
            super.viewDidMoveToWindow()
            guard let window else { return }
            onWindowChange?(window)
        }
    }

    @MainActor
    final class Coordinator {
        private var observation: NSObjectProtocol?
        private weak var observed: NSWindow?

        /// Re-pointed when the view moves to another window; the close
        /// notification is scoped to that one window, so Settings or Olaylar
        /// closing never reaches the player's session.
        func observe(_ window: NSWindow, onClose: @escaping @MainActor () -> Void) {
            guard observed !== window else { return }
            stopObserving()
            observed = window
            observation = NotificationCenter.default.addObserver(
                forName: NSWindow.willCloseNotification,
                object: window,
                queue: .main
            ) { _ in
                MainActor.assumeIsolated {
                    onClose()
                }
            }
        }

        func stopObserving() {
            if let observation {
                NotificationCenter.default.removeObserver(observation)
            }
            observation = nil
            observed = nil
        }
    }
}