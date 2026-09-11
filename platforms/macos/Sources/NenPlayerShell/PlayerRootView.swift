import AppKit
import NenPlaybackMPV
import SwiftUI

public extension Notification.Name {
    static let nenPlayerWindowReopened = Notification.Name("player.nen.macos.window-reopened")
}

public struct PlayerRootView: View {
    @ObservedObject private var model: PlayerModel
    private let onTransportLayout: (([TransportLayoutElement: CGRect]) -> Void)?
    @State private var presentedPanel: PresentedPanel?
    /// The surface's own height, and the top edge of the transport bar in the
    /// same space. Together they are the share of the surface the chrome
    /// covers — the number ADR-0037 sends to the core.
    @State private var surfaceHeight: CGFloat = 0
    @State private var chromeTop: CGFloat = 0
    /// The part of the full-size content view reserved by the hidden titlebar.
    /// SwiftUI cannot expose this while it computes `fittingSize`; AppKit's
    /// window writer measures it and feeds it back after attachment.
    @State private var safeAreaOverhead: CGSize = .zero
    /// Delivers `Esc` while this window is key (NEN-047). Neither
    /// `PlayerCommands`' own `Esc` key equivalent nor SwiftUI's
    /// `onExitCommand` fired on the real app: measured with the media
    /// playing and this window key, both stayed silent while a mouse click
    /// on the equivalent menu item worked, isolating the gap to key delivery
    /// rather than to focus, `isEnabled`, or the action itself. A local
    /// monitor is the AppKit primitive both of those are built on, so it is
    /// the mechanism most likely to receive the key where they did not —
    /// though live confirmation of that itself hit a wall: with the local
    /// monitor in place and logging every key it saw, a plain `Esc` produced
    /// no log line even for a stock, unrelated `NSOpenPanel`'s own built-in
    /// cancel-on-`Esc` in the same run, so `Esc` delivery could not be
    /// exercised live in that environment at all, independent of this code.
    /// `bash scripts/test-macos.sh` covers what a live run cannot here: that
    /// `setFullScreen` drives `isFullScreen`, the flag this monitor and the
    /// menu item both gate on. It is scoped to the player window by checking
    /// `event.window` — otherwise a local monitor is app-wide and would also
    /// see `Esc` while Settings is key — and it lets every event through
    /// unconsumed except the one case this task adds: `Esc`, this window,
    /// full screen.
    @State private var escapeMonitor: Any?

    public init(model: PlayerModel) {
        self.model = model
        onTransportLayout = nil
    }

    init(
        model: PlayerModel,
        onTransportLayout: @escaping ([TransportLayoutElement: CGRect]) -> Void
    ) {
        self.model = model
        self.onTransportLayout = onTransportLayout
    }

