import Foundation
import NenCore
import Security

/// The macOS login-keychain adapter for the credential port (ADR-0020).
///
/// The Rust side owns validation and the only UI-facing store object. This
/// adapter deliberately exposes only the reverse-FFI protocol: it never logs,
/// serializes, or otherwise describes a credential value.
public final class KeychainCredentialStore: ForeignSecureCredentialStore, @unchecked Sendable {
    /// The fixed service used by the production application bundle.
    public static let productionService = "player.nen.macos"

    /// The isolated service used by this target's real-Keychain tests.
    ///
    /// It is internal so production callers cannot accidentally select the
    /// test namespace; `@testable import` uses it to construct the test-only
    /// adapter.
    static let testService = "player.nen.macos.test"

    private let service: String

    public init() {
        service = Self.productionService
    }

    /// Test-only injection for the separate Keychain service required by
    /// ADR-0020. The application uses `init()` and cannot choose a service.
    init(service: String) {
        self.service = service
    }

    public func get(kind: FfiCredentialKind) throws -> String? {
        var result: CFTypeRef?
        let status = SecItemCopyMatching(
            query(for: kind, returningData: true) as CFDictionary,
            &result
        )

        switch status {
        case errSecSuccess:
            guard let data = result as? Data,
                  let value = String(data: data, encoding: .utf8)
            else {
                throw FfiCredentialError.Corrupt
            }
            return value
        case errSecItemNotFound:
            return nil
        default:
            throw Self.error(for: status)
        }
    }

    public func contains(kind: FfiCredentialKind) throws -> Bool {
        var result: CFTypeRef?
        let status = SecItemCopyMatching(
            query(for: kind, returningData: false) as CFDictionary,
            &result
        )

        switch status {
        case errSecSuccess:
            return true
        case errSecItemNotFound:
            return false
        default:
            throw Self.error(for: status)
        }
    }

    public func set(kind: FfiCredentialKind, value: String) throws {
        let updateAttributes: [String: Any] = [
            kSecValueData as String: Data(value.utf8)
        ]
        var status = SecItemUpdate(
            query(for: kind, returningData: false) as CFDictionary,
            updateAttributes as CFDictionary
        )

        if status == errSecItemNotFound {
            var addQuery = query(for: kind, returningData: false)
            addQuery[kSecValueData as String] = Data(value.utf8)
            addQuery[kSecAttrAccessible as String] =
                kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
            status = SecItemAdd(addQuery as CFDictionary, nil)

            // A second writer may have inserted the same account between our
            // update and add. Updating once more preserves set's overwrite
            // semantics without broadening the query beyond this record.
            if status == errSecDuplicateItem {
                status = SecItemUpdate(
                    query(for: kind, returningData: false) as CFDictionary,
                    updateAttributes as CFDictionary
                )
            }
        }

        guard status == errSecSuccess else {
            throw Self.error(for: status)
        }
    }

    public func delete(kind: FfiCredentialKind) throws {
        let status = SecItemDelete(query(for: kind, returningData: false) as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw Self.error(for: status)
        }
    }

    /// Builds a fully qualified generic-password query. Every mutation is
    /// scoped by the app's fixed service and the closed kind-to-account map.
    private func query(
        for kind: FfiCredentialKind,
        returningData: Bool
    ) -> [String: Any] {
        var query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: kind.account
        ]
        if returningData {
            query[kSecReturnData as String] = true
            query[kSecMatchLimit as String] = kSecMatchLimitOne
        }
        return query
    }

    /// Maps OSStatus without allowing a status code or any platform payload
    /// to cross the FFI boundary. Not-found is handled by each operation
    /// because get/contains/delete intentionally have different semantics.
    static func error(for status: OSStatus) -> FfiCredentialError {
        switch status {
        case errSecAuthFailed, errSecInteractionNotAllowed, errSecUserCanceled:
            return .Denied
        default:
            return .Unavailable
        }
    }
}

private extension FfiCredentialKind {
    var account: String {
        switch self {
        case .openSubtitles:
            return "opensubtitles"
        case .openAi:
            return "openai"
        case .openRouter:
            return "openrouter"
        }
    }
}

/// The single production composition hook from the macOS app to the Rust
/// credential object. Settings and providers receive this FFI object later;
/// they never construct or call the Keychain adapter themselves.
public enum CredentialCompositionRoot {
    public static func makeProductionStore() -> FfiSecureCredentialStore {
        FfiSecureCredentialStore(store: KeychainCredentialStore())
    }
}
