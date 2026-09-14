import SwiftUI

/// M3's one Settings surface, widened by `NEN-101` and `NEN-113`.
///
/// ADR-0031 Karar 6 narrows `NEN-024`'s "no settings screen before a later
/// milestone" ban to exactly this window: no *general* settings screen, only
/// named preference pickers. `NEN-101`'s target-language picker is the third
/// row under that same narrowing (ADR-0031 Notlar, 2026-09-10). `NEN-113` adds
/// only the three named credential rows; this is still not a general settings
/// screen.
@MainActor
public struct SubtitlePreferencesSettingsView: View {
    @ObservedObject private var model: PlayerModel
    @StateObject private var credentials: CredentialSettingsModel
    @State private var openSubtitlesDraft = ""
    @State private var openAiDraft = ""
    @State private var openRouterDraft = ""

    public init(model: PlayerModel) {
        self.model = model
        _credentials = StateObject(wrappedValue: CredentialSettingsModel(store: model.credentialStore))
    }

    public var body: some View {
        Form {
            picker("Birinci tercih edilen dil", selection: primaryBinding)
            picker("İkinci tercih edilen dil", selection: secondaryBinding)
            picker("AI çeviri hedef dili", selection: translationTargetBinding)

            Section("API anahtarları") {
                credentialRow(.openSubtitles, draft: $openSubtitlesDraft)
                credentialRow(.openAi, draft: $openAiDraft)
                credentialRow(.openRouter, draft: $openRouterDraft)
            }
        }
        .padding()
        .frame(width: 440)
        .alert(
            "API anahtarı",
            isPresented: Binding(
                get: { credentials.errorMessage != nil },
                set: { isPresented in
                    if !isPresented { credentials.dismissError() }
                }
            )
        ) {
            Button("Tamam", role: .cancel) { credentials.dismissError() }
        } message: {
            Text(credentials.errorMessage ?? "İşlem tamamlanamadı.")
        }
        .onDisappear {
            openSubtitlesDraft = ""
            openAiDraft = ""
            openRouterDraft = ""
            credentials.dismissError()
        }
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

    private var translationTargetBinding: Binding<String?> {
        Binding(
            get: { model.translationTargetLanguage },
            set: { model.updateTranslationTargetLanguage($0) }
        )
    }

    private func credentialRow(
        _ provider: CredentialProvider,
        draft: Binding<String>
    ) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text(provider.title)
                Spacer(minLength: 8)
                Label(
                    credentials.statusLabel(for: provider),
                    systemImage: credentials.isStored(provider) ? "checkmark.circle" : "circle"
                )
                .accessibilityIdentifier(provider.statusAccessibilityIdentifier)
                .foregroundStyle(.secondary)
            }

            SecureField(provider.fieldPlaceholder, text: draft)
                .textFieldStyle(.roundedBorder)
                .accessibilityIdentifier(provider.fieldAccessibilityIdentifier)
                .accessibilityLabel(provider.fieldPlaceholder)
                .disabled(!credentials.isAvailable)
                .onSubmit { save(provider, draft: draft) }

            HStack {
                Button(credentials.actionLabel(for: provider)) {
                    save(provider, draft: draft)
                }
                .accessibilityIdentifier(provider.saveAccessibilityIdentifier)
                .accessibilityLabel("\(provider.title) \(credentials.actionLabel(for: provider))")
                .disabled(!credentials.isAvailable || isBlank(draft.wrappedValue))

                Button("Sil", role: .destructive) {
                    _ = credentials.delete(provider)
                    draft.wrappedValue = ""
                }
                .accessibilityIdentifier(provider.deleteAccessibilityIdentifier)
                .accessibilityLabel("\(provider.title) Sil")
                .disabled(!credentials.isAvailable || !credentials.isStored(provider))
            }
        }
        .padding(.vertical, 4)
    }

    private func save(_ provider: CredentialProvider, draft: Binding<String>) {
        _ = credentials.save(draft.wrappedValue, for: provider)
        // Clear every attempted value, including a rejected one. The user
        // must re-enter after an error, which bounds the secret's lifetime in
        // SwiftUI state and avoids leaving a failed credential in memory.
        draft.wrappedValue = ""
    }

    private func isBlank(_ value: String) -> Bool {
        value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
    }
}
