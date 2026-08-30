import AppKit
import SwiftUI

enum GlassSurfaceStyle: Equatable {
    case transport
    case attachedPanel(cornerRadius: CGFloat = 18)
    case floating(cornerRadius: CGFloat = 16)
}

/// NEN-067's ultra-thin glass, with explicit edge rules for each placement.
///
/// CSS's `blur(10px)` has no one-to-one SwiftUI parameter. `.hudWindow`
/// supplies the platform blur and the reference's 10% white tint is applied
/// above it. Contrast belongs to the controls, not to a heavy dark card.
struct GlassSurface: View {
    let style: GlassSurfaceStyle

    init(style: GlassSurfaceStyle = .floating()) {
        self.style = style
    }

    @ViewBuilder
    var body: some View {
        switch style {
        case .transport:
            glass
                .overlay(alignment: .top) {
                    Rectangle()
                        .fill(Color.white.opacity(0.30))
                        .frame(height: 0.5)
                }
        case let .attachedPanel(cornerRadius):
            glass
                .clipShape(
                    UnevenRoundedRectangle(
                        topLeadingRadius: cornerRadius,
                        bottomLeadingRadius: 0,
                        bottomTrailingRadius: 0,
                        topTrailingRadius: cornerRadius,
                        style: .continuous
                    )
                )
                .overlay(alignment: .top) {
                    Rectangle()
                        .fill(Color.white.opacity(0.30))
                        .frame(height: 0.5)
                }
                .overlay(alignment: .leading) {
                    Rectangle()
                        .fill(Color.white.opacity(0.16))
                        .frame(width: 0.5)
                }
                .overlay(alignment: .trailing) {
                    Rectangle()
                        .fill(Color.white.opacity(0.16))
                        .frame(width: 0.5)
                }
        case let .floating(cornerRadius):
            glass
                .clipShape(RoundedRectangle(cornerRadius: cornerRadius, style: .continuous))
                .overlay {
                    RoundedRectangle(cornerRadius: cornerRadius, style: .continuous)
                        .stroke(Color.white.opacity(0.22), lineWidth: 0.5)
                }
        }
    }

    private var glass: some View {
        ZStack {
            HUDVisualEffectView()
            Color.white.opacity(0.10)
        }
    }
}

private struct HUDVisualEffectView: NSViewRepresentable {
    func makeNSView(context: Context) -> NSVisualEffectView {
        let view = NSVisualEffectView()
        view.material = .hudWindow
        view.blendingMode = .withinWindow
        view.state = .active
        view.isEmphasized = true
        return view
    }

    func updateNSView(_ nsView: NSVisualEffectView, context: Context) {
        nsView.state = .active
    }
}

/// A visually small track with a deliberately larger interaction surface.
///
/// The drag path, focus commands and accessibility adjustment all meet at the
/// same binding. A custom-looking slider therefore does not trade away the
/// keyboard and VoiceOver behaviours the system slider previously supplied.
struct GlassSlider: View {
    @Binding var value: Double

    let range: ClosedRange<Double>
    let trackHeight: CGFloat
    let thumbDiameter: CGFloat
    let keyboardStep: Double
    let accessibilityLabel: String
    let accessibilityValue: String
    let onEditingChanged: (Bool) -> Void

    @Environment(\.isEnabled) private var isEnabled
    @FocusState private var isFocused: Bool
    @State private var isEditing = false

    var body: some View {
        GeometryReader { geometry in
            let width = max(1, geometry.size.width)
            let progress = normalizedValue
            let thumbX = progress * width

            ZStack(alignment: .leading) {
                Capsule()
                    .fill(Color.white.opacity(0.16))
                    .frame(height: trackHeight)

                Capsule()
                    .fill(Color.accentColor)
                    .frame(width: max(trackHeight, thumbX), height: trackHeight)

                Circle()
                    .fill(.white)
                    .frame(width: thumbDiameter, height: thumbDiameter)
                    .shadow(color: .black.opacity(0.45), radius: 3, y: 1)
                    .offset(x: min(
                        max(0, thumbX - thumbDiameter / 2),
                        max(0, width - thumbDiameter)
                    ))
            }
            .frame(maxHeight: .infinity, alignment: .center)
            .contentShape(Rectangle())
            .overlay {
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(
                        isFocused ? Color.accentColor.opacity(0.70) : .clear,
                        lineWidth: 2
                    )
            }
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { gesture in
                        guard isEnabled else { return }
                        if !isEditing {
                            isEditing = true
                            onEditingChanged(true)
                        }
                        setValue(from: gesture.location.x, width: width)
                    }
                    .onEnded { gesture in
                        guard isEnabled else { return }
                        setValue(from: gesture.location.x, width: width)
                        isEditing = false
                        onEditingChanged(false)
                    }
            )
        }
        .opacity(isEnabled ? 1 : 0.45)
        .focusable(isEnabled)
        .focused($isFocused)
        .onMoveCommand { direction in
            guard isEnabled else { return }
            switch direction {
            case .left:
                adjust(by: -keyboardStep)
            case .right:
                adjust(by: keyboardStep)
            default:
                break
            }
        }
        .accessibilityElement()
        .accessibilityLabel(Text(accessibilityLabel))
        .accessibilityValue(Text(accessibilityValue))
        .accessibilityAdjustableAction { direction in
            guard isEnabled else { return }
            switch direction {
            case .increment:
                adjust(by: keyboardStep)
            case .decrement:
                adjust(by: -keyboardStep)
            @unknown default:
                break
            }
        }
    }

    private var normalizedValue: Double {
        guard range.upperBound > range.lowerBound else { return 0 }
        let clamped = min(range.upperBound, max(range.lowerBound, value))
        return (clamped - range.lowerBound) / (range.upperBound - range.lowerBound)
    }

    private func setValue(from x: CGFloat, width: CGFloat) {
        let fraction = min(1, max(0, x / width))
        value = range.lowerBound + Double(fraction) * (range.upperBound - range.lowerBound)
    }

    private func adjust(by delta: Double) {
        onEditingChanged(true)
        value = min(range.upperBound, max(range.lowerBound, value + delta))
        onEditingChanged(false)
    }
}
