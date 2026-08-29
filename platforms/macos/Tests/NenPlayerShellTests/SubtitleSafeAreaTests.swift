import AppKit
import Foundation
import NenCore
import OpenGL.GL
import OpenGL.GL3
import SwiftUI
import Testing

@testable import NenPlaybackMPV
@testable import NenPlayerShell

/// NEN-066: the line the engine draws is not hidden by the player's own
/// chrome.
///
/// Every other subtitle test in this repository asks **mpv** what it is
/// drawing. That question was answered correctly while the window stayed blank
/// — the render path composited the subtitle and the transport bar sat on top
/// of it — so it cannot be the question that closes this. These tests compare
/// two things nothing else compares: where the engine puts the pixels, and how
/// much of the surface the shell has covered.
///
/// **Security (K23 #4):** no dialogue is printed. A failure reports pixel
/// counts and a band in points.
@Suite("Subtitle safe area")
@MainActor
struct SubtitleSafeAreaTests {
    // MARK: - The surface

    /// The surface the measurement renders into, in points and in pixels.
    ///
    /// 640x360 pt is below the 720x450 pt the window allows at its smallest,
    /// on purpose: it is the harshest ratio of chrome to surface the product
    /// can produce, so a subtitle that clears the bar here clears it anywhere.
    static let surfacePoints = CGSize(width: 640, height: 360)
    static let backingScale: CGFloat = 2

    var surfacePixelWidth: Int32 { Int32(Self.surfacePoints.width * Self.backingScale) }
    var surfacePixelHeight: Int32 { Int32(Self.surfacePoints.height * Self.backingScale) }

    /// The repo root, derived from this file's own path — the fixture is
    /// shared with the Rust tests and must stay the same clip.
    static func fixturePath(_ name: String) -> String {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<5 { url = url.deletingLastPathComponent() }
        return url.appendingPathComponent("fixtures/media/\(name)").path
    }

    /// How much of the surface bottom the player's chrome covers, in points.
    ///
    /// Measured from the real view rather than written down: the bar's height
    /// is a layout result, and a constant here would go stale the first time
    /// the bar gains a row.
    static func chromeHeight() -> CGFloat {
        let model = PlayerModel(startsPolling: false, managesCursor: false)
        var presented = false
        let bar = TransportControls(
            model: model,
            showsSubtitlePanel: Binding(get: { presented }, set: { presented = $0 }),
            onInteractionOutsideSubtitlePanel: {}
        )
        let host = NSHostingView(rootView: bar.frame(width: surfacePoints.width))
        host.layoutSubtreeIfNeeded()
        return host.fittingSize.height + PlayerRootView.chromeBottomPadding
    }

    /// Lets the main actor run for a while without blocking it.
    ///
    /// `Thread.sleep` would hold the main actor, and every other `@MainActor`
    /// test in this target is driven by timers on it.
    func settle(_ seconds: TimeInterval) {
        RunLoop.current.run(until: Date().addingTimeInterval(seconds))
    }

    // MARK: - Offscreen capture

    /// An offscreen colour buffer the view's own render path draws into.
    final class OffscreenTarget {
        let width: Int32
        let height: Int32
        private var framebuffer: GLuint = 0
        private var texture: GLuint = 0

        init(view: MPVVideoView, width: Int32, height: Int32) throws {
            self.width = width
            self.height = height
            view.openGLContext?.makeCurrentContext()
            glGenTextures(1, &texture)
            glBindTexture(GLenum(GL_TEXTURE_2D), texture)
            glTexImage2D(
                GLenum(GL_TEXTURE_2D), 0, GL_RGBA8, width, height, 0,
                GLenum(GL_RGBA), GLenum(GL_UNSIGNED_BYTE), nil
            )
            glTexParameteri(GLenum(GL_TEXTURE_2D), GLenum(GL_TEXTURE_MIN_FILTER), GL_LINEAR)
            glGenFramebuffers(1, &framebuffer)
            glBindFramebuffer(GLenum(GL_FRAMEBUFFER), framebuffer)
            glFramebufferTexture2D(
                GLenum(GL_FRAMEBUFFER), GLenum(GL_COLOR_ATTACHMENT0),
                GLenum(GL_TEXTURE_2D), texture, 0
            )
            let status = glCheckFramebufferStatus(GLenum(GL_FRAMEBUFFER))
            glBindFramebuffer(GLenum(GL_FRAMEBUFFER), 0)
            guard status == GLenum(GL_FRAMEBUFFER_COMPLETE) else {
                throw Failure.incompleteFramebuffer(status: status)
            }
        }

