import AppKit
import NenPlaybackMPV
import SwiftUI

public extension Notification.Name {
    static let nenPlayerWindowReopened = Notification.Name("player.nen.macos.window-reopened")
}

public struct PlayerRootView: View {
    @ObservedObject private var model: PlayerModel
    @State private var showsSubtitlePanel = false

    public init(model: PlayerModel) {
        self.model = model
    }

    public var body: some View {
        ZStack {
            Color.black
                .ignoresSafeArea()

            VideoSurface(model: model)
                .opacity(model.hasMedia ? 1 : 0)

            if let fatalMessage = model.fatalMessage {
                FatalState(
                    mediaName: model.mediaName,
                    message: fatalMessage,
                    openAction: model.chooseMedia
                )
            } else if model.mediaName == nil {
                EmptyState(
                    recentMediaName: model.recentMediaName,
                    openAction: model.chooseMedia,
                    openRecentAction: model.openRecentMedia
                )
            } else {
                playerChrome
            }

            if let transientMessage = model.transientMessage {
                Text(transientMessage)
                    .font(.callout.weight(.medium))
                    .padding(.horizontal, 14)
                    .padding(.vertical, 9)
                    .background(.ultraThinMaterial, in: Capsule())
                    .padding(.bottom, 150)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
                    .transition(.opacity)
            }
        }
        .frame(minWidth: 720, minHeight: 450)
        .background(
            WindowTitleWriter(
                title: model.windowTitle,
                controlsVisible: model.controlsVisible,
                hasMedia: model.hasMedia
            )
        )
        .animation(.easeOut(duration: 0.24), value: model.controlsVisible)
        .animation(.easeOut(duration: 0.24), value: showsSubtitlePanel)
        .animation(.easeOut(duration: 0.16), value: model.transientMessage)
        .onChange(of: model.mediaPresentationRevision) { _, _ in
            setSubtitlePanelPresented(false)
        }
        .onChange(of: model.fatalMessage) { _, fatalMessage in
            if fatalMessage != nil {
                setSubtitlePanelPresented(false)
            }
        }
        .onContinuousHover { phase in
            switch phase {
            case .active:
                model.pointerMoved()
            case .ended:
                model.pointerLeft()
            }
        }
        .dropDestination(for: URL.self) { urls, _ in
            guard let url = urls.first, url.isFileURL else { return false }
            model.openMedia(at: url)
            return true
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didResignActiveNotification)) { _ in
            setSubtitlePanelPresented(false)
            model.applicationResignedActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            model.applicationBecameActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: .nenPlayerWindowReopened)) { _ in
            model.resume()
        }
        .onDisappear {
            setSubtitlePanelPresented(false)
            model.shutdown()
        }
    }

    private var playerChrome: some View {
        ZStack {
            LinearGradient(
                colors: [.black.opacity(0.42), .clear],
                startPoint: .top,
                endPoint: .bottom
            )
            .frame(height: 116)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .top)
            .opacity(model.controlsVisible ? 1 : 0)
            .allowsHitTesting(false)

            if let mediaName = model.mediaName {
                Text(mediaName)
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(Color.white.opacity(0.96))
                    .lineLimit(1)
                    .truncationMode(.middle)
                    .shadow(color: .black.opacity(0.50), radius: 6, y: 1)
                    .padding(.leading, 92)
                    .padding(.trailing, 22)
                    .padding(.top, 18)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
                    .opacity(model.controlsVisible ? 1 : 0)
                    .offset(y: model.controlsVisible ? 0 : -10)
                    .allowsHitTesting(false)
            }

            if showsSubtitlePanel {
                Color.clear
                    .contentShape(Rectangle())
                    .onTapGesture {
                        setSubtitlePanelPresented(false)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }

            VStack(alignment: .trailing, spacing: 12) {
                if showsSubtitlePanel {
                    SubtitleMenuView(model: model)
                        .transition(.move(edge: .bottom).combined(with: .opacity))
                }

                TransportControls(
                    model: model,
                    showsSubtitlePanel: subtitlePanelBinding,
                    onInteractionOutsideSubtitlePanel: {
                        setSubtitlePanelPresented(false)
                    }
                )
                .frame(maxWidth: .infinity)
            }
                .padding(.horizontal, 22)
                .padding(.bottom, 20)
                .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
                .opacity(model.controlsVisible ? 1 : 0)
                .offset(y: model.controlsVisible ? 0 : 14)
                .allowsHitTesting(model.controlsVisible)
        }
    }

    private var subtitlePanelBinding: Binding<Bool> {
        Binding(
            get: { showsSubtitlePanel },
            set: { setSubtitlePanelPresented($0) }
        )
    }

    private func setSubtitlePanelPresented(_ presented: Bool) {
        showsSubtitlePanel = presented
        model.setControlsPinned(presented)
    }
}

