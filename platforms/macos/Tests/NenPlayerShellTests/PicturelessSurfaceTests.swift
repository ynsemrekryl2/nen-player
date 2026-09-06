import AppKit
import Foundation
import NenCore
import OpenGL.GL
import OpenGL.GL3
import Testing

@testable import NenPlaybackMPV
@testable import NenPlayerShell

/// NEN-069: a medium with no picture leaves nothing of the medium before it.
///
/// The reported defect was one frame too long on screen — opening
/// `audio-only-clip.mka` after a video left the video's last frame standing.
/// Measurement split it into three parts, and only two of them were broken
/// (`evidence/M3/NEN-069-measurement.md`):
///
/// - **Who asks for the redraw** — missing, and this is the reported defect.
///   mpv's update callback fires when mpv has produced a new frame, and a
///   medium with no picture never produces one, so nothing ever asked this
///   view to draw again.
/// - **What the redraw draws** — already right. Driven by hand before anything
///   was changed, the render path paints an empty picture black on its own. So
///   the first half of the pixel test below was **green before the fix**: it
///   is not what proves the fix, it pins the assumption the fix rests on, so a
///   libmpv that stopped clearing could not restore the defect in silence.
///   Its negative control is therefore the trigger test above, not itself.
/// - **Where the clear lands** — broken, and found by this suite rather than
///   by the report. The context-less branch of `renderFrame` cleared whatever
///   framebuffer happened to be bound instead of the one it was handed. On
///   screen that is 0 either way, so the product was never wrong; the
///   shutdown assertion below was red until it was fixed.
///
/// **Security (K23):** no path and no dialogue is printed. A failure reports
/// pixel counts and a fixture name.
@Suite("Pictureless surface")
@MainActor
struct PicturelessSurfaceTests {
    // MARK: - Who asks for the redraw

    /// A view outside a window cannot answer this question: AppKit discards
    /// `needsDisplay` when there is nothing to display into — measured, it
    /// reads back `false` however it is set. So the surface is put in a
    /// borderless window, which is also what the product does with it.
    private func windowedSurface() -> (NSWindow, MPVVideoView) {
        let window = NSWindow(
            contentRect: NSRect(x: 0, y: 0, width: 640, height: 360),
            styleMask: [.borderless],
            backing: .buffered,
            defer: false
        )
        let view = MPVVideoView.makePlaybackSurface()
        view.frame = NSRect(x: 0, y: 0, width: 640, height: 360)
        window.contentView?.addSubview(view)
        // Whatever AppKit wanted drawn on the way in is drawn now, so the flag
        // this test reads is the one the events under test set.
        view.displayIfNeeded()
        return (window, view)
    }

