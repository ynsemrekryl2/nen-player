import CoreGraphics
import Foundation

/// How the player's window is sized and shaped for the picture it is showing
/// (ADR-0038 Karar 4).
///
/// The port carries one number — the video's display size — and stops there.
/// What is done with it is the platform's decision, and this is where it is
/// made: the opening size, the aspect lock's minimum, and the clamp onto the
/// screen the window will actually appear on.
///
/// **Pure on purpose.** Nothing here touches an `NSWindow`, a screen or a
/// model. AppKit geometry is famously hard to test — a window has to exist, be
/// on a screen, and be given a run loop — so the arithmetic lives here where a
/// test can state the whole input, and `WindowGeometryWriter` is left with
/// nothing but the applying.
public enum WindowGeometry {
    /// The smallest content area NEN-073's chrome fits in, in points.
    ///
    /// Measured rather than chosen:
    ///
    /// - **693 pt wide** — the transport row's intrinsic width is ≈590 pt,
    ///   with room for long time labels and the subtitle selection label
    ///   (2×22 pt padding, 11×6 pt spacing, the 34+30+30 pt buttons, ≈30 pt
    ///   elapsed time, the 76 pt seek minimum, 43 pt trailing time, 22 pt
    ///   speaker, the 72 pt volume minimum, 4.5 pt divider, 69 pt CC, 40 pt
    ///   rate, 30 pt full screen). A medium over an hour long widens the time
    ///   labels to `0:00:00` and takes it to ≈606 pt.
    /// - **390 pt tall** — the 326 pt subtitle panel above the 57 pt bar.
    ///
    /// The 720×450 this replaces was this number rounded up, and rounding is
    /// exactly what made it wrong: as a *fixed* pair it contradicts every
    /// aspect ratio wider than 1.60, so the window could not both honour it and
    /// stay letterbox-free.
    public static let chromeBase = CGSize(width: 693, height: 390)

    /// The smallest content size a window locked to `ratio` may have.
    ///
    /// Derived from [`chromeBase`](chromeBase) rather than stated per ratio,
    /// because a minimum that did not itself satisfy the aspect lock is a
    /// minimum AppKit cannot honour — and the black bars come back at exactly
    /// the size the user is most likely to reach for.
    ///
    /// Width leads: the chrome's height budget is the softer of the two, so the
    /// derived size is the base height scaled out to the ratio, floored at the
    /// base width. 16:9 → 693×390 · 2.39:1 → 932×390 · 4:3 → 693×520 ·
    /// 9:16 → 693×1232.
    ///
    /// A non-finite or non-positive ratio has no shape to honour, so the base
    /// is returned unchanged.
    public static func minimumContentSize(for ratio: CGFloat) -> CGSize {
        guard ratio.isFinite, ratio > 0 else { return chromeBase }
        let width = max(chromeBase.width, (chromeBase.height * ratio).rounded())
        return CGSize(width: width, height: (width / ratio).rounded())
    }

    /// The content size a window should open at for a picture of `media`.
    ///
    /// Nominally one point per pixel — the product's promise is that a medium
    /// opens at its own size. Two things bound that:
    ///
    /// 1. **The screen.** A 4K picture does not fit on the desk it is being
    ///    watched from, so it is scaled down *preserving the ratio*; scaling
    ///    each axis to fit would be the letterboxing this whole task exists to
    ///    remove, applied by the window instead of by the renderer.
    /// 2. **The chrome.** Below the derived minimum the transport row cannot
    ///    lay out, so the size is raised to it — after the scaling, because
    ///    [`minimumContentSize`](minimumContentSize(for:)) already honours the
    ///    ratio and raising to it cannot reintroduce a bar.
    ///
    /// Raising can therefore produce a window taller than the screen (a 9:16
    /// picture on a short display). That is deliberate: a window the chrome
    /// does not fit in is broken in a way the user cannot fix, while one whose
    /// edge runs past the screen is merely awkward and still usable.
    public static func contentSize(for media: CGSize, visibleFrame: CGRect) -> CGSize {
        guard media.width > 0, media.height > 0 else { return chromeBase }
        let ratio = media.width / media.height

        var size = media
        if visibleFrame.width > 0, visibleFrame.height > 0 {
            let scale = min(1, min(
                visibleFrame.width / media.width,
                visibleFrame.height / media.height
            ))
            size = CGSize(
                width: (media.width * scale).rounded(),
                height: (media.height * scale).rounded()
            )
        }

        let minimum = minimumContentSize(for: ratio)
        guard size.width < minimum.width || size.height < minimum.height else {
            return size
        }
        return minimum
    }

    /// Moves `size` into `visibleFrame` around the window's current centre.
    ///
    /// Centre-preserving because the alternative — keeping the top-left — walks
    /// the window down and to the right every time a medium of a different
    /// shape is opened, and a player is a window people open many media in.
    ///
    /// The clamp is second and only ever translates: nudging the origin keeps
    /// the size, and therefore the aspect ratio, exactly as
    /// [`contentSize`](contentSize(for:visibleFrame:)) decided it.
    ///
    /// When the window is larger than the screen the two bounds on an axis
    /// contradict each other, and which one is applied last decides what stays
    /// reachable. Left wins on x and **top** wins on y — AppKit's y grows
    /// upward, so pinning the top is what keeps the traffic lights and the
    /// title bar on screen. Losing the bottom edge of an oversized window costs
    /// the user nothing; losing the close button costs them the window.
    public static func recentredFrame(
        size: CGSize,
        around current: CGRect,
        visibleFrame: CGRect
    ) -> CGRect {
        var origin = CGPoint(
            x: current.midX - size.width / 2,
            y: current.midY - size.height / 2
        )
        guard visibleFrame.width > 0, visibleFrame.height > 0 else {
            return CGRect(origin: origin, size: size)
        }
        origin.x = min(origin.x, visibleFrame.maxX - size.width)
        origin.x = max(origin.x, visibleFrame.minX)
        origin.y = max(origin.y, visibleFrame.minY)
        origin.y = min(origin.y, visibleFrame.maxY - size.height)
        return CGRect(origin: origin, size: size)
    }
}
