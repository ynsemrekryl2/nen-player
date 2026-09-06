import AppKit
import Foundation
import NenCore
import SwiftUI
import Testing

@testable import NenPlaybackMPV
@testable import NenPlayerShell

/// NEN-068: the picture fills the window, all the way to its edges.
///
/// The aspect lock is worthless if the surface it is locked to is not the
/// surface the video is drawn into. Measured on the real app: with the window
/// correctly locked to 16:9 for a 1920x1080 medium, the picture still sat
/// inside a ~28 pt black margin on the left and right and ~32 pt at the top —
/// at **every** size and every resolution, from the moment the medium opened.
///
/// The cause was the shell's own layout, not the renderer: `Color.black`
/// ignored the window's safe area and the video surface did not, so the black
/// behind it showed through as a letterbox the player drew for itself.
///
/// This is asserted against a **real window**, because a hosting view with no
/// window has no safe area and would make the check vacuous — which is exactly
/// how the defect survived until it was seen by eye.
@Suite("Video surface fill")
@MainActor
struct VideoSurfaceFillTests {
    /// A window shaped like the app's: full-size content under a hidden title
    /// bar, which is where the top safe-area inset comes from.
    private func window() -> NSWindow {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 1_024, height: 576),
            styleMask: [.titled, .closable, .miniaturizable, .resizable, .fullSizeContentView],
            backing: .buffered,
            defer: true
        )
        window.titleVisibility = .hidden
        window.titlebarAppearsTransparent = true
        return window
    }

    /// The video surface somewhere under `view`.
    private func videoSurface(in view: NSView) -> MPVVideoView? {
        if let surface = view as? MPVVideoView { return surface }
        for child in view.subviews {
            if let found = videoSurface(in: child) { return found }
        }
        return nil
    }

    @Test("the video surface covers the whole content view, safe area included")
    func theSurfaceFillsTheWindow() throws {
        let session = FakeSession()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            sessionFactory: { _ in session }
        )
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))

        let window = window()
        let host = NSHostingView(rootView: PlayerRootView(model: model))
        window.contentView = host
        host.frame = window.contentLayoutRect
        host.layoutSubtreeIfNeeded()
        // The layout the video surface belongs to is created on a later pass.
        RunLoop.current.run(until: Date().addingTimeInterval(0.3))

        let content = try #require(window.contentView, "the window has no content view")
        let surface = try #require(
            videoSurface(in: content),
            "the player never created a video surface to measure"
        )
        let covered = surface.convert(surface.bounds, to: content)

        // Every edge, because the defect was on three of them and a check that
        // looked at one would have passed while the picture was still boxed.
        #expect(
            abs(covered.minX - content.bounds.minX) < 1,
            "a \(covered.minX - content.bounds.minX) pt black margin on the left"
        )
        #expect(
            abs(covered.maxX - content.bounds.maxX) < 1,
            "a \(content.bounds.maxX - covered.maxX) pt black margin on the right"
        )
        #expect(
            abs(covered.minY - content.bounds.minY) < 1,
            "a \(covered.minY - content.bounds.minY) pt black margin at the bottom"
        )
        #expect(
            abs(covered.maxY - content.bounds.maxY) < 1,
            "a \(content.bounds.maxY - covered.maxY) pt black margin at the top"
        )
    }

    @Test("the window really does have a safe area to ignore")
    func theCheckIsNotVacuous() throws {
        // The negative control. Without a safe area there is nothing for the
        // video surface to ignore, and the test above would pass on a shell
        // that had never been fixed. What is asserted is that this window
        // inset the content — which is the pressure the fix is under.
        let window = window()
        let host = NSHostingView(rootView: Color.clear)
        window.contentView = host
        host.layoutSubtreeIfNeeded()

        let insets = host.safeAreaInsets
        #expect(
            insets.top > 0 || insets.left > 0 || insets.right > 0 || insets.bottom > 0,
            """
            this window reports no safe area at all (\(insets)), so the fill \
            test above proves nothing — give it a window that does
            """
        )
    }
}