    @Test("an opened medium is drawn once without mpv asking")
    func aMediumThatOpensAsksForItsOwnRedraw() {
        let (window, view) = windowedSurface()
        withExtendedLifetime(window) {
            let session = FakeSession()
            session.currentVideoGeometry = nil
            let model = PlayerModel(
                startsPolling: false,
                managesCursor: false,
                sessionFactory: { _ in session }
            )
            model.attach(to: view)
            view.displayIfNeeded()
            #expect(view.needsDisplay == false, "the surface was already dirty, so this proves nothing")

            model.consume([.stateChanged(state: .ready)])

            #expect(
                view.needsDisplay,
                """
                nothing asked the surface to draw when the medium opened, so a \
                medium with no picture keeps showing the previous one
                """
            )
        }
    }

    @Test("nothing else in the stream asks the surface to draw")
    func onlyOpeningAMediumAsks() {
        let (window, view) = windowedSurface()
        withExtendedLifetime(window) {
            let session = FakeSession()
            let model = PlayerModel(
                startsPolling: false,
                managesCursor: false,
                sessionFactory: { _ in session }
            )
            model.attach(to: view)
            view.displayIfNeeded()

            // Position reports arrive twenty times a second and state changes
            // arrive on every transport press. A redraw on each of them would
            // put a render of mpv's current frame on the main actor at that
            // rate, and the test above would pass for the wrong reason.
            model.consume([
                .positionChanged(positionMs: 4_040),
                .stateChanged(state: .playing),
                .stateChanged(state: .paused),
                .positionChanged(positionMs: 4_080)
            ])

            #expect(view.needsDisplay == false)
        }
    }

    // MARK: - What the redraw puts there

    /// The offscreen capture is `NEN-066`'s — the same framebuffer the
    /// subtitle band was measured in, driving the same product render path.
    private func offscreen(_ view: MPVVideoView) throws -> SubtitleSafeAreaTests.OffscreenTarget {
        try SubtitleSafeAreaTests.OffscreenTarget(view: view, width: 1_280, height: 720)
    }

    /// Pixels that are not black, with a tolerance for the codec's own floor.
    private static func lit(_ pixels: [UInt8]) -> Int {
        var count = 0
        for index in stride(from: 0, to: pixels.count, by: 4)
        where pixels[index] > 8 || pixels[index + 1] > 8 || pixels[index + 2] > 8 {
            count += 1
        }
        return count
    }

    /// Gives the main actor up rather than pumping its run loop.
    ///
    /// `RunLoop.run(until:)` would keep this test on the actor and re-enter
    /// it, and the shell tests sharing this target are driven by timers that
    /// resume there. Measured while writing this suite: with the run loop
    /// pumped instead of yielded, three of those tests missed **five-second**
    /// deadlines that had already been widened once for NEN-066's libmpv
    /// suite. Awaiting suspends, so their continuations get the actor back.
    private func yieldFor(_ seconds: Double) async {
        try? await Task.sleep(nanoseconds: UInt64(seconds * 1_000_000_000))
    }

    private func waitUntilReady(_ engine: MPVPlaybackEngine) async {
        let deadline = Date().addingTimeInterval(5)
        while Date() < deadline, engine.state() != .ready {
            await yieldFor(0.02)
        }
        #expect(engine.state() == .ready)
    }

    /// Both surfaces the medium can leave behind, in one engine on purpose.
    ///
    /// Each real libmpv engine this target adds is main-actor time the
    /// timer-driven shell tests beside it do not get, and there is a standing
    /// task for that contention (`NEN-049`). Measured while writing this
    /// suite: as two tests with an engine each, the parallel package turned
    /// `ContractTests.successiveMediaReportTheirOwnDisplaySize` from an
    /// occasional red into a red on every run. One engine and two loads say
    /// the same thing for half the cost.
    @Test("a medium with no picture, and then no medium, keep none of the last one")
    func nothingOfThePreviousMediumSurvives() async throws {
        let view = MPVVideoView.makePlaybackSurface()
        view.frame = NSRect(x: 0, y: 0, width: 640, height: 360)
        let engine = try MPVPlaybackEngine(videoView: view)
        let target = try offscreen(view)

        try engine.setVolume(volume: 0)
        try engine.load(locator: SubtitleSafeAreaTests.fixturePath("menu-clip.mkv"))
        await waitUntilReady(engine)
        try engine.seek(toMs: 5_000)
        await yieldFor(0.4)
        _ = target.capture(from: view)
        await yieldFor(0.1)
        // Asserted rather than assumed: if the clip never reached the surface,
        // the emptiness below would mean nothing.
        let withPicture = Self.lit(target.capture(from: view))
        #expect(
            withPicture > 0,
            "menu-clip.mkv put nothing on the surface, so there is nothing to clear"
        )

        try engine.load(locator: SubtitleSafeAreaTests.fixturePath("audio-only-clip.mka"))
        await waitUntilReady(engine)
        await yieldFor(0.4)
        let withoutPicture = Self.lit(target.capture(from: view))
        #expect(
            withoutPicture == 0,
            """
            \(withoutPicture) pixels of the previous medium survived the \
            redraw of a medium that has no picture
            """
        )

        // The window's red button and ⌘W come here (NEN-046). No new code
        // carries this half: `shutdown` detaches the render context, and a
        // view with no context clears — this is what keeps that true.
        try engine.shutdown()
        await yieldFor(0.2)
        #expect(Self.lit(target.capture(from: view)) == 0)
    }
}