private struct VideoSurface: NSViewRepresentable {
    let model: PlayerModel

    func makeNSView(context: Context) -> MPVVideoView {
        let view = MPVVideoView.makePlaybackSurface()
        model.attach(to: view)
        return view
    }

    func updateNSView(_ nsView: MPVVideoView, context: Context) {
        model.attach(to: nsView)
    }
}

private struct EmptyState: View {
    let recentMediaName: String?
    let openAction: () -> Void
    let openRecentAction: () -> Void

    var body: some View {
        VStack(spacing: 18) {
            Image(systemName: "play.rectangle.on.rectangle")
                .font(.system(size: 54, weight: .light))
                .foregroundStyle(.secondary)
            Text("Videoyu buraya bırakın")
                .font(.title2.weight(.semibold))
            Button("Aç…", action: openAction)
                .keyboardShortcut("o", modifiers: .command)
                .controlSize(.large)
            if let recentMediaName {
                Button(action: openRecentAction) {
                    Label(recentMediaName, systemImage: "clock.arrow.circlepath")
                        .lineLimit(1)
                }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
                .help("Son açılan medyayı yeniden aç")
            }
        }
        .padding(36)
        .frame(maxWidth: 460)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
        .foregroundStyle(.white)
    }
}

private struct FatalState: View {
    let mediaName: String?
    let message: String
    let openAction: () -> Void

    var body: some View {
        VStack(spacing: 14) {
            Image(systemName: "exclamationmark.triangle")
                .font(.system(size: 44, weight: .light))
                .foregroundStyle(.orange)
            if let mediaName {
                Text(mediaName)
                    .font(.headline)
                    .lineLimit(1)
            }
            Text(message)
                .foregroundStyle(.secondary)
            Button("Başka dosya aç", action: openAction)
                .controlSize(.large)
        }
        .padding(36)
        .frame(maxWidth: 460)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 20))
        .foregroundStyle(.white)
    }
}

private struct WindowTitleWriter: NSViewRepresentable {
    let title: String
    let controlsVisible: Bool
    let hasMedia: Bool

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeNSView(context: Context) -> NSView {
        NSView()
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        let title = title
        let showButtons = !hasMedia || controlsVisible
        DispatchQueue.main.async {
            guard let window = nsView.window else { return }
            window.title = title
            window.titleVisibility = .hidden
            window.titlebarAppearsTransparent = true
            // Deliberately do not set `representedURL`: its proxy icon exposes
            // the full path, which ADR-0031 forbids on evidence surfaces.
            window.representedURL = nil
            context.coordinator.updateButtons(in: window, visible: showButtons)
        }
    }

    @MainActor
    final class Coordinator {
        private var generation = 0

        func updateButtons(in window: NSWindow, visible: Bool) {
            generation += 1
            let updateGeneration = generation
            let buttons = [NSWindow.ButtonType.closeButton, .miniaturizeButton, .zoomButton]
                .compactMap(window.standardWindowButton)

            guard !window.styleMask.contains(.fullScreen) else {
                for button in buttons {
                    button.isHidden = false
                    button.isEnabled = true
                    button.alphaValue = 1
                }
                return
            }

            if visible {
                for button in buttons {
                    button.isHidden = false
                    button.isEnabled = true
                }
            }

            NSAnimationContext.runAnimationGroup { animation in
                animation.duration = 0.24
                animation.timingFunction = CAMediaTimingFunction(name: .easeOut)
                for button in buttons {
                    button.animator().alphaValue = visible ? 1 : 0
                }
            }

            guard !visible else { return }
            Task { @MainActor [weak self, weak window] in
                try? await Task.sleep(for: .milliseconds(240))
                guard let self,
                      let window,
                      updateGeneration == generation,
                      !window.styleMask.contains(.fullScreen)
                else { return }
                for buttonType in [NSWindow.ButtonType.closeButton, .miniaturizeButton, .zoomButton] {
                    guard let button = window.standardWindowButton(buttonType) else { continue }
                    button.isEnabled = false
                    button.isHidden = true
                }
            }
        }
    }
}
