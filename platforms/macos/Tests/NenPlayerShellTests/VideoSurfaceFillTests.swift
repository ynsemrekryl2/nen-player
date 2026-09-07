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

    private func windowAttachmentView(in view: NSView) -> WindowGeometryWriter.WindowAttachmentView? {
        if let attachment = view as? WindowGeometryWriter.WindowAttachmentView { return attachment }
        for child in view.subviews {
            if let found = windowAttachmentView(in: child) { return found }
        }
        return nil
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
        #expect(waitUntil { videoSurface(in: host) != nil })

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

    @Test("the minimum window, hosting root and video surface keep one aspect ratio")
    func theMinimumWindowKeepsEverySurfaceAligned() throws {
        let geometries = [
            FfiVideoGeometry(width: 160, height: 90),
            FfiVideoGeometry(width: 160, height: 120),
            FfiVideoGeometry(width: 239, height: 100),
            FfiVideoGeometry(width: 90, height: 160),
        ]

        for geometry in geometries {
            let session = FakeSession()
            session.currentState = .ready
            session.currentVideoGeometry = geometry
            let model = PlayerModel(
                startsPolling: false,
                managesCursor: false,
                preferenceStore: MemoryPreferenceStore(),
                sessionFactory: { _ in session }
            )

            let window = window()
            var transportFrames: [TransportLayoutElement: CGRect] = [:]
            let host = NSHostingView(
                rootView: PlayerRootView(model: model) { transportFrames = $0 }
            )
            window.contentView = host
            defer {
                model.shutdown()
                window.close()
            }
            host.layoutSubtreeIfNeeded()
            #expect(waitUntil { windowAttachmentView(in: host) != nil })
            // Production learns the display geometry after the SwiftUI
            // hierarchy is attached to its window. Trigger the same sequence
            // so the writer measures the real titlebar safe area.
            model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/contract-clip.mkv"))
            model.consume([
                .stateChanged(state: .ready),
                .videoGeometryChanged,
            ])
            #expect(waitUntil {
                window.contentAspectRatio
                    == NSSize(width: CGFloat(geometry.width), height: CGFloat(geometry.height))
                    && videoSurface(in: host) != nil
                    && transportFrames[.bar] != nil
            })
            #expect(model.videoGeometry == geometry)
            _ = try #require(
                windowAttachmentView(in: host),
                "SwiftUI never created the window attachment view"
            )
            #expect(
                window.contentAspectRatio
                    == NSSize(width: CGFloat(geometry.width), height: CGFloat(geometry.height)),
                "the attached window writer never applied \(geometry)"
            )

            // Push past every supported minimum. SwiftUI and AppKit must
            // settle on one safe-area-aware, aspect-correct answer.
            window.setFrame(
                NSRect(origin: window.frame.origin, size: NSSize(width: 300, height: 200)),
                display: false
            )
            host.layoutSubtreeIfNeeded()
            #expect(waitUntil {
                host.layoutSubtreeIfNeeded()
                guard let surface = videoSurface(in: host) else { return false }
                let covered = surface.convert(surface.bounds, to: host)
                return abs(covered.width - host.bounds.width) < 1
                    && abs(covered.height - host.bounds.height) < 1
                    && transportFrames[.bar]?.width == host.frame.width
            })

            let content = try #require(window.contentView, "the window has no content view")
            let surface = try #require(
                videoSurface(in: content),
                "the player never created a video surface to measure"
            )
            let covered = surface.convert(surface.bounds, to: content)
            let ratio = CGFloat(geometry.width) / CGFloat(geometry.height)
            let insets = content.safeAreaInsets
            let overhead = CGSize(
                width: insets.left + insets.right,
                height: insets.top + insets.bottom
            )
            let expected = WindowGeometry.minimumContentSize(
                for: CGSize(width: CGFloat(geometry.width), height: CGFloat(geometry.height)),
                safeAreaOverhead: overhead
            )

            #expect(abs(window.frame.width - expected.width) <= 1)
            #expect(abs(window.frame.height - expected.height) <= 1)
            #expect(
                abs(window.frame.width / window.frame.height - ratio) < 0.01,
                "the minimum window \(window.frame.size) is not ratio \(ratio)"
            )
            #expect(
                abs(host.frame.width - window.frame.width) < 1
                    && abs(host.frame.height - window.frame.height) < 1,
                "the hosting root \(host.frame.size) overflows the minimum window \(window.frame.size)"
            )
            #expect(
                abs(covered.width - content.bounds.width) < 1
                    && abs(covered.height - content.bounds.height) < 1,
                "the video surface \(covered.size) does not cover the window \(content.bounds.size)"
            )
            #expect(
                abs(covered.width / covered.height - ratio) < 0.01,
                "the minimum video surface \(covered.size) is not ratio \(ratio)"
            )
            let bar = try #require(
                transportFrames[.bar],
                "the minimum transport was not laid out"
            )
            let playPause = try #require(transportFrames[.playPause])
            let seek = try #require(transportFrames[.seek])
            let fullScreen = try #require(transportFrames[.fullScreen])
            #expect(abs(bar.height - TransportControls.height) < 0.5)
            #expect(abs(bar.width - covered.width) < 1)
            #expect(playPause.minX >= TransportControls.horizontalPadding - 0.5)
            #expect(fullScreen.maxX <= bar.width - TransportControls.horizontalPadding + 0.5)
            #expect(seek.width >= 76)
        }
    }
}