        /// Renders one frame through the product's own path and reads it back.
        func capture(from view: MPVVideoView) -> [UInt8] {
            view.openGLContext?.makeCurrentContext()
            view.renderFrame(intoFramebuffer: Int32(framebuffer), width: width, height: height)
            glBindFramebuffer(GLenum(GL_FRAMEBUFFER), framebuffer)
            glReadBuffer(GLenum(GL_COLOR_ATTACHMENT0))
            glFinish()
            var pixels = [UInt8](repeating: 0, count: Int(width * height) * 4)
            pixels.withUnsafeMutableBytes { raw in
                glReadPixels(
                    0, 0, width, height,
                    GLenum(GL_RGBA), GLenum(GL_UNSIGNED_BYTE), raw.baseAddress
                )
            }
            glBindFramebuffer(GLenum(GL_FRAMEBUFFER), 0)
            return pixels
        }

        enum Failure: Error {
            case incompleteFramebuffer(status: GLenum)
        }
    }

    /// Where two frames differ, in rows counted **up from the bottom**.
    ///
    /// The frames are the same moment of the same clip with the subtitle on
    /// and off, so every differing pixel is the subtitle. Comparing two
    /// moments instead would compare the video with itself: the fixture is
    /// `testsrc` and it moves.
    static func band(_ on: [UInt8], _ off: [UInt8], width: Int32, height: Int32)
        -> (pixels: Int, lowest: Int, highest: Int) {
        var lowest = Int.max
        var highest = -1
        var count = 0
        let rowBytes = Int(width) * 4
        for row in 0..<Int(height) {
            for column in 0..<Int(width) {
                let offset = row * rowBytes + column * 4
                if on[offset] != off[offset]
                    || on[offset + 1] != off[offset + 1]
                    || on[offset + 2] != off[offset + 2] {
                    count += 1
                    lowest = min(lowest, row)
                    highest = max(highest, row)
                }
            }
        }
        return (count, lowest, highest)
    }

    /// The subtitle band the engine produces right now, in points from the
    /// bottom of the surface.
    private func measureBand(
        engine: MPVPlaybackEngine, view: MPVVideoView, target: OffscreenTarget
    ) throws -> (pixels: Int, bottom: CGFloat, top: CGFloat) {
        let on = try frame(engine: engine, view: view, target: target, drawing: true)
        let off = try frame(engine: engine, view: view, target: target, drawing: false)

        let measured = Self.band(on, off, width: target.width, height: target.height)
        #expect(measured.pixels > 0, "no pixel changed, so nothing was drawn to measure")
        return (
            measured.pixels,
            CGFloat(measured.lowest) / Self.backingScale,
            CGFloat(measured.highest) / Self.backingScale
        )
    }

