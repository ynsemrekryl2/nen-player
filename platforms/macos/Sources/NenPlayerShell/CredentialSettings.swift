import Combine
import Foundation
import NenCore

/// The three providers whose credentials are part of M6's macOS settings
/// surface. The labels are the only provider-specific text the shell owns;
/// the secret itself never becomes a property of this type.
public enum CredentialProvider: CaseIterable, Hashable, Identifiable {
    case openSubtitles
    case openAi
    case openRouter

    public var id: String {
        switch self {
        case .openSubtitles: "opensubtitles"
        case .openAi: "openai"
        case .openRouter: "openrouter"
        }
    }

    public var title: String {
        switch self {
        case .openSubtitles: "OpenSubtitles API anahtarı"
        case .openAi: "OpenAI API anahtarı"
        case .openRouter: "OpenRouter API anahtarı"
        }
    }

    public var fieldPlaceholder: String {
        switch self {
        case .openSubtitles: "Yeni OpenSubtitles API anahtarı"
        case .openAi: "Yeni OpenAI API anahtarı"
        case .openRouter: "Yeni OpenRouter API anahtarı"
        }
    }

    public var fieldAccessibilityIdentifier: String {
        "credential.\(id).field"
    }

    public var statusAccessibilityIdentifier: String {
        "credential.\(id).status"
    }

    public var saveAccessibilityIdentifier: String {
        "credential.\(id).save"
    }

    public var deleteAccessibilityIdentifier: String {
        "credential.\(id).delete"
    }

    var kind: FfiCredentialKind {
        switch self {
        case .openSubtitles: .openSubtitles
        case .openAi: .openAi
        case .openRouter: .openRouter
        }
    }
}

/// A credential action's closed result. The settings view only needs to know
/// whether it should clear its short-lived input; it never receives a stored
/// value back from the FFI object.
public enum CredentialSettingsAction: Equatable {
    case saved
    case ignoredEmpty
    case deleted
    case failed(CredentialSettingsFailure)
}

public enum CredentialSettingsFailure: Equatable {
    case invalid
    case unavailable
    case denied
    case corrupt
}

/// View model for the three API-key rows.
///
/// `FfiSecureCredentialStore` intentionally exposes only `contains`, `set` and
/// `delete` to the UI. A fresh instance therefore restores the "Kayıtlı"
/// indicators after relaunch without ever reading a secret into SwiftUI
/// state. Draft text belongs to the view and is cleared after a successful
/// save or delete; this model never retains it.
@MainActor
public final class CredentialSettingsModel: ObservableObject {
    @Published public private(set) var isAvailable = true
    @Published public private(set) var openSubtitlesStored = false
    @Published public private(set) var openAiStored = false
    @Published public private(set) var openRouterStored = false
    @Published public private(set) var errorMessage: String?

    private let store: FfiSecureCredentialStore?

    public init(store: FfiSecureCredentialStore?) {
        self.store = store
        isAvailable = store != nil
        refresh()
    }

    public func isStored(_ provider: CredentialProvider) -> Bool {
        switch provider {
        case .openSubtitles: openSubtitlesStored
        case .openAi: openAiStored
        case .openRouter: openRouterStored
        }
    }

    public func actionLabel(for provider: CredentialProvider) -> String {
        guard isAvailable else { return "Kullanılamıyor" }
        return isStored(provider) ? "Değiştir" : "Kaydet"
    }

    public func statusLabel(for provider: CredentialProvider) -> String {
        guard isAvailable else { return "Erişilemiyor" }
        return isStored(provider) ? "Kayıtlı" : "Kayıtlı değil"
    }

    /// The complete set of user-visible strings owned by this view model.
    ///
    /// This is deliberately a projection of labels and state only. It gives
    /// the security tests one place to assert that a credential value can
    /// never reappear in a view-model presentation string.
    public var presentationTexts: [String] {
        CredentialProvider.allCases.flatMap { provider in
            [
                provider.title,
                statusLabel(for: provider),
                actionLabel(for: provider),
                "Sil"
            ]
        }
    }

    /// Reads presence only. The FFI object has no UI-facing getter for a
    /// stored value, so relaunch recovery cannot populate a secret field.
    public func refresh() {
        errorMessage = nil
        guard store != nil else {
            isAvailable = false
            for provider in CredentialProvider.allCases {
                setStored(false, for: provider)
            }
            return
        }
        isAvailable = true
        for provider in CredentialProvider.allCases {
            do {
                setStored(try store?.contains(kind: provider.kind) ?? false, for: provider)
            } catch let error as FfiCredentialError {
                setStored(false, for: provider)
                if errorMessage == nil {
                    errorMessage = Self.message(for: Self.failure(for: error))
                }
            } catch {
                setStored(false, for: provider)
                if errorMessage == nil {
                    errorMessage = Self.message(for: .unavailable)
                }
            }
        }
    }

    /// Stores a new value through the Rust-owned FFI object.
    ///
    /// Empty and whitespace-only input is a no-op by design. Non-empty input
    /// goes through the `ApiKey` newtype in Rust, so control characters and
    /// overlong values take the same typed `Invalid` path as every other
    /// caller.
    @discardableResult
    public func save(_ value: String, for provider: CredentialProvider) -> CredentialSettingsAction {
        guard !value.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty else {
            return .ignoredEmpty
        }
        guard let store else {
            return fail(.unavailable)
        }

        do {
            try store.set(kind: provider.kind, value: value)
            setStored(true, for: provider)
            errorMessage = nil
            return .saved
        } catch let error as FfiCredentialError {
            return fail(Self.failure(for: error))
        } catch {
            return fail(.unavailable)
        }
    }

    @discardableResult
    public func delete(_ provider: CredentialProvider) -> CredentialSettingsAction {
        guard let store else {
            return fail(.unavailable)
        }

        do {
            try store.delete(kind: provider.kind)
            setStored(false, for: provider)
            errorMessage = nil
            return .deleted
        } catch let error as FfiCredentialError {
            return fail(Self.failure(for: error))
        } catch {
            return fail(.unavailable)
        }
    }

    public func dismissError() {
        errorMessage = nil
    }

    public static func message(for failure: CredentialSettingsFailure) -> String {
        switch failure {
        case .invalid: "Bu API anahtarı geçersiz."
        case .unavailable: "API anahtarı kaydedilemedi."
        case .denied: "API anahtarına erişim reddedildi."
        case .corrupt: "Kayıtlı API anahtarı okunamadı."
        }
    }

    private func fail(_ failure: CredentialSettingsFailure) -> CredentialSettingsAction {
        errorMessage = Self.message(for: failure)
        return .failed(failure)
    }

    private func setStored(_ stored: Bool, for provider: CredentialProvider) {
        switch provider {
        case .openSubtitles: openSubtitlesStored = stored
        case .openAi: openAiStored = stored
        case .openRouter: openRouterStored = stored
        }
    }

    private static func failure(for error: FfiCredentialError) -> CredentialSettingsFailure {
        switch error {
        case .Invalid: .invalid
        case .Unavailable: .unavailable
        case .Denied: .denied
        case .Corrupt: .corrupt
        }
    }
}
