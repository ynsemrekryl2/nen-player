import SwiftUI

struct PlaybackRatePanel: View {
    @ObservedObject var model: PlayerModel

    static func label(for rate: Float) -> String {
        switch rate {
        case 1: return "1×"
        case 0.5: return "0.5×"
        case 0.75: return "0.75×"
        case 1.5: return "1.5×"
        case 2: return "2×"
        default: return "\(rate)×"
        }
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 9) {
            Text("OYNATMA HIZI")
                .font(.system(size: 9.5).monospaced())
                .kerning(0.8)
                .foregroundStyle(.secondary)

            HStack(spacing: 4) {
                ForEach(PlayerModel.playbackRateOptions, id: \.self) { rate in
                    Button {
                        model.setPlaybackRate(rate)
                    } label: {
                        Text(Self.label(for: rate))
                            .font(.system(size: 11.5, weight: .medium, design: .rounded))
                            .monospacedDigit()
                            .frame(maxWidth: .infinity)
                            .frame(height: 31)
                            .background(
                                Color.white.opacity(model.playbackRate == rate ? 0.16 : 0.04),
                                in: RoundedRectangle(cornerRadius: 7)
                            )
                            .overlay {
                                if model.playbackRate == rate {
                                    RoundedRectangle(cornerRadius: 7)
                                        .stroke(Color.white.opacity(0.28), lineWidth: 0.5)
                                }
                            }
                    }
                    .buttonStyle(.plain)
                    .accessibilityLabel("\(Self.label(for: rate)) oynatma hızı")
                }
            }
        }
        .padding(.horizontal, 14)
        .padding(.top, 12)
        .padding(.bottom, 10)
        .frame(width: 334, height: 94)
        .environment(\.colorScheme, .dark)
        .foregroundStyle(.white)
        .shadow(color: .black.opacity(0.72), radius: 1.5, y: 1)
        .background { GlassSurface(style: .attachedPanel()) }
    }
}
