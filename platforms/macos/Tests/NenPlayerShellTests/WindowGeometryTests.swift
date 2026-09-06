import AppKit
import Foundation
import NenCore
import Testing

@testable import NenPlayerShell

/// NEN-068 / ADR-0038 Karar 4: the window's shape, as arithmetic.
///
/// The product's promise is one sentence — *outside full screen the window
/// never shows a black bar above or below the picture* — and it reduces to two
/// pure functions. They are tested here rather than through a live `NSWindow`
/// because a window test can only ever observe what AppKit did with what it was
/// told; this observes what it is told.
///
/// The ratios are the ones the product actually meets: 16:9 and 4:3 for what
/// people watch, 2.39:1 for film, 9:16 for what a phone records, and anamorphic
/// for the case where the stored frame is a lie.
@Suite("Window geometry")
struct WindowGeometryTests {
    /// A 16-inch MacBook Pro's usable area, minus the menu bar.
    static let laptopScreen = CGRect(x: 0, y: 0, width: 1_512, height: 916)

    // MARK: - The derived minimum

    @Test("the minimum for a ratio is itself that ratio")
    func theMinimumHonoursTheRatio() {
        // The property that matters more than any single number: a minimum
        // that did not satisfy the aspect lock is one AppKit cannot honour,
        // and the bars come back at exactly the size a user drags to.
        for ratio in [16.0 / 9, 4.0 / 3, 2.39, 9.0 / 16, 1.0] as [CGFloat] {
            let size = WindowGeometry.minimumContentSize(for: ratio)
            let produced = size.width / size.height
            #expect(
                abs(produced - ratio) < 0.01,
                "minimum for \(ratio) is \(size.width)x\(size.height), a ratio of \(produced)"
            )
        }
    }

    @Test("the minimum is never smaller than the chrome needs")
    func theMinimumClearsTheChrome() {
        for ratio in [16.0 / 9, 4.0 / 3, 2.39, 9.0 / 16] as [CGFloat] {
            let size = WindowGeometry.minimumContentSize(for: ratio)
            #expect(size.width >= WindowGeometry.chromeBase.width)
            #expect(size.height >= WindowGeometry.chromeBase.height)
        }
    }

    @Test("each ratio derives the size the task measured")
    func theDerivedMinimums() {
        #expect(WindowGeometry.minimumContentSize(for: 16.0 / 9) == CGSize(width: 693, height: 390))
        #expect(WindowGeometry.minimumContentSize(for: 2.39) == CGSize(width: 932, height: 390))
        #expect(WindowGeometry.minimumContentSize(for: 4.0 / 3) == CGSize(width: 693, height: 520))
        #expect(WindowGeometry.minimumContentSize(for: 9.0 / 16) == CGSize(width: 693, height: 1_232))
    }

    @Test("a ratio that is not a ratio falls back to the chrome base")
    func aDegenerateRatioIsRefused() {
        // Reachable: a geometry that arrived mid-reconfiguration, a division
        // that produced a `nan`. The window must still be usable.
        for ratio in [0, -1.5, CGFloat.nan, CGFloat.infinity] as [CGFloat] {
            #expect(WindowGeometry.minimumContentSize(for: ratio) == WindowGeometry.chromeBase)
        }
    }

    // MARK: - The opening size

    @Test("a medium smaller than the chrome opens at the derived minimum")
    func aTinyMediumOpensAtTheMinimum() {
        // Every media fixture in this repository is 160x90 — far below what the
        // transport row needs. The picture is scaled up by the renderer; the
        // window is not shrunk to it.
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 160, height: 90),
            visibleFrame: Self.laptopScreen
        )
        #expect(size == CGSize(width: 693, height: 390))
    }

    @Test("a medium that fits opens at its own size, one point per pixel")
    func aFittingMediumOpensAtItsOwnSize() {
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 1_280, height: 720),
            visibleFrame: Self.laptopScreen
        )
        #expect(size == CGSize(width: 1_280, height: 720))
    }

    @Test("an anamorphic medium opens at its display size, not its stored one")
    func anAnamorphicMediumOpensAtItsDisplaySize() {
        // `fixtures/media/anamorphic-clip.mkv` is stored 720x576 and displayed
        // 1024x576 — libmpv reports the corrected size and the port carries it
        // (ADR-0038 Karar 3), so what reaches here is already 1024x576. What
        // this test holds is that the window does not then undo it: 16:9 in,
        // 16:9 out, never the 5:4 of the stored frame.
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 1_024, height: 576),
            visibleFrame: Self.laptopScreen
        )
        #expect(size == CGSize(width: 1_024, height: 576))
        #expect(abs(size.width / size.height - 16.0 / 9) < 0.01)
        #expect(size != CGSize(width: 720, height: 576))
    }

    @Test("a medium larger than the screen is scaled down whole")
    func anOversizedMediumIsScaledPreservingItsRatio() {
        let media = CGSize(width: 3_840, height: 2_160)
        let size = WindowGeometry.contentSize(for: media, visibleFrame: Self.laptopScreen)

        #expect(size.width <= Self.laptopScreen.width)
        #expect(size.height <= Self.laptopScreen.height)
        // The point of scaling both axes by the same factor: fitting each to
        // its own bound would letterbox the window, which is the very defect
        // being removed — applied by the shell instead of by the renderer.
        #expect(
            abs(size.width / size.height - media.width / media.height) < 0.01,
            "scaled to \(size.width)x\(size.height), which is no longer 16:9"
        )
    }

    @Test("a tall medium is scaled down but still keeps its ratio")
    func aVerticalMediumKeepsItsRatio() {
        // 3:4 on a laptop: scaling to the screen would produce 687x916, just
        // below the 693x924 floor this chrome needs. The floor wins, even
        // though that leaves the bottom of the window below this screen.
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 1_080, height: 1_440),
            visibleFrame: Self.laptopScreen
        )
        #expect(size == WindowGeometry.minimumContentSize(for: 0.75))
        #expect(size.height > Self.laptopScreen.height)
        #expect(abs(size.width / size.height - 0.75) < 0.01)
    }

    @Test("the chrome's floor outranks the screen when the two disagree")
    func theMinimumWinsOverTheScreen() {
        // A phone recording, 9:16, on a laptop. Scaled to fit the height it
        // would be 515x916, under the 693x1232 the chrome needs — so the floor
        // raises it back past the screen's height, on purpose.
        //
        // The alternative is worse in a way the user cannot work around: a
        // window the transport row does not fit in is broken, while one whose
        // bottom edge runs off the screen is merely awkward and still fully
        // usable — `recentredFrame` keeps its title bar reachable.
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 1_080, height: 1_920),
            visibleFrame: Self.laptopScreen
        )
        #expect(size == WindowGeometry.minimumContentSize(for: 9.0 / 16))
        #expect(size.height > Self.laptopScreen.height)
        #expect(abs(size.width / size.height - 0.5625) < 0.01)
    }

    @Test("scaling never lands below the minimum")
    func scalingIsFlooredByTheMinimum() {
        // A cinema-ratio medium on a screen too short for it: scaled to fit the
        // height it would end up under the 932x390 the chrome needs. The floor
        // is applied last, and it is itself the right ratio, so raising to it
        // cannot put a bar back.
        let cramped = CGRect(x: 0, y: 0, width: 800, height: 300)
        let size = WindowGeometry.contentSize(
            for: CGSize(width: 2_390, height: 1_000),
            visibleFrame: cramped
        )
        let minimum = WindowGeometry.minimumContentSize(for: 2.39)
        #expect(size == minimum)
        #expect(abs(size.width / size.height - 2.39) < 0.01)
    }

    @Test("a medium with no size at all falls back to the chrome base")
    func aDegenerateMediumIsRefused() {
        for media in [CGSize(width: 0, height: 576), CGSize(width: 1_024, height: 0), .zero] {
            #expect(
                WindowGeometry.contentSize(for: media, visibleFrame: Self.laptopScreen)
                    == WindowGeometry.chromeBase
            )
        }
    }

    // MARK: - Placement

    @Test("resizing keeps the window's centre")
    func resizingKeepsTheCentre() {
        // A player is a window many media are opened in. Keeping the top-left
        // instead would walk it down and to the right, once per file.
        let current = CGRect(x: 400, y: 300, width: 700, height: 400)
        let placed = WindowGeometry.recentredFrame(
            size: CGSize(width: 932, height: 420),
            around: current,
            visibleFrame: Self.laptopScreen
        )
        #expect(abs(placed.midX - current.midX) < 0.5)
        #expect(abs(placed.midY - current.midY) < 0.5)
        #expect(placed.size == CGSize(width: 932, height: 420))
    }

    @Test("a window recentred off the screen is nudged back on")
    func placementClampsToTheScreen() {
        // The clamp only ever translates: the size, and therefore the aspect
        // ratio the whole task is about, must survive it untouched.
        let current = CGRect(x: 1_400, y: 850, width: 200, height: 120)
        let size = CGSize(width: 1_024, height: 576)
        let placed = WindowGeometry.recentredFrame(
            size: size,
            around: current,
            visibleFrame: Self.laptopScreen
        )
        #expect(placed.size == size)
        #expect(placed.maxX <= Self.laptopScreen.maxX + 0.5)
        #expect(placed.maxY <= Self.laptopScreen.maxY + 0.5)
        #expect(placed.minX >= Self.laptopScreen.minX - 0.5)
        #expect(placed.minY >= Self.laptopScreen.minY - 0.5)
    }

    @Test("a window taller than the screen keeps its title bar reachable")
    func anOversizedWindowKeepsItsControlsReachable() {
        // The 9:16 minimum is 693x1232, taller than a laptop's usable height,
        // so this frame is reached through the ordinary path — see
        // `theMinimumWinsOverTheScreen`. On the y axis the two bounds
        // contradict each other, and which one survives decides whether the
        // traffic lights are on screen: the top must win.
        let placed = WindowGeometry.recentredFrame(
            size: CGSize(width: 693, height: 1_232),
            around: CGRect(x: 100, y: 100, width: 693, height: 400),
            visibleFrame: Self.laptopScreen
        )
        #expect(placed.maxY == Self.laptopScreen.maxY, "the title bar ran off the top")
        // The x axis is not in conflict — the window fits across — so nothing
        // moves it and the centre is kept, as everywhere else.
        #expect(abs(placed.midX - 446.5) < 0.5)
    }

    @Test("a window wider than the screen keeps its left edge on screen")
    func anOverwideWindowKeepsItsLeftEdge() {
        // The other axis of the same conflict. Left wins, because that is the
        // corner the close, minimise and zoom buttons live in.
        let placed = WindowGeometry.recentredFrame(
            size: CGSize(width: 2_000, height: 400),
            around: CGRect(x: 300, y: 300, width: 600, height: 400),
            visibleFrame: Self.laptopScreen
        )
        #expect(placed.minX == Self.laptopScreen.minX)
    }
}
