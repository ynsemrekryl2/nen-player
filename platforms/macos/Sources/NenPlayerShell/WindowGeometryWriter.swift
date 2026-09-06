import AppKit
import NenCore
import SwiftUI

/// Applies [`WindowGeometry`]'s answers to the real window (ADR-0038 Karar 4).
///
/// The same shape `WindowTitleWriter` has, for the same reason: SwiftUI has no
/// vocabulary for an aspect-locked window, so the one piece of AppKit the shell
/// needs is reached through a backing `NSView` rather than by giving the model
/// a window reference it would then have to keep alive.
///
/// **It decides nothing.** Every number comes from `WindowGeometry`, which is
/// pure and tested on its own; what is left here is the applying, and the one
/// rule that cannot be expressed as arithmetic — *when* to resize.
struct WindowGeometryWriter: NSViewRepresentable {
    /// The picture's display size, or `nil` when there is no picture.
    let geometry: FfiVideoGeometry?
    /// Changes once per medium that reaches the session, even when two files
    /// share a basename. This is what makes "a new medium" a fact rather than
    /// an inference from the size having changed — two 16:9 films in a row are
    /// two openings, and a stream that reconfigures mid-playback is not one.
    let mediaRevision: UInt64

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context _: Context) -> NSView {
        NSView()
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        let geometry = geometry
        let mediaRevision = mediaRevision
        // Deferred for `WindowTitleWriter`'s reason: during `updateNSView` the
        // view is not reliably in a window yet, and resizing one from inside a
        // SwiftUI layout pass re-enters that pass.
        DispatchQueue.main.async {
            guard let window = nsView.window else { return }
            context.coordinator.apply(
                geometry: geometry,
                mediaRevision: mediaRevision,
                to: window
            )
        }
    }

    static func dismantleNSView(_: NSView, coordinator: Coordinator) {
        coordinator.stopObserving()
    }

    @MainActor
    final class Coordinator {
        /// The medium the window has already been sized for.
        ///
        /// `nil` until the first one, so the very first medium is a resize and
        /// not a no-op. Compared rather than counted: an update can run many
        /// times per medium, and only the first of them may move the window.
        private var sizedForRevision: UInt64?
        // AppKit clears `.fullScreen` before its exit animation finishes.
        // Keep geometry writes suspended from will-enter through did-exit.
        private var suspendedForFullScreen = false
        private var latestGeometry: FfiVideoGeometry?
        private var latestRevision: UInt64 = 0
        private var observations: [NSObjectProtocol] = []
        private weak var observed: NSWindow?

        func apply(geometry: FfiVideoGeometry?, mediaRevision: UInt64, to window: NSWindow) {
            observe(window)
            latestGeometry = geometry
            latestRevision = mediaRevision
            guard !suspendedForFullScreen, !window.styleMask.contains(.fullScreen) else { return }

            guard let geometry,
                  let size = Self.displaySize(geometry) else {
                // Audio-only, a failed medium, or nothing open: there is no
                // picture, so there is no shape to hold. The SwiftUI root owns
                // the chrome floor; this writer only releases the aspect lock.
                Self.releaseAspectLock(in: window)
                // **The revision is deliberately not claimed here.** Opening a
                // medium clears the size and bumps the revision in the same
                // turn — the shell cannot know the new picture's shape until
                // the engine announces it — so this branch runs first for every
                // single medium. Marking the revision sized here would consume
                // the one chance the medium had to open at its own size, and
                // the announcement arriving moments later would find its work
                // already "done". Measured on the real app: the window kept the
                // previous frame and pillarboxed the picture inside it.
                return
            }

            window.contentAspectRatio = size

            // **Only on a new medium.** A later reconfiguration of the same
            // stream updates the lock and the floor — so the window still
            // cannot be dragged into a letterbox — but does not move a window
            // the user has already placed and sized.
            guard sizedForRevision != mediaRevision else { return }
            sizedForRevision = mediaRevision
            resize(window, to: size)
        }

        private static func releaseAspectLock(in window: NSWindow) {
            // Setting contentAspectRatio to zero leaves resizeIncrements at
            // (0, 0). On macOS 27 that stalls native full-screen exit. AppKit's
            // documented reset cancels the ratio by restoring unit increments.
            window.resizeIncrements = NSSize(width: 1, height: 1)
        }

        private func resize(_ window: NSWindow, to media: CGSize) {
            let visible = (window.screen ?? NSScreen.main)?.visibleFrame ?? .zero
            let content = WindowGeometry.contentSize(for: media, visibleFrame: visible)
            // Through `frameRect(forContentRect:)` rather than `setContentSize`
            // plus a separate move: the two would be two animations and the
            // window would visibly step. One `setFrame` is one movement.
            let currentFrame = window.frame
            let framed = window.frameRect(
                forContentRect: CGRect(origin: .zero, size: content)
            )
            let placed = WindowGeometry.recentredFrame(
                size: framed.size,
                around: currentFrame,
                visibleFrame: visible
            )
            window.setFrame(placed, display: true, animate: false)
        }

        /// Releases the lock for full screen and restores it on the way out.
        ///
        /// Notifications rather than a `styleMask` check on the next update:
        /// leaving full screen does not itself schedule a SwiftUI update, so a
        /// window that came back would keep the released lock until something
        /// else happened to redraw — and stay resizable into a letterbox for as
        /// long as that took.
        private func observe(_ window: NSWindow) {
            guard observed !== window else { return }
            stopObserving()
            observed = window
            suspendedForFullScreen = window.styleMask.contains(.fullScreen)

            let centre = NotificationCenter.default
            observations = [
                centre.addObserver(
                    forName: NSWindow.willEnterFullScreenNotification,
                    object: window,
                    queue: .main
                ) { [weak self] _ in
                    MainActor.assumeIsolated {
                        self?.suspendedForFullScreen = true
                        Self.releaseAspectLock(in: window)
                    }
                },
                centre.addObserver(
                    forName: NSWindow.didExitFullScreenNotification,
                    object: window,
                    queue: .main
                ) { [weak self] _ in
                    MainActor.assumeIsolated {
                        guard let self else { return }
                        self.suspendedForFullScreen = false
                        self.apply(
                            geometry: self.latestGeometry,
                            mediaRevision: self.latestRevision,
                            to: window
                        )
                    }
                }
            ]
        }

        func stopObserving() {
            for observation in observations {
                NotificationCenter.default.removeObserver(observation)
            }
            observations = []
            observed = nil
        }

        /// The size as points, refusing anything degenerate.
        ///
        /// The core already refuses a zero dimension, so this is the boundary
        /// keeping its own promise rather than distrusting it: dividing by a
        /// height that reached here as `0` would put `nan` into a window frame.
        private static func displaySize(_ geometry: FfiVideoGeometry) -> CGSize? {
            guard geometry.width > 0, geometry.height > 0 else { return nil }
            return CGSize(width: CGFloat(geometry.width), height: CGFloat(geometry.height))
        }
    }
}
