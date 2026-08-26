import AppKit
import SwiftUI

public struct PlayerCommands: Commands {
    @Environment(\.openWindow) private var openWindow
    private let model: PlayerModel

    public init(model: PlayerModel) {
        self.model = model
    }

    public var body: some Commands {
        CommandGroup(replacing: .newItem) {
            Button("Aç…", action: chooseMedia)
                .keyboardShortcut("o", modifiers: .command)
        }

        CommandMenu("Oynatma") {
            Button("Oynat / Duraklat", action: model.togglePlayback)
                .keyboardShortcut(KeyEquivalent(" "), modifiers: [])
            Divider()
            Button("5 Saniye Geri", action: { model.seekRelative(seconds: -5) })
                .keyboardShortcut(.leftArrow, modifiers: [])
            Button("5 Saniye İleri", action: { model.seekRelative(seconds: 5) })
                .keyboardShortcut(.rightArrow, modifiers: [])
            Button("30 Saniye Geri", action: { model.seekRelative(seconds: -30) })
                .keyboardShortcut(.leftArrow, modifiers: .shift)
            Button("30 Saniye İleri", action: { model.seekRelative(seconds: 30) })
                .keyboardShortcut(.rightArrow, modifiers: .shift)
            Divider()
            Button("Sesi Artır", action: { model.adjustVolume(by: 0.05) })
                .keyboardShortcut(.upArrow, modifiers: [])
            Button("Sesi Azalt", action: { model.adjustVolume(by: -0.05) })
                .keyboardShortcut(.downArrow, modifiers: [])
            Divider()
            Button("Tam Ekran", action: toggleFullScreen)
                .keyboardShortcut("f", modifiers: [])
            Button("Tam Ekrandan Çık", action: leaveFullScreen)
                .keyboardShortcut(KeyEquivalent("\u{1b}"), modifiers: [])
        }
    }

    private func chooseMedia() {
        openWindow(id: "player")
        model.resume()
        model.chooseMedia()
    }

    private func toggleFullScreen() {
        NSApp.keyWindow?.toggleFullScreen(nil)
    }

    private func leaveFullScreen() {
        guard let window = NSApp.keyWindow, window.styleMask.contains(.fullScreen) else { return }
        window.toggleFullScreen(nil)
    }
}
