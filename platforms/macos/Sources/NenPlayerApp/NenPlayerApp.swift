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
                // The delegate is constructed before `model` exists (SwiftUI
                // owns both), so a handoff arriving during launch has nowhere
                // to land yet — attaching here closes that gap the moment the
                // window's content appears, and replays anything queued
                // meanwhile (`HandoffCoordinator`, `NenPlayerShell`).
                .onAppear { appDelegate.handoff.attach(model) }
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

/// Handles what AppKit calls the delegate for; decides nothing itself.
///
/// ADR-0043 Karar 4 puts every judgement about a handoff in the core, reached
/// through `HandoffIntake`, and every reconciliation with the model's
/// lifecycle in `HandoffCoordinator` (both `NenPlayerShell`, both unit
/// tested — this type is not, since `NenPlayerApp` has no test target). What
/// is left here is exactly the two things only AppKit can hand over — `argv`
/// at launch, and an `application(_:open:)` callback for
/// open-with/`nenplayer://` — passed straight through.
@MainActor
private final class AppDelegate: NSObject, NSApplicationDelegate {
    let handoff = HandoffCoordinator()

    func applicationDidFinishLaunching(_: Notification) {
        // NEN-078 Bulgu 4: a sender like Stremio launches the executable
        // directly with its own CLI arguments — this is the argv leg of
        // ADR-0043 Karar 1, and it is only ever observable here, at launch.
        handoff.deliver(HandoffIntake.fromArgv(CommandLine.arguments))
    }

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

    /// Open-with (`CFBundleDocumentTypes`) and the `nenplayer` custom scheme
    /// (`CFBundleURLTypes`) both funnel through this single AppKit callback —
    /// ADR-0043 Karar 1's other two legs. One medium is played at a time, so
    /// only the last URL of a batch is acted on.
    func application(_ application: NSApplication, open urls: [URL]) {
        guard let url = urls.last else { return }
        handoff.deliver(HandoffIntake.fromURL(url.absoluteString))
        if let playerWindow = application.windows.first(where: { $0.identifier?.rawValue == "player" }) {
            playerWindow.makeKeyAndOrderFront(nil)
        }
    }
}
