import Foundation
import NenCore
import Testing

@testable import NenPlayerShell

@Suite("macOS credential settings")
@MainActor
struct CredentialSettingsTests {
    @Test("save marks a key registered, delete clears it, and empty input is ignored")
    func credentialLifecycleUsesOnlyPresenceOnTheUiPath() {
        let foreign = FakeCredentialStore()
        let ffiStore = FfiSecureCredentialStore(store: foreign)
        let settings = CredentialSettingsModel(store: ffiStore)

        #expect(settings.save(" \t ", for: .openAi) == .ignoredEmpty)
        #expect(foreign.setCalls == 0)
        #expect(!settings.isStored(.openAi))

        for provider in CredentialProvider.allCases {
            let value = "fixture-\(provider.id)"
            #expect(settings.save(value, for: provider) == .saved)
            #expect(settings.isStored(provider))
            #expect(foreign.storedValue(for: provider.kind) == value)
        }
        #expect(foreign.setCalls == CredentialProvider.allCases.count)
        #expect(foreign.getCalls == 0, "the settings path uses contains, never get")

        // A new model is the relaunch path. It restores only the presence
        // indicator, not the value.
        let relaunched = CredentialSettingsModel(store: ffiStore)
        for provider in CredentialProvider.allCases {
            #expect(relaunched.isStored(provider))
            #expect(relaunched.delete(provider) == .deleted)
            #expect(!relaunched.isStored(provider))
        }
        #expect(foreign.deleteCalls == CredentialProvider.allCases.count)
        #expect(foreign.getCalls == 0)
    }

