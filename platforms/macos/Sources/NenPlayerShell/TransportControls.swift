import NenCore
import SwiftUI

struct TransportControls: View {
    @ObservedObject var model: PlayerModel
    @State private var showsSubtitleMenu = false

    var body: some View {
        VStack(spacing: 10) {
            HStack(spacing: 12) {
                Button(action: model.togglePlayback) {
                    Image(systemName: model.isPlaying ? "pause.fill" : "play.fill")
                        .frame(width: 20)
                }
                .buttonStyle(.plain)
                .help(model.isPlaying ? "Duraklat" : "Oynat")

                Slider(
                    value: Binding(
                        get: { Double(model.displayedPositionMilliseconds) },
                        set: { model.previewSeek(to: UInt64(max(0, $0))) }
                    ),
                    in: 0...Double(max(1, model.durationMilliseconds ?? 1)),
                    onEditingChanged: { editing in
                        if !editing { model.commitSeek() }
                    }
                )
                .disabled(model.durationMilliseconds == nil)

                Button(model.durationText, action: model.toggleDurationMode)
                    .buttonStyle(.plain)
                    .monospacedDigit()
                    .help("Geçen ve kalan süre arasında geçiş yap")
            }

            HStack(spacing: 10) {
                Image(systemName: model.volume == 0 ? "speaker.slash.fill" : "speaker.wave.2.fill")
                Slider(
                    value: Binding(
                        get: { Double(model.volume) },
                        set: { model.setVolume(Float($0)) }
                    ),
                    in: 0...1
                )
                .frame(width: 130)
                Spacer()

                // §8: one subtitle button, and this is it. M3 has a single
                // surface — the same list is deliberately not mirrored into the
                // menu bar.
                Button {
                    showsSubtitleMenu.toggle()
                } label: {
                    Image(systemName: "captions.bubble")
                        .symbolVariant(model.selectedSubtitleToken == nil ? .none : .fill)
                        .frame(width: 20)
                }
                .buttonStyle(.plain)
                .help("Altyazılar")
                .disabled(!model.hasMedia)
                .popover(isPresented: $showsSubtitleMenu, arrowEdge: .top) {
                    SubtitleMenuView(model: model)
                }

                Text(model.playbackState.label)
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
        }
        .padding(.horizontal, 18)
        .padding(.vertical, 14)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 15))
        .padding(18)
        .foregroundStyle(.white)
    }
}

private extension FfiPlaybackState {
    var label: String {
        switch self {
        case .idle: "Hazır"
        case .buffering: "Yükleniyor"
        case .ready: "Hazır"
        case .playing: "Oynatılıyor"
        case .paused: "Duraklatıldı"
        case .ended: "Bitti"
        case .failed: "Oynatılamadı"
        }
    }
}
