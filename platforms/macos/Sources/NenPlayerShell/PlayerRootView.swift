import AppKit
import NenPlaybackMPV
import SwiftUI

public extension Notification.Name {
    static let nenPlayerWindowReopened = Notification.Name("player.nen.macos.window-reopened")
}

public struct PlayerRootView: View {
    @ObservedObject private var model: PlayerModel
    @State private var presentedPanel: PresentedPanel?
    /// The surface's own height, and the top edge of the transport bar in the
    /// same space. Together they are the share of the surface the chrome
    /// covers — the number ADR-0037 sends to the core.
    @State private var surfaceHeight: CGFloat = 0
    @State private var chromeTop: CGFloat = 0

    public init(model: PlayerModel) {
        self.model = model
    }

    public var body: some View {
        ZStack {
            Color.black
                .ignoresSafeArea()

            VideoSurface(model: model)
                // **The picture fills the window, not the safe area.**
                //
                // `Color.black` above already ignores it, so anything the video
                // surface leaves uncovered shows through as black — which is
                // exactly the letterbox NEN-068 exists to remove, produced by
                // the shell rather than by the renderer.
                //
                // Measured on the real app: a 1920x1080 medium in a window
                // locked to 16:9 still had a ~28 pt black margin left and
                // right and ~32 pt at the top, at every size and every
                // resolution. The window was the right shape; the surface
                // inside it was not.
                //
                // The chrome deliberately does **not** ignore the safe area:
                // the transport row and the traffic lights belong inside it.
                .ignoresSafeArea()
                .opacity(model.hasMedia ? 1 : 0)

            if model.hasMedia, model.playbackState == .buffering {
                BufferingOverlay()
            }

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
                    .padding(.bottom, TransportControls.height + 18)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
                    .transition(.opacity)
            }
        }
        // **The root imposes no minimum of its own** (ADR-0038 Karar 4).
        //
        // Deleting the old `.frame(minWidth: 720, minHeight: 450)` was not
        // enough: SwiftUI still propagates whatever minimum its *content* needs
        // to the window, and the transport row's intrinsic width is ≈590 pt.
        // Measured on the real app — a 16:9 medium opened correctly at 693x390
        // and was then pulled down to 594x327 by SwiftUI's own sizing, which is
        // neither the derived minimum nor the medium's ratio.
        //
        // Letting the root compress to zero puts the floor back in one place,
        // `WindowGeometry` via `contentMinSize`. The row can never actually be
        // squeezed, because AppKit refuses to resize past that floor — the
        // compression is permission SwiftUI needs, not a size the user reaches.
        .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
        .coordinateSpace(name: Self.surfaceSpace)
        .background(
            GeometryReader { surface in
                Color.clear
                    .onAppear { surfaceHeight = surface.size.height }
                    .onChange(of: surface.size.height) { _, height in
                        surfaceHeight = height
                    }
            }
        )
        .onChange(of: surfaceHeight) { _, _ in reportSubtitleInset() }
        .onChange(of: chromeTop) { _, _ in reportSubtitleInset() }
        .onChange(of: model.controlsVisible) { _, _ in reportSubtitleInset() }
        .background(
            WindowTitleWriter(
                title: model.windowTitle,
                controlsVisible: model.controlsVisible,
                hasMedia: model.hasMedia
            )
        )
        // The window's minimum comes from here and nowhere else (ADR-0038
        // Karar 4). A `.frame(minWidth:minHeight:)` alongside it would be a
        // second minimum: SwiftUI propagates its own to the window, AppKit
        // cannot honour two that disagree, and the one that wins at the
        // smallest size brings the black bars back — which is the defect this
        // whole task removes.
        .background(
            WindowGeometryWriter(
                geometry: model.videoGeometry,
                mediaRevision: model.mediaPresentationRevision
            )
        )
        .animation(.easeOut(duration: 0.24), value: model.controlsVisible)
        .animation(.easeOut(duration: 0.24), value: presentedPanel)
        .animation(.easeOut(duration: 0.16), value: model.transientMessage)
        .onChange(of: model.mediaPresentationRevision) { _, _ in
            setPresentedPanel(nil)
        }
        .onChange(of: model.fatalMessage) { _, fatalMessage in
            if fatalMessage != nil {
                setPresentedPanel(nil)
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
            setPresentedPanel(nil)
            model.applicationResignedActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            model.applicationBecameActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: .nenPlayerWindowReopened)) { _ in
            model.resume()
        }
        .onDisappear {
            setPresentedPanel(nil)
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

            if presentedPanel != nil {
                Color.clear
                    .contentShape(Rectangle())
                    .onTapGesture {
                        setPresentedPanel(nil)
                    }
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
            }

            ZStack(alignment: .bottomTrailing) {
                TransportControls(
                    model: model,
                    presentedPanel: presentedPanelBinding,
                    onInteractionOutsidePanel: {
                        setPresentedPanel(nil)
                    },
                    onToggleFullScreen: toggleFullScreen
                )
                .frame(maxWidth: .infinity)
                .background(
                    GeometryReader { bar in
                        Color.clear
                            .onAppear {
                                chromeTop = bar.frame(in: .named(Self.surfaceSpace)).minY
                            }
                            .onChange(of: bar.frame(in: .named(Self.surfaceSpace)).minY) {
                                _, top in chromeTop = top
                            }
                    }
                )

                Group {
                    switch presentedPanel {
                    case .some(.subtitles):
                        SubtitleMenuView(model: model)
                    case .some(.playbackRate):
                        PlaybackRatePanel(model: model)
                    case .none:
                        EmptyView()
                    }
                }
                .padding(.trailing, 26)
                .padding(.bottom, TransportControls.height)
                .transition(.move(edge: .bottom).combined(with: .opacity))
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottomTrailing)
            .opacity(model.controlsVisible ? 1 : 0)
            .offset(y: model.controlsVisible ? 0 : 14)
            .allowsHitTesting(model.controlsVisible)
        }
    }

    private var presentedPanelBinding: Binding<PresentedPanel?> {
        Binding(
            get: { presentedPanel },
            set: { setPresentedPanel($0) }
        )
    }

    /// The coordinate space the surface and the chrome are measured in.
    private static let surfaceSpace = "nen.player.surface"

    /// How far the transport bar sits above the bottom of the surface.
    ///
    /// Kept named because ADR-0037's layout test combines it with the measured
    /// bar. The flush NEN-067 chrome intentionally contributes no extra gap.
    static let chromeBottomPadding: CGFloat = 0

    /// Tells the model how much of the surface the transport bar covers.
    ///
    /// Measured rather than inferred from a legacy constant: the current bar
    /// lays out at 57 pt and this measurement remains the source sent to the
    /// renderer. What is sent is the share of the **height**, because that is
    /// the unit the renderer port takes (ADR-0037 Karar 2).
    ///
    /// Only the transport bar, not the subtitle panel above it: the panel is
    /// right-aligned and open only while the user is picking a row, and
    /// declaring it too would push the subtitle towards the middle of the
    /// picture (ADR-0037 Karar 6).
    ///
    /// Zero while the chrome is hidden, which is where the player spends most
    /// of a session — the subtitle belongs at the bottom of the picture
    /// whenever nothing is covering it.
    private func reportSubtitleInset() {
        guard surfaceHeight > 0 else { return }
        // `chromeTop` is zero until the bar has been laid out, and it is never
        // laid out at all in the empty and fatal states — there is no
        // transport bar without a medium. Reading that zero as "the chrome
        // covers everything" would push the subtitle half way up the picture
        // for as long as no bar exists.
        guard model.controlsVisible, chromeTop > 0 else {
            model.setSubtitleBottomInset(0)
            return
        }
        let covered = surfaceHeight - chromeTop
        guard covered > 0 else {
            model.setSubtitleBottomInset(0)
            return
        }
        model.setSubtitleBottomInset(min(covered / surfaceHeight, PlayerRootView.maximumInset))
    }

    /// The port's own ceiling, mirrored so a pathological layout is capped
    /// here rather than refused there.
    ///
    /// Not a disagreement with ADR-0037 Karar 2: the refusal still exists and
    /// still fires for anything this shell sends wrong. This only keeps a
    /// transient layout — a window mid-resize, a bar measured before the
    /// surface — from producing a value the core is right to reject.
    private static let maximumInset: CGFloat = 0.5

    private func setPresentedPanel(_ panel: PresentedPanel?) {
        presentedPanel = panel
        model.setControlsPinned(panel != nil)
    }

    private func toggleFullScreen() {
        NSApp.keyWindow?.toggleFullScreen(nil)
    }
}

private struct BufferingOverlay: View {
    var body: some View {
        ZStack {
            Color.black.opacity(0.16)
                .ignoresSafeArea()

            HStack(spacing: 10) {
                ProgressView()
                    .controlSize(.small)
                Text("Yükleniyor")
                    .font(.system(size: 12, weight: .medium))
            }
            .padding(.horizontal, 16)
            .frame(height: 42)
            .foregroundStyle(.white)
            .shadow(color: .black.opacity(0.72), radius: 1.5, y: 1)
            .background { GlassSurface(style: .floating(cornerRadius: 13)) }
        }
        .allowsHitTesting(false)
        .accessibilityElement(children: .combine)
        .accessibilityLabel("Video yükleniyor")
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
