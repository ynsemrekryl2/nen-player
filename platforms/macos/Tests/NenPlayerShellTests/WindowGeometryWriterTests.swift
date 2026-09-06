import AppKit
import Foundation
import NenCore
import Testing

@testable import NenPlayerShell

/// NEN-068: what `WindowGeometry`'s arithmetic actually does to a window.
///
/// `WindowGeometryTests` proves the numbers. This proves they arrive — on a
/// real `NSWindow`, through the same coordinator the shell uses, because the
/// two halves can be individually right and still not meet: a lock that is
/// computed and never applied leaves exactly the black bars this task removes.
///
/// The window is never shown. Everything asserted here is state AppKit holds
/// whether or not anything is on screen, which is what makes this repeatable
/// rather than a screenshot.
@Suite("Window geometry writer")
@MainActor
struct WindowGeometryWriterTests {
    /// A window shaped like the app's, and never ordered front.
    private func window() -> NSWindow {
        NSWindow(
            contentRect: NSRect(x: 200, y: 200, width: 1_080, height: 680),
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: true
        )
    }

    private func visibleFrame(_ window: NSWindow) -> CGRect {
        (window.screen ?? NSScreen.main)?.visibleFrame ?? .zero
    }

    @Test("a picture locks the window to its ratio and to the chrome's floor")
    func aPictureLocksTheWindow() {
        let window = window()
        let contentMinimum = CGSize(width: 333, height: 222)
        window.contentMinSize = contentMinimum
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1,
            to: window
        )

