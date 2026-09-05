import NenCore
import SwiftUI

enum PresentedPanel: Equatable, Sendable {
    case subtitles
    case playbackRate

    static func toggling(_ requested: PresentedPanel, from current: PresentedPanel?)
        -> PresentedPanel? {
        current == requested ? nil : requested
    }
}

enum TransportLayoutElement: Hashable {
    case playPause
    case seek
    case fullScreen
}

private struct TransportLayoutPreferenceKey: PreferenceKey {
    static let defaultValue: [TransportLayoutElement: CGRect] = [:]

    static func reduce(
        value: inout [TransportLayoutElement: CGRect],
        nextValue: () -> [TransportLayoutElement: CGRect]
    ) {
        value.merge(nextValue(), uniquingKeysWith: { _, latest in latest })
    }
}

private let transportLayoutSpace = "nen.transport.layout"

struct TransportControls: View {
    static let height: CGFloat = 57
    static let horizontalPadding: CGFloat = 22

    @ObservedObject var model: PlayerModel
    @Binding private var presentedPanel: PresentedPanel?
    private let onInteractionOutsidePanel: () -> Void
    private let onToggleFullScreen: () -> Void
    private let onLayout: (([TransportLayoutElement: CGRect]) -> Void)?

    init(
        model: PlayerModel,
        presentedPanel: Binding<PresentedPanel?>,
        onInteractionOutsidePanel: @escaping () -> Void,
        onToggleFullScreen: @escaping () -> Void,
        onLayout: (([TransportLayoutElement: CGRect]) -> Void)? = nil
    ) {
        self.model = model
        _presentedPanel = presentedPanel
        self.onInteractionOutsidePanel = onInteractionOutsidePanel
        self.onToggleFullScreen = onToggleFullScreen
        self.onLayout = onLayout
    }