    public var body: some View {
        let safeAreaOverheadBinding = $safeAreaOverhead
        let minimum = WindowGeometry.minimumLayoutSize(
            for: displaySize,
            safeAreaOverhead: safeAreaOverhead
        )

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
                    recentMedia: model.recentMedia,
                    openAction: model.chooseMedia,
                    openRecentAction: model.openRecentMedia
                )
            } else {
                playerChrome
            }

            // The transient message and the translation status pill share
            // one bottom-aligned slot rather than two independent overlays —
            // both are ephemeral, neither is permanent chrome, and a start/
            // finish transient can briefly coexist with a still-draining
            // progress pill right at the handoff between them.
            VStack(spacing: 8) {
                if let transientMessage = model.transientMessage {
                    Text(transientMessage)
                        .font(.callout.weight(.medium))
                        .padding(.horizontal, 14)
                        .padding(.vertical, 9)
                        .background(.ultraThinMaterial, in: Capsule())
                        .transition(.opacity)
                }
                if let translationProgress = model.translationProgress {
                    TranslationStatusPill(
                        state: translationProgress,
                        cancelAction: model.cancelTranslation
                    )
                    .transition(.opacity)
                }
            }
            .padding(.bottom, TransportControls.height + 18)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
        }
        // The Window scene derives its user-resize floor from the root view.
        // Advertising zero here let SwiftUI overwrite the `contentMinSize`
        // written through AppKit and made the window draggable below the size
        // the transport needs. This minimum is the same aspect-correct answer
        // that sizes the opening window, so the content and aspect constraints
        // cannot disagree at the floor (ADR-0038 Karar 4).
        .frame(
            minWidth: minimum.width,
            maxWidth: .infinity,
            minHeight: minimum.height,
            maxHeight: .infinity
        )
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
        // AppKit still owns the aspect lock and one-time opening size. The
        // root above owns the minimum so SwiftUI's scene sizing and AppKit do
        // not race to write `contentMinSize`.
        .background(
            WindowGeometryWriter(
                geometry: model.videoGeometry,
                mediaRevision: model.mediaPresentationRevision,
                onSafeAreaOverheadChange: { overhead in
                    guard safeAreaOverheadBinding.wrappedValue != overhead else { return }
                    safeAreaOverheadBinding.wrappedValue = overhead
                }
            )
        )
        .animation(.easeOut(duration: 0.24), value: model.controlsVisible)
        .animation(.easeOut(duration: 0.24), value: presentedPanel)
        .animation(.easeOut(duration: 0.16), value: model.transientMessage)
        .animation(.easeOut(duration: 0.16), value: model.translationProgress)
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
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didEnterFullScreenNotification)) { notification in
            if isPlayerWindow(notification.object) {
                model.setFullScreen(true)
            }
        }
        .onReceive(NotificationCenter.default.publisher(for: NSWindow.didExitFullScreenNotification)) { notification in
            if isPlayerWindow(notification.object) {
                model.setFullScreen(false)
            }
        }
        .onAppear {
            installEscapeMonitor()
            // Measured on the real `.app`: at launch, AppKit/SwiftUI mount
            // this view, tear it down, and mount it again — all within
            // ~100 ms, before the window ever visibly changes — for reasons
            // internal to `Window(id:)` + `.windowResizability` and outside
            // this view's control. The video surface's own `NSView` survives
            // that churn, but `onDisappear` below still fires and shuts the
            // playback session down with it. `resume()` is exactly the
            // self-heal `.nenPlayerWindowReopened` already uses for the
            // Dock-reopen case (NEN-046): a no-op when a session already
            // exists, and otherwise a straight `attach(to:)` on the surface
            // that never actually went away. Without it, a handoff (NEN-080)
            // — or in principle any load — arriving during that window would
            // queue behind a session that nothing ever rebuilds.
            model.resume()
        }
        .onDisappear {
            removeEscapeMonitor()
            setPresentedPanel(nil)
            model.shutdown()
        }
        // Scopes the seven playback shortcuts in `PlayerCommands` to this
        // scene: present only while the player window is key, `nil` the
        // instant Settings (or any other scene) takes focus (NEN-047).
        .focusedSceneValue(\.playerModel, model)
    }

    private var displaySize: CGSize? {
        guard let geometry = model.videoGeometry else { return nil }
        return CGSize(width: CGFloat(geometry.width), height: CGFloat(geometry.height))
    }

    private func isPlayerWindow(_ object: Any?) -> Bool {
        (object as? NSWindow)?.identifier?.rawValue == "player"
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
            // The top shade belongs to the window, including the traffic-light
            // safe area. The title below remains in the safe area so it never
            // competes with the system buttons.
            .ignoresSafeArea(.container, edges: .top)
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
                    onToggleFullScreen: toggleFullScreen,
                    onLayout: onTransportLayout
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

    private func leaveFullScreen() {
        guard let window = NSApp.keyWindow, window.styleMask.contains(.fullScreen) else { return }
        window.toggleFullScreen(nil)
    }

    /// Delivers `Esc` while this window is key (NEN-047). Neither
    /// `PlayerCommands`' own `Esc` key equivalent nor SwiftUI's
    /// `onExitCommand` fired on the real app — measured with the media
    /// playing and this window key, both stayed silent while a mouse click
    /// on the equivalent menu item worked, isolating the gap to key delivery
    /// rather than to focus, `isEnabled`, or the action itself. A local
    /// monitor is the AppKit primitive both of those are built on, so it
    /// receives the key where they did not. Every event is returned
    /// unconsumed — passed through exactly as the system would have without
    /// this monitor — except the one case this task adds: `Esc`, this
    /// window key, full screen.
    private func installEscapeMonitor() {
        guard escapeMonitor == nil else { return }
        escapeMonitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { event in
            guard event.keyCode == 53, isPlayerWindow(event.window), model.isFullScreen else {
                return event
            }
            leaveFullScreen()
            return nil
        }
    }

    private func removeEscapeMonitor() {
        guard let escapeMonitor else { return }
        NSEvent.removeMonitor(escapeMonitor)
        self.escapeMonitor = nil
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
    let recentMedia: [RecentMediaEntry]
    let openAction: () -> Void
    let openRecentAction: (RecentMediaEntry.ID) -> Void

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
            if !recentMedia.isEmpty {
                VStack(spacing: 6) {
                    ForEach(recentMedia) { entry in
                        Button {
                            openRecentAction(entry.id)
                        } label: {
                            Label(entry.displayName, systemImage: "clock.arrow.circlepath")
                                .lineLimit(1)
                        }
                        .buttonStyle(.plain)
                        .foregroundStyle(.secondary)
                        .help("Son açılan medyayı yeniden aç")
                    }
                }
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