    /// One frame, with the subtitle either on or off.
    ///
    /// Waits on the engine's own answer rather than on a fixed delay: this
    /// test shares the main actor with every timer-driven shell test in this
    /// target, so it holds it for as little as it can.
    private func frame(
        engine: MPVPlaybackEngine, view: MPVVideoView, target: OffscreenTarget, drawing: Bool
    ) throws -> [UInt8] {
        try engine.selectTrack(kind: .subtitle, track: drawing ? Self.subtitleTrack : nil)
        let deadline = Date().addingTimeInterval(3)
        while Date() < deadline {
            settle(0.02)
            if (((try? engine.renderedSubtitleText()) ?? nil) != nil) == drawing { break }
        }
        #expect(
            (((try? engine.renderedSubtitleText()) ?? nil) != nil) == drawing,
            "the engine never reached the state this frame is supposed to show"
        )
        // Twice: the first render is what the change produces, the second is
        // what a viewer would still be looking at afterwards.
        _ = target.capture(from: view)
        settle(0.05)
        return target.capture(from: view)
    }

    /// The clip's English subtitle track, by ff-index — the same id the
    /// contract fixture documents.
    static let subtitleTrack: UInt32 = 3

    /// A moment inside that track's second cue (10-13 s).
    static let momentMs: UInt64 = 11_000

    // MARK: - The defect

    @Test("the line the engine draws is not left under the transport bar")
    func theSubtitleClearsTheChrome() throws {
        let chrome = Self.chromeHeight()
        #expect(chrome > 0)

        let view = MPVVideoView.makePlaybackSurface()
        view.frame = NSRect(origin: .zero, size: Self.surfacePoints)
        let engine = try MPVPlaybackEngine(videoView: view)
        defer { try? engine.shutdown() }
        let target = try OffscreenTarget(
            view: view, width: surfacePixelWidth, height: surfacePixelHeight
        )

        try engine.setVolume(volume: 0)
        try engine.load(locator: Self.fixturePath("contract-clip.mkv"))
        let deadline = Date().addingTimeInterval(5)
        while Date() < deadline, engine.state() != .ready {
            settle(0.02)
        }
        #expect(engine.state() == .ready)
        try engine.seek(toMs: Self.momentMs)
        settle(0.3)

        // With nothing declared, the engine draws where it always has — and
        // that is underneath the bar. This half is the defect NEN-066 opened
        // for; it is asserted rather than assumed so the other half cannot
        // pass by accident on a build where the subtitle never appears at all.
        try engine.setSubtitleBottomInset(fraction: 0)
        let uncovered = try measureBand(engine: engine, view: view, target: target)
        #expect(
            uncovered.bottom < chrome,
            """
            the engine's own band starts at \(uncovered.bottom) pt, above the \
            \(chrome) pt of chrome — nothing is being covered and this test \
            can no longer tell the fix from the defect
            """
        )

        // Declared: the same share the shell sends when its chrome is visible.
        let inset = chrome / Self.surfacePoints.height
        try engine.setSubtitleBottomInset(fraction: Float(inset))
        let lifted = try measureBand(engine: engine, view: view, target: target)
        #expect(
            lifted.bottom >= chrome,
            """
            the subtitle's lowest pixel is \(lifted.bottom) pt from the bottom \
            and the chrome covers \(chrome) pt, so the line is behind it
            """
        )
    }

    // MARK: - What the shell declares

    @Test("nothing is declared while the chrome is hidden")
    func theShellDeclaresZeroWhenTheChromeIsHidden() {
        let session = FakeSession()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.setSubtitleBottomInset(0.372)
        model.setSubtitleBottomInset(0)
        #expect(session.subtitleBottomInsets == [0.372, 0])
    }

    @Test("the same inset is not declared twice")
    func theShellDoesNotRepeatItself() {
        // Geometry reports arrive on every layout pass; the value changes
        // twice per hide cycle. Forwarding each report would put a property
        // write on mpv for every frame of a resize.
        let session = FakeSession()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())

        model.setSubtitleBottomInset(0.372)
        model.setSubtitleBottomInset(0.372)
        model.setSubtitleBottomInset(0.372)
        #expect(session.subtitleBottomInsets == [0.372])
    }

    @Test("the real chrome measures itself and reaches the session")
    func theViewReportsWhatItCovers() throws {
        // The wiring between the two halves: the bar's own layout produces the
        // number, and it arrives at the session as a share of the surface.
        // Nothing else in this target hosts the real view, so without this the
        // measured geometry and the declared inset could drift apart.
        let session = FakeSession()
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        model.openMedia(at: URL(fileURLWithPath: Self.fixturePath("contract-clip.mkv")))

        // The window's own minimum, because the view enforces it: asking for
        // less would measure a surface the player never shows.
        let host = NSHostingView(rootView: PlayerRootView(model: model))
        host.frame = NSRect(x: 0, y: 0, width: 720, height: 450)
        host.layoutSubtreeIfNeeded()
        settle(0.3)

        let declared = try #require(
            session.subtitleBottomInsets.last,
            "the view never told the session how much of the surface it covers"
        )
        let expected = Float(Self.chromeHeight() / host.bounds.height)
        #expect(
            abs(declared - expected) < 0.01,
            "declared \(declared), the bar measures \(expected)"
        )
    }

    @Test("a new session is told what the last one was told")
    func aReopenedWindowKeepsTheInset() {
        // NEN-046's window lifecycle: closing the window drops the session and
        // reopening it builds another. The chrome did not move in between, so
        // a fresh engine that started at the bottom would put the subtitle
        // back under the bar.
        let first = FakeSession()
        let second = FakeSession()
        var built = 0
        let model = PlayerModel(
            startsPolling: false,
            managesCursor: false,
            sessionFactory: { _ in
                built += 1
                return built == 1 ? first : second
            }
        )
        let surface = MPVVideoView.makePlaybackSurface()
        model.attach(to: surface)
        model.setSubtitleBottomInset(0.372)
        #expect(first.subtitleBottomInsets == [0.372])

        model.shutdown()
        model.attach(to: surface)
        #expect(second.subtitleBottomInsets == [0.372])
    }
}
