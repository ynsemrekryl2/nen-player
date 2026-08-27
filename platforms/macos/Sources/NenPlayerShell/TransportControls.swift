import NenCore
import SwiftUI

struct TransportControls: View {
    @ObservedObject var model: PlayerModel
    @State private var showsSubtitleMenu = false

    var body: some View {
        VStack(spacing: 14) {
            GlassSlider(
                value: Binding(
                    get: { Double(model.displayedPositionMilliseconds) },
                    set: { model.previewSeek(to: UInt64(max(0, $0))) }
                ),
                range: 0...Double(max(1, model.durationMilliseconds ?? 1)),
                trackHeight: 5,
                thumbDiameter: 14,
                keyboardStep: 5_000,
                accessibilityLabel: "Oynatma konumu",
                accessibilityValue: model.durationText,
                onEditingChanged: { editing in
                    if !editing { model.commitSeek() }
                }
            )
            .frame(height: 22)
            .disabled(model.durationMilliseconds == nil)

            HStack(spacing: 6) {
                Button(action: model.togglePlayback) {
                    Image(systemName: model.isPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 17, weight: .semibold))
                        .frame(width: 46, height: 46)
                        .background(Color.white.opacity(0.14), in: RoundedRectangle(cornerRadius: 15))
                        .overlay {
                            RoundedRectangle(cornerRadius: 15)
                                .stroke(Color.white.opacity(0.14), lineWidth: 1)
                        }
                }
                .buttonStyle(.plain)
                .help(model.isPlaying ? "Duraklat" : "Oynat")

                transportButton(symbol: "gobackward.5", help: "5 saniye geri") {
                    model.seekRelative(seconds: -5)
                }
                transportButton(symbol: "goforward.5", help: "5 saniye ileri") {
                    model.seekRelative(seconds: 5)
                }

                Image(systemName: model.volume == 0 ? "speaker.slash.fill" : "speaker.wave.2.fill")
                    .font(.system(size: 13, weight: .medium))
                    .frame(width: 28, height: 34)
                    .accessibilityHidden(true)

                GlassSlider(
                    value: Binding(
                        get: { Double(model.volume) },
                        set: { model.setVolume(Float($0)) }
                    ),
                    range: 0...1,
                    trackHeight: 4,
                    thumbDiameter: 11,
                    keyboardStep: 0.05,
                    accessibilityLabel: "Ses düzeyi",
                    accessibilityValue: "%\(Int((model.volume * 100).rounded()))",
                    onEditingChanged: { _ in }
                )
                .frame(width: 92, height: 22)

                Button(model.durationText, action: model.toggleDurationMode)
                    .buttonStyle(.plain)
                    .font(.system(size: 12, weight: .medium, design: .monospaced))
                    .foregroundStyle(Color.white.opacity(0.88))
                    .monospacedDigit()
                    .padding(.leading, 4)
                    .help("Geçen ve kalan süre arasında geçiş yap")

                Spacer()

                // §8: one subtitle button, and this is it. M3 has a single
                // surface — the same list is deliberately not mirrored into the
                // menu bar.
                Button {
                    showsSubtitleMenu.toggle()
                } label: {
                    HStack(spacing: 7) {
                        Text("CC")
                            .font(.system(size: 8, weight: .bold, design: .monospaced))
                            .frame(width: 24, height: 16)
                            .overlay {
                                RoundedRectangle(cornerRadius: 4)
                                    .stroke(Color.white.opacity(0.80), lineWidth: 1.5)
                            }
                        Text(
                            SubtitleMenuPresentation.selectionLabel(
                                selectedToken: model.selectedSubtitleToken,
                                menu: model.subtitleMenu
                            )
                        )
                        .font(.system(size: 11, weight: .medium))
                        .lineLimit(1)
                    }
                    .foregroundStyle(Color.white.opacity(0.80))
                    .padding(.horizontal, 12)
                    .frame(height: 36)
                    .background(
                        Color.white.opacity(showsSubtitleMenu ? 0.18 : 0.07),
                        in: RoundedRectangle(cornerRadius: 12)
                    )
                }
                .buttonStyle(.plain)
                .help("Altyazılar")
                .disabled(!model.hasMedia)
                .popover(isPresented: $showsSubtitleMenu, arrowEdge: .top) {
                    SubtitleMenuView(model: model)
                }
            }
        }
        .padding(.horizontal, 20)
        .padding(.top, 18)
        .padding(.bottom, 14)
        .background { GlassSurface() }
        .padding(.horizontal, 22)
        .padding(.bottom, 20)
        .foregroundStyle(.white)
    }

    private func transportButton(
        symbol: String,
        help: String,
        action: @escaping () -> Void
    ) -> some View {
        Button(action: action) {
            Image(systemName: symbol)
                .font(.system(size: 16, weight: .medium))
                .frame(width: 38, height: 38)
                .contentShape(RoundedRectangle(cornerRadius: 12))
        }
        .buttonStyle(.plain)
        .help(help)
    }
}
