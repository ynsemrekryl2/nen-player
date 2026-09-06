import SwiftUI

/// M3's one Settings surface.
///
/// ADR-0031 Karar 6 narrows `NEN-024`'s "no settings screen before a later
/// milestone" ban to exactly this: **only** the two preferred-language
/// pickers live here. Anything else belongs to its own task and its own
/// widening of that decision.
public struct SubtitlePreferencesSettingsView: View {
    @ObservedObject private var model: PlayerModel

    public init(model: PlayerModel) {
        self.model = model
    }

    public var body: some View {
        Form {
            picker("Birinci tercih edilen dil", selection: primaryBinding)
            picker("İkinci tercih edilen dil", selection: secondaryBinding)
        }
        .padding()
        .frame(width: 360)
    }

    private func picker(_ title: String, selection: Binding<String?>) -> some View {
        Picker(title, selection: selection) {
            Text("Yok").tag(String?.none)
            ForEach(LanguageCatalog.allCodes, id: \.self) { code in
                Text(SubtitleMenuPresentation.endonym(for: code)).tag(String?.some(code))
            }
        }
    }

    private var primaryBinding: Binding<String?> {
        Binding(
            get: { model.subtitlePreferences.primary },
            set: { newValue in
                model.updateSubtitlePreferences(
                    SubtitleLanguagePreferences(
                        primary: newValue,
                        secondary: model.subtitlePreferences.secondary
                    )
                )
            }
        )
    }

    private var secondaryBinding: Binding<String?> {
        Binding(
            get: { model.subtitlePreferences.secondary },
            set: { newValue in
                model.updateSubtitlePreferences(
                    SubtitleLanguagePreferences(
                        primary: model.subtitlePreferences.primary,
                        secondary: newValue
                    )
                )
            }
        )
    }
}
