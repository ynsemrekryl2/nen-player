import AppKit
import SwiftUI

public struct PlayerCommands: Commands {
    @Environment(\.openWindow) private var openWindow
    // The player-scene-scoped shortcuts are gated on focus (NEN-047) so they
    // go dead the instant another scene — Settings — is key. `⌘O` stays
    // bound to the app's own model regardless of focus: NEN-046 already
    // proved it must bring the player window back and open the picker while
    // that window is fully closed, i.e. before any scene could hold focus.
    @ObservedObject private var model: PlayerModel
    @FocusedValue(\.playerModel) private var focusedModel: PlayerModel?

    public init(model: PlayerModel) {
        self.model = model
    }

    public var body: some Commands {
        CommandGroup(replacing: .newItem) {
            Button("Aç…", action: chooseMedia)
                .keyboardShortcut("o", modifiers: .command)
            Button("Altyazı Dosyası Yükle…", action: model.chooseSubtitleFile)
                .keyboardShortcut("o", modifiers: [.command, .shift])
            Divider()
            // Empty is a harmless no-op in `clearRecentMedia()`, but disabling
            // here keeps the item from ever looking actionable on a fresh
            // install (NEN-042).
            Button("Son Açılanları Temizle", action: model.clearRecentMedia)
                .disabled(model.recentMedia.isEmpty)
        }

        CommandMenu("Oynatma") {
            Button("Oynat / Duraklat", action: model.togglePlayback)
                .keyboardShortcut(KeyEquivalent(" "), modifiers: [])
                .disabled(focusedModel == nil)
            Divider()
            Button("5 Saniye Geri") { model.seekRelative(seconds: -5) }
                .keyboardShortcut(.leftArrow, modifiers: [])
                .disabled(focusedModel == nil)
            Button("5 Saniye İleri") { model.seekRelative(seconds: 5) }
                .keyboardShortcut(.rightArrow, modifiers: [])
                .disabled(focusedModel == nil)
            Button("30 Saniye Geri") { model.seekRelative(seconds: -30) }
                .keyboardShortcut(.leftArrow, modifiers: .shift)
                .disabled(focusedModel == nil)
            Button("30 Saniye İleri") { model.seekRelative(seconds: 30) }
                .keyboardShortcut(.rightArrow, modifiers: .shift)
                .disabled(focusedModel == nil)
            Divider()
            Button("Sesi Artır") { model.adjustVolume(by: 0.05) }
                .keyboardShortcut(.upArrow, modifiers: [])
                .disabled(focusedModel == nil)
            Button("Sesi Azalt") { model.adjustVolume(by: -0.05) }
                .keyboardShortcut(.downArrow, modifiers: [])
                .disabled(focusedModel == nil)
            Divider()
            Button("Tam Ekran", action: toggleFullScreen)
                .keyboardShortcut("f", modifiers: [])
                .disabled(focusedModel == nil)
            // The key equivalent below is cosmetic, not the real path out of
            // full screen (NEN-047): measured on the real app, AppKit never
            // routes a plain, unmodified Escape through
            // `NSMenu.performKeyEquivalent` — this item stayed dead whether
            // it was enabled or disabled, while a mouse click on it worked
            // fine, isolating the gap to key-equivalent delivery
            // specifically rather than to focus or enabled state. The actual
            // exit is `PlayerRootView`'s local `NSEvent` monitor, the AppKit
            // primitive `NSMenu` key equivalents are themselves built on —
            // see that file for what it could and could not confirm live.
            // This entry stays for menu discoverability and the mouse-click
            // path; `leaveFullScreen()`'s own style-mask guard keeps a click
            // outside full screen a harmless no-op, so gating this on focus
            // alone (like every other item here) is enough.
            Button("Tam Ekrandan Çık", action: leaveFullScreen)
                .keyboardShortcut(KeyEquivalent("\u{1b}"), modifiers: [])
                .disabled(focusedModel == nil)
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
