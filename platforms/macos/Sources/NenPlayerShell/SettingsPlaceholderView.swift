import SwiftUI

/// NEN-024 establishes the standard macOS Settings scene. NEN-037 replaces
/// this intentionally small placeholder with the two language selectors.
public struct SettingsPlaceholderView: View {
    public init() {}

    public var body: some View {
        Text("Henüz ayarlanabilir bir seçenek yok.")
            .foregroundStyle(.secondary)
            .frame(width: 360, height: 120)
            .padding()
    }
}
