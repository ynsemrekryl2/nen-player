import AppKit
import NenPlaybackMPV
import SwiftUI

public extension Notification.Name {
    static let nenPlayerWindowReopened = Notification.Name("player.nen.macos.window-reopened")
}

public struct PlayerRootView: View {
    @ObservedObject private var model: PlayerModel

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
            } else if model.controlsVisible {
                TransportControls(model: model)
                    .transition(.opacity)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
            }

            if let transientMessage = model.transientMessage {
                Text(transientMessage)
                    .font(.callout.weight(.medium))
                    .padding(.horizontal, 14)
                    .padding(.vertical, 9)
                    .background(.ultraThinMaterial, in: Capsule())
                    .padding(.bottom, 104)
                    .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .bottom)
                    .transition(.opacity)
            }
        }
        .frame(minWidth: 720, minHeight: 450)
        .background(WindowTitleWriter(title: model.windowTitle))
        .animation(.easeOut(duration: 0.16), value: model.controlsVisible)
        .animation(.easeOut(duration: 0.16), value: model.transientMessage)
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
            model.applicationResignedActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: NSApplication.didBecomeActiveNotification)) { _ in
            model.applicationBecameActive()
        }
        .onReceive(NotificationCenter.default.publisher(for: .nenPlayerWindowReopened)) { _ in
            model.resume()
        }
        .onDisappear {
            model.shutdown()
        }
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

    func makeNSView(context: Context) -> NSView {
        NSView()
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        nsView.window?.title = title
        // Deliberately do not set `representedURL`: its proxy icon exposes the
        // full path, which ADR-0031 forbids on recorded evidence surfaces.
        nsView.window?.representedURL = nil
    }
}
