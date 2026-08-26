import NenPlayerShell
import SwiftUI

@main
struct NenPlayerApp: App {
    @StateObject private var model = PlayerModel()

    var body: some Scene {
        WindowGroup {
            PlayerRootView(model: model)
        }
        .defaultSize(width: 1_080, height: 680)
        .commands {
            PlayerCommands(model: model)
        }

        Settings {
            SettingsPlaceholderView()
        }
    }
}