        #expect(window.contentAspectRatio == NSSize(width: 1_920, height: 1_080))
        #expect(window.contentMinSize == contentMinimum, "the writer competed with SwiftUI's minimum")
    }

    @Test("a 4:3 picture locks to 4:3, not to the 16:9 the last one had")
    func adifferentRatioReplacesTheLock() {
        // Two media in a row, which is the ordinary way a player is used. The
        // second must not inherit the first's shape.
        let window = window()
        let contentMinimum = CGSize(width: 333, height: 222)
        window.contentMinSize = contentMinimum
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1,
            to: window
        )
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 640, height: 480),
            mediaRevision: 2,
            to: window
        )

        #expect(window.contentAspectRatio == NSSize(width: 640, height: 480))
        #expect(window.contentMinSize == contentMinimum, "the writer competed with SwiftUI's minimum")
    }

    @Test("an anamorphic picture locks to what is displayed, not what is stored")
    func anAnamorphicPictureLocksToItsDisplayShape() {
        // The size the adapter reports for `fixtures/media/anamorphic-clip.mkv`
        // — 1024x576 for a 720x576 frame. The window must end up 16:9.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_024, height: 576),
            mediaRevision: 1,
            to: window
        )

        let locked = window.contentAspectRatio
        #expect(abs(locked.width / locked.height - 16.0 / 9) < 0.01)
        #expect(locked != NSSize(width: 720, height: 576))
    }

    @Test("no picture means no lock and a free resize")
    func noPictureLeavesTheWindowFree() {
        // Audio-only, a failed medium, the empty state. ADR-0038 accepts this
        // as the cost of the design: nothing to hold a ratio to.
        let window = window()
        let contentMinimum = CGSize(width: 333, height: 222)
        window.contentMinSize = contentMinimum
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1,
            to: window
        )
        coordinator.apply(geometry: nil, mediaRevision: 2, to: window)

        #expect(window.contentAspectRatio == .zero, "the lock outlived the picture")
        #expect(window.contentMinSize == contentMinimum, "the writer competed with SwiftUI's minimum")
        #expect(window.resizeIncrements == NSSize(width: 1, height: 1))
    }

    @Test("a new medium sizes the window to the picture")
    func aNewMediumSizesTheWindow() {
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        let media = CGSize(width: 1_280, height: 720)

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 1,
            to: window
        )

        let expected = WindowGeometry.contentSize(
            for: media,
            visibleFrame: visibleFrame(window)
        )
        let content = window.contentRect(forFrameRect: window.frame)
        #expect(abs(content.width - expected.width) < 1)
        #expect(abs(content.height - expected.height) < 1)
        // The whole point, stated as the product states it: the content area is
        // the picture's own shape, so there is nothing left over to letterbox.
        #expect(abs(content.width / content.height - 1_280.0 / 720) < 0.01)
    }

    @Test("a tiny medium opens at the chrome's derived minimum, not at 160x90")
    func aTinyMediumOpensAtTheMinimum() {
        // Every media fixture in this repository is 160x90.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 160, height: 90),
            mediaRevision: 1,
            to: window
        )

        let content = window.contentRect(forFrameRect: window.frame)
        #expect(abs(content.width - 693) < 1)
        #expect(abs(content.height - 390) < 1)
    }

    @Test("a reconfiguration of the same medium does not move the window")
    func aReconfigurationLeavesThePlacementAlone() {
        // The rule that cannot be expressed as arithmetic. A stream that
        // reconfigures mid-playback, or any repeat update, must refresh the
        // lock without yanking a window the user has already sized and placed.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 7,
            to: window
        )

        // The user then resizes and moves it.
        let placed = NSRect(x: 40, y: 60, width: 900, height: 520)
        window.setFrame(placed, display: false)

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 7,
            to: window
        )
        #expect(window.frame == placed, "a repeat update moved the user's window")

        // Same medium, new size: the lock follows, the frame still does not.
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 640, height: 480),
            mediaRevision: 7,
            to: window
        )
        #expect(window.frame == placed, "a reconfiguration moved the user's window")
        #expect(window.contentAspectRatio == NSSize(width: 640, height: 480))
    }

    @Test("the next medium does size the window, even at the same ratio")
    func thenextMediumSizesTheWindowAgain() {
        // The counterpart to the test above: "do not move it" must not become
        // "never move it again". Two 16:9 films in a row are two openings, and
        // the revision is what tells them apart — the size alone cannot.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 1,
            to: window
        )
        window.setFrame(NSRect(x: 40, y: 60, width: 900, height: 520), display: false)

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 2,
            to: window
        )

        let content = window.contentRect(forFrameRect: window.frame)
        #expect(abs(content.width / content.height - 1_280.0 / 720) < 0.01)
        #expect(content.width > 900, "the second medium did not reopen at its own size")
    }

    @Test("a medium that announces its size after opening still gets sized")
    func aSizeThatArrivesAfterTheOpeningStillSizesTheWindow() {
        // **The real sequence, and the one the first version of this writer got
        // wrong.** `openMedia` clears the geometry and bumps the revision in
        // the same turn, because the shell cannot know the new picture's shape
        // until the engine announces it. So every medium arrives as
        // `(nil, N)` and only then as `(size, N)`.
        //
        // The first version marked revision N sized on the `nil` pass, so the
        // announcement found the work already done and the window never moved.
        // Measured on the real app: a 16:9 clip opened into the previous
        // medium's 1031x432 frame and was pillarboxed inside it. Unit tests
        // that jumped straight to `(size, N)` all passed.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        let opening = NSRect(x: 40, y: 60, width: 1_031, height: 432)
        window.setFrame(opening, display: false)

        coordinator.apply(geometry: nil, mediaRevision: 1, to: window)
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_280, height: 720),
            mediaRevision: 1,
            to: window
        )

        #expect(window.frame != opening, "the window never opened at the picture's size")
        let content = window.contentRect(forFrameRect: window.frame)
        #expect(abs(content.width / content.height - 1_280.0 / 720) < 0.01)
    }

    @Test("full screen releases the lock and leaving it puts the lock back")
    func fullScreenReleasesAndRestoresTheLock() {
        // Entering full screen with the lock on makes AppKit fight its own
        // animation; leaving it does not schedule a SwiftUI update, so the lock
        // has to be restored from the notification or it stays off — and the
        // window stays draggable into a letterbox until something else redraws.
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1,
            to: window
        )
        let locked = window.contentAspectRatio
        #expect(locked != .zero)

        NotificationCenter.default.post(
            name: NSWindow.willEnterFullScreenNotification,
            object: window
        )
        #expect(window.contentAspectRatio == .zero, "the lock survived into full screen")
        // A zero aspect getter alone missed the real macOS 27 exit failure:
        // setting the ratio to zero leaves zero resize increments behind.
        #expect(window.resizeIncrements == NSSize(width: 1, height: 1))

        NotificationCenter.default.post(
            name: NSWindow.didExitFullScreenNotification,
            object: window
        )
        #expect(window.contentAspectRatio == locked, "the lock never came back")

        coordinator.stopObserving()
    }

    @Test("updates cannot restore constraints while full screen is transitioning")
    func updatesDuringFullScreenStaySuspended() {
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        defer { coordinator.stopObserving() }
        let geometry = FfiVideoGeometry(width: 1_920, height: 1_080)
        coordinator.apply(geometry: geometry, mediaRevision: 1, to: window)
        let minimum = window.contentMinSize

        NotificationCenter.default.post(name: NSWindow.willEnterFullScreenNotification, object: window)
        // The style flag is false before entry and again before exit completes.
        // A SwiftUI update in either interval must not reinstall the lock.
        #expect(!window.styleMask.contains(.fullScreen))
        coordinator.apply(geometry: geometry, mediaRevision: 1, to: window)
        #expect(window.contentAspectRatio == .zero)
        #expect(window.resizeIncrements == NSSize(width: 1, height: 1))
        #expect(window.contentMinSize == minimum)

        NotificationCenter.default.post(name: NSWindow.willExitFullScreenNotification, object: window)
        coordinator.apply(geometry: geometry, mediaRevision: 1, to: window)
        #expect(window.contentAspectRatio == .zero)
        #expect(window.resizeIncrements == NSSize(width: 1, height: 1))

        NotificationCenter.default.post(name: NSWindow.didExitFullScreenNotification, object: window)
        #expect(window.contentAspectRatio == NSSize(width: 1_920, height: 1_080))
    }

    @Test("a medium opened in full screen is sized only after exit")
    func aNewMediumWaitsForFullScreenExit() {
        let window = window()
        let coordinator = WindowGeometryWriter.Coordinator()
        defer { coordinator.stopObserving() }
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1, to: window
        )
        NotificationCenter.default.post(name: NSWindow.willEnterFullScreenNotification, object: window)
        let frame = window.frame
        let minimum = window.contentMinSize
        coordinator.apply(geometry: nil, mediaRevision: 2, to: window)
        let next = FfiVideoGeometry(width: 640, height: 480)
        coordinator.apply(geometry: next, mediaRevision: 2, to: window)
        #expect(window.frame == frame)
        #expect(window.contentMinSize == minimum)
        #expect(window.contentAspectRatio == .zero)

        NotificationCenter.default.post(name: NSWindow.didExitFullScreenNotification, object: window)
        #expect(window.contentAspectRatio == NSSize(width: 640, height: 480))
        #expect(window.contentMinSize == minimum, "the writer competed with SwiftUI's minimum")
        let expected = WindowGeometry.contentSize(
            for: CGSize(width: 640, height: 480), visibleFrame: visibleFrame(window)
        )
        let content = window.contentRect(forFrameRect: window.frame)
        #expect(abs(content.width - expected.width) < 1)
        #expect(abs(content.height - expected.height) < 1)
    }

    @Test("losing the video in full screen does not restore the outgoing ratio")
    func noPictureOnExitLeavesTheWindowFree() {
        let window = window()
        let contentMinimum = CGSize(width: 333, height: 222)
        window.contentMinSize = contentMinimum
        let coordinator = WindowGeometryWriter.Coordinator()
        defer { coordinator.stopObserving() }
        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_920, height: 1_080),
            mediaRevision: 1, to: window
        )
        NotificationCenter.default.post(name: NSWindow.willEnterFullScreenNotification, object: window)
        coordinator.apply(geometry: nil, mediaRevision: 2, to: window)
        NotificationCenter.default.post(name: NSWindow.didExitFullScreenNotification, object: window)
        #expect(window.contentAspectRatio == .zero)
        #expect(window.resizeIncrements == NSSize(width: 1, height: 1))
        #expect(window.contentMinSize == contentMinimum, "the writer competed with SwiftUI's minimum")
    }

    @Test("a degenerate size is treated as no picture rather than divided by")
    func aDegenerateSizeIsRefused() {
        // The core refuses a zero dimension before building one, so this is the
        // boundary keeping its own promise: dividing by a zero height would put
        // a `nan` into a window frame.
        let window = window()
        let contentMinimum = CGSize(width: 333, height: 222)
        window.contentMinSize = contentMinimum
        let coordinator = WindowGeometryWriter.Coordinator()

        coordinator.apply(
            geometry: FfiVideoGeometry(width: 1_024, height: 0),
            mediaRevision: 1,
            to: window
        )

        #expect(window.contentAspectRatio == .zero)
        #expect(window.contentMinSize == contentMinimum, "the writer competed with SwiftUI's minimum")
        #expect(window.frame.width.isFinite)
        #expect(window.frame.height.isFinite)
    }
}
