import AppKit
import NenPlayerShell
import SwiftUI

@main
struct NenPlayerApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate
    @StateObject private var model = PlayerModel()

    var body: some Scene {
        Window("Nen Player", id: "player") {
            PlayerRootView(model: model)
        }
        .windowStyle(.hiddenTitleBar)
        // The player root publishes an aspect-correct minimum for its current
        // medium. Make that the scene's explicit resize policy so SwiftUI —
        // the window owner — enforces the same floor during user dragging.
        .windowResizability(.contentMinSize)
        .defaultSize(width: 1_080, height: 680)
        .commands {
            PlayerCommands(model: model)
        }

        Settings {
            SubtitlePreferencesSettingsView(model: model)
        }
    }
}

private final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    func applicationShouldHandleReopen(
        _ sender: NSApplication,
        hasVisibleWindows _: Bool
    ) -> Bool {
        if let playerWindow = sender.windows.first(where: { $0.identifier?.rawValue == "player" }),
           !playerWindow.isVisible {
            playerWindow.makeKeyAndOrderFront(nil)
            NotificationCenter.default.post(name: .nenPlayerWindowReopened, object: nil)
        }
        return true
    }
}