    @Test("control characters and overlong values use the typed invalid error")
    func invalidValuesArePresentedWithoutPayload() {
        let foreign = FakeCredentialStore()
        let settings = CredentialSettingsModel(
            store: FfiSecureCredentialStore(store: foreign)
        )

        #expect(
            settings.save("fixture\nkey", for: .openRouter)
                == .failed(.invalid)
        )
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .invalid))
        #expect(!settings.isStored(.openRouter))

        settings.dismissError()
        #expect(
            settings.save(String(repeating: "x", count: 513), for: .openRouter)
                == .failed(.invalid)
        )
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .invalid))
        #expect(foreign.setCalls == 0)
    }

    @Test("all typed store failures have closed Turkish messages")
    func typedFailuresStayPayloadFree() {
        let foreign = FakeCredentialStore()
        let settings = CredentialSettingsModel(
            store: FfiSecureCredentialStore(store: foreign)
        )

        foreign.setError = .Unavailable
        #expect(settings.save("fixture-unavailable", for: .openSubtitles) == .failed(.unavailable))
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .unavailable))

        foreign.setError = .Denied
        #expect(settings.save("fixture-denied", for: .openAi) == .failed(.denied))
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .denied))

        foreign.setError = .Corrupt
        #expect(settings.save("fixture-corrupt", for: .openRouter) == .failed(.corrupt))
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .corrupt))

        foreign.setError = nil
        foreign.containsError = .Corrupt
        let refreshed = CredentialSettingsModel(store: FfiSecureCredentialStore(store: foreign))
        #expect(refreshed.errorMessage == CredentialSettingsModel.message(for: .corrupt))

        foreign.containsError = nil
        foreign.deleteError = .Denied
        #expect(settings.delete(.openSubtitles) == .failed(.denied))
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .denied))

        let messages = [
            CredentialSettingsModel.message(for: .invalid),
            CredentialSettingsModel.message(for: .unavailable),
            CredentialSettingsModel.message(for: .denied),
            CredentialSettingsModel.message(for: .corrupt)
        ]
        #expect(!messages.contains { $0.contains("fixture") })
    }

    @Test("missing production composition is visible and disables the rows")
    func missingCompositionIsNotReportedAsNotStored() {
        let settings = CredentialSettingsModel(store: nil)

        #expect(!settings.isAvailable)
        for provider in CredentialProvider.allCases {
            #expect(settings.statusLabel(for: provider) == "Erişilemiyor")
            #expect(settings.actionLabel(for: provider) == "Kullanılamıyor")
        }
        #expect(settings.save("fixture-missing-store", for: .openAi) == .failed(.unavailable))
        #expect(settings.errorMessage == CredentialSettingsModel.message(for: .unavailable))
    }

    @Test("each provider has a distinct non-secret accessibility surface")
    func providerAccessibilityIdentifiersAreClosed() {
        let providers = CredentialProvider.allCases
        #expect(Set(providers.map(\.fieldAccessibilityIdentifier)).count == 3)
        #expect(Set(providers.map(\.statusAccessibilityIdentifier)).count == 3)
        #expect(Set(providers.map(\.saveAccessibilityIdentifier)).count == 3)
        #expect(Set(providers.map(\.deleteAccessibilityIdentifier)).count == 3)
        #expect(providers.allSatisfy { $0.fieldAccessibilityIdentifier.hasPrefix("credential.") })
    }

    @Test("the sentinel is absent from plaintext persistence and presentation strings")
    func secretDoesNotEnterDefaultsApplicationSupportOrViewText() throws {
        let sentinel = "fixture-settings-sentinel"

        let foreign = FakeCredentialStore()
        let settings = CredentialSettingsModel(
            store: FfiSecureCredentialStore(store: foreign)
        )
        #expect(settings.save(sentinel, for: .openSubtitles) == .saved)

        #expect(
            !UserDefaults.standard.dictionaryRepresentation().values
                .contains { String(describing: $0).contains(sentinel) }
        )
        let preferencesFile = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Preferences/player.nen.macos.plist")
        #expect(!contains(sentinel, inFile: preferencesFile))
        #expect(!contains(sentinel, under: applicationSupportRoot()))
        #expect(!settings.presentationTexts.contains { $0.contains(sentinel) })
    }

    private func applicationSupportRoot() -> URL? {
        FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask)
            .first?
            .appendingPathComponent("NenPlayer", isDirectory: true)
    }

    private func contains(_ value: String, under root: URL?) -> Bool {
        guard let root, FileManager.default.fileExists(atPath: root.path),
              let enumerator = FileManager.default.enumerator(
                  at: root,
                  includingPropertiesForKeys: [.isRegularFileKey],
                  options: [.skipsHiddenFiles]
              )
        else { return false }

        let needle = Data(value.utf8)
        for case let file as URL in enumerator {
            guard (try? file.resourceValues(forKeys: [.isRegularFileKey]).isRegularFile) == true,
                  let data = try? Data(contentsOf: file)
            else { continue }
            if data.range(of: needle) != nil { return true }
        }
        return false
    }

    private func contains(_ value: String, inFile file: URL) -> Bool {
        guard let data = try? Data(contentsOf: file) else { return false }
        return data.range(of: Data(value.utf8)) != nil
    }
}

private final class FakeCredentialStore: ForeignSecureCredentialStore, @unchecked Sendable {
    private var values: [FfiCredentialKind: String] = [:]
    var getCalls = 0
    var setCalls = 0
    var deleteCalls = 0
    var containsError: FfiCredentialError?
    var setError: FfiCredentialError?
    var deleteError: FfiCredentialError?

    func get(kind: FfiCredentialKind) throws -> String? {
        getCalls += 1
        return values[kind]
    }

    func contains(kind: FfiCredentialKind) throws -> Bool {
        if let containsError { throw containsError }
        return values[kind] != nil
    }

    func set(kind: FfiCredentialKind, value: String) throws {
        if let setError { throw setError }
        setCalls += 1
        values[kind] = value
    }

    func delete(kind: FfiCredentialKind) throws {
        if let deleteError { throw deleteError }
        deleteCalls += 1
        values[kind] = nil
    }

    func storedValue(for kind: FfiCredentialKind) -> String? {
        values[kind]
    }
}