    var body: some View {
        HStack(spacing: 6) {
            transportButton(
                symbol: model.isPlaying ? "pause.fill" : "play.fill",
                help: model.isPlaying ? "Duraklat" : "Oynat",
                prominent: true
            ) {
                closePanel()
                model.togglePlayback()
            }
            .reportTransportFrame(.playPause, enabled: onLayout != nil)

            transportButton(symbol: "gobackward.5", help: "5 saniye geri") {
                closePanel()
                model.seekRelative(seconds: -5)
            }

            transportButton(symbol: "goforward.5", help: "5 saniye ileri") {
                closePanel()
                model.seekRelative(seconds: 5)
            }

            timeLabel(model.elapsedTimeText)
                .accessibilityLabel("Geçen süre")

            GlassSlider(
                value: Binding(
                    get: { Double(model.displayedPositionMilliseconds) },
                    set: { model.previewSeek(to: UInt64(max(0, $0))) }
                ),
                range: 0...Double(max(1, model.durationMilliseconds ?? 1)),
                trackHeight: 3,
                thumbDiameter: 10,
                keyboardStep: 5_000,
                accessibilityLabel: "Oynatma konumu",
                accessibilityValue: model.durationText,
                onEditingChanged: { editing in
                    if editing {
                        closePanel()
                    } else {
                        model.commitSeek()
                    }
                }
            )
            .frame(minWidth: 76, maxWidth: .infinity)
            .frame(height: 28)
            .disabled(model.durationMilliseconds == nil)
            .reportTransportFrame(.seek, enabled: onLayout != nil)

            Button(model.trailingTimeText) {
                closePanel()
                model.toggleDurationMode()
            }
            .buttonStyle(.plain)
            .font(.system(size: 11, weight: .medium, design: .monospaced))
            .monospacedDigit()
            .foregroundStyle(Color.white.opacity(0.90))
            .frame(minWidth: 43, alignment: .trailing)
            .help("Kalan ve toplam süre arasında geçiş yap")

            Image(systemName: model.volume == 0 ? "speaker.slash.fill" : "speaker.wave.2.fill")
                .font(.system(size: 12, weight: .medium))
                .frame(width: 22, height: 32)
                .accessibilityHidden(true)

            GlassSlider(
                value: Binding(
                    get: { Double(model.volume) },
                    set: { model.setVolume(Float($0)) }
                ),
                range: 0...1,
                trackHeight: 3,
                thumbDiameter: 9,
                keyboardStep: 0.05,
                accessibilityLabel: "Ses düzeyi",
                accessibilityValue: "%\(Int((model.volume * 100).rounded()))",
                onEditingChanged: { editing in
                    if editing { closePanel() }
                }
            )
            .frame(minWidth: 72, idealWidth: 92, maxWidth: 92)
            .frame(height: 28)

            Rectangle()
                .fill(Color.white.opacity(0.22))
                .frame(width: 0.5, height: 22)
                .padding(.horizontal, 2)

            Button {
                presentedPanel = PresentedPanel.toggling(.subtitles, from: presentedPanel)
            } label: {
                HStack(spacing: 5) {
                    Text("CC")
                        .font(.system(size: 8, weight: .bold, design: .monospaced))
                        .frame(width: 22, height: 15)
                        .overlay {
                            RoundedRectangle(cornerRadius: 3)
                                .stroke(Color.white.opacity(0.88), lineWidth: 1)
                        }
                    Text(
                        SubtitleMenuPresentation.selectionLabel(
                            selectedToken: model.selectedSubtitleToken,
                            menu: model.subtitleMenu
                        )
                    )
                    .font(.system(size: 10.5, weight: .medium))
                    .lineLimit(1)
                    .truncationMode(.tail)
                    .frame(minWidth: 28, idealWidth: 52, maxWidth: 68, alignment: .leading)
                }
                .foregroundStyle(Color.white.opacity(0.92))
                .padding(.horizontal, 7)
                .frame(height: 32)
                .background(
                    Color.white.opacity(presentedPanel == .subtitles ? 0.15 : 0.001),
                    in: RoundedRectangle(cornerRadius: 8)
                )
            }
            .buttonStyle(.plain)
            .help("Altyazılar")
            .disabled(!model.hasMedia)

            Button {
                presentedPanel = PresentedPanel.toggling(.playbackRate, from: presentedPanel)
            } label: {
                Text(PlaybackRatePanel.label(for: model.playbackRate))
                    .font(.system(size: 10.5, weight: .semibold, design: .rounded))
                    .monospacedDigit()
                    .frame(minWidth: 34)
                    .frame(height: 32)
                    .padding(.horizontal, 3)
                    .background(
                        Color.white.opacity(presentedPanel == .playbackRate ? 0.15 : 0.001),
                        in: RoundedRectangle(cornerRadius: 8)
                    )
            }
            .buttonStyle(.plain)
            .help("Oynatma hızı")
            .disabled(!model.hasMedia)

            transportButton(symbol: "arrow.up.left.and.arrow.down.right", help: "Tam Ekran") {
                closePanel()
                onToggleFullScreen()
            }
            .reportTransportFrame(.fullScreen, enabled: onLayout != nil)
        }
        .padding(.horizontal, Self.horizontalPadding)
        .frame(maxWidth: .infinity)
        .frame(height: Self.height)
        .foregroundStyle(.white)
        // Shadow only the glyphs and labels. The transport surface itself has
        // no card shadow in the ultra-thin recipe.
        .shadow(color: .black.opacity(0.72), radius: 1.5, y: 1)
        .background {
            GlassSurface(style: .transport)
                .contentShape(Rectangle())
                .onTapGesture(perform: closePanel)
        }
        .coordinateSpace(name: transportLayoutSpace)
        .onPreferenceChange(TransportLayoutPreferenceKey.self) { frames in
            onLayout?(frames)
        }
    }

    private func timeLabel(_ text: String) -> some View {
        Text(text)
            .font(.system(size: 11, weight: .medium, design: .monospaced))
            .monospacedDigit()
            .foregroundStyle(Color.white.opacity(0.90))
            .fixedSize(horizontal: true, vertical: false)
    }

    private func closePanel() {
        onInteractionOutsidePanel()
    }

    private func transportButton(
        symbol: String,
        help: String,
        prominent: Bool = false,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: prominent ? 15 : 13, weight: .semibold))
                .frame(width: prominent ? 34 : 30, height: 34)
                .background(
                    Color.white.opacity(prominent ? 0.14 : 0.001),
                    in: RoundedRectangle(cornerRadius: 9)
                )
                .contentShape(RoundedRectangle(cornerRadius: 9))
        }
        .buttonStyle(.plain)
        .help(help)
    }
}

private extension View {
    @ViewBuilder
    func reportTransportFrame(_ element: TransportLayoutElement, enabled: Bool) -> some View {
        if enabled {
            background {
                GeometryReader { geometry in
                    Color.clear.preference(
                        key: TransportLayoutPreferenceKey.self,
                        value: [element: geometry.frame(in: .named(transportLayoutSpace))]
                    )
                }
            }
        } else {
            self
        }
    }
}
