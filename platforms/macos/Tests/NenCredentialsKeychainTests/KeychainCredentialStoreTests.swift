import Foundation
import NenCore
import Security
import Testing

@testable import NenCredentialsKeychain

/// These tests intentionally use the real login Keychain. The service is
/// isolated from the production bundle and every record created by the suite
/// is removed before the all-services count is compared again.
@Suite("macOS Keychain credential adapter", .serialized)
struct KeychainCredentialStoreTests {
    private static let kinds: [FfiCredentialKind] = [
        .openSubtitles,
        .openAi,
        .openRouter
    ]
    private static let values = [
        "contract-opensubtitles",
        "contract-openai",
        "contract-openrouter"
    ]

    @Test("real Keychain satisfies the credential contract and stays isolated")
    func realKeychainSatisfiesContract() throws {
        let store = KeychainCredentialStore(service: KeychainCredentialStore.testService)
        try KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService)
        defer { try? KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService) }

        let before = try KeychainTestSupport.countAllGenericPasswords()

        for kind in Self.kinds {
            #expect(try store.get(kind: kind) == nil)
            #expect(try !store.contains(kind: kind))
        }

        for (kind, value) in zip(Self.kinds, Self.values) {
            try store.set(kind: kind, value: value)
            #expect(try store.get(kind: kind) == value)
            #expect(try store.contains(kind: kind))
        }

        // A kind can only see its own account, not a neighboring provider's.
        for (index, kind) in Self.kinds.enumerated() {
            #expect(try store.get(kind: kind) == Self.values[index])
        }

        for kind in Self.kinds {
            try store.delete(kind: kind)
            #expect(try store.get(kind: kind) == nil)
            #expect(try !store.contains(kind: kind))
        }

        // Deleting a missing record is idempotent and must not touch another
        // service. The count is taken only after the test namespace is empty.
        try store.delete(kind: .openAi)
        let after = try KeychainTestSupport.countAllGenericPasswords()
        #expect(before == after)
    }

    @Test("the Rust-facing object writes through the same real adapter")
    func theFfiObjectUsesTheRealAdapter() throws {
        let store = KeychainCredentialStore(service: KeychainCredentialStore.testService)
        try KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService)
        defer { try? KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService) }

        let ffiStore = FfiSecureCredentialStore(store: store)
        try ffiStore.set(kind: .openAi, value: "  reverse-ffi-contract  ")
        #expect(try ffiStore.contains(kind: .openAi))
        #expect(try store.get(kind: .openAi) == "reverse-ffi-contract")
        try ffiStore.delete(kind: .openAi)
        #expect(try store.get(kind: .openAi) == nil)
    }

    @Test("Keychain status errors stay typed and payload-free")
    func keychainStatusErrorsAreTyped() {
        #expect(KeychainCredentialStore.error(for: errSecAuthFailed) == .Denied)
        #expect(KeychainCredentialStore.error(for: errSecInteractionNotAllowed) == .Denied)
        #expect(KeychainCredentialStore.error(for: errSecUserCanceled) == .Denied)
        #expect(KeychainCredentialStore.error(for: errSecParam) == .Unavailable)

        let printed = [
            FfiCredentialError.Invalid,
            FfiCredentialError.Unavailable,
            FfiCredentialError.Denied,
            FfiCredentialError.Corrupt
        ].map { String(reflecting: $0) }.joined(separator: " ")
        #expect(!printed.contains("contract-opensubtitles"))
        #expect(!printed.contains("reverse-ffi-contract"))
    }

    @Test("not found is absence and malformed stored bytes are corrupt")
    func missingAndMalformedRecordsUseClosedErrors() throws {
        let store = KeychainCredentialStore(service: KeychainCredentialStore.testService)
        try KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService)
        defer { try? KeychainTestSupport.deleteAllRecords(service: KeychainCredentialStore.testService) }

        #expect(try store.get(kind: .openRouter) == nil)
        #expect(try !store.contains(kind: .openRouter))

        try KeychainTestSupport.addRawRecord(
            service: KeychainCredentialStore.testService,
            account: "openrouter",
            data: Data([0xff])
        )
        #expect(throws: FfiCredentialError.Corrupt) {
            _ = try store.get(kind: .openRouter)
        }
    }
}

private enum KeychainTestSupport {
    enum Failure: Error {
        case query
        case cleanup
    }

    static func countAllGenericPasswords() throws -> Int {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecMatchLimit as String: kSecMatchLimitAll,
            kSecReturnAttributes as String: true
        ]
        var result: CFTypeRef?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        switch status {
        case errSecItemNotFound:
            return 0
        case errSecSuccess:
            if let records = result as? [[String: Any]] {
                return records.count
            }
            if let records = result as? [Any] {
                return records.count
            }
            return result == nil ? 0 : 1
        default:
            throw Failure.query
        }
    }

    static func deleteAllRecords(service: String) throws {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service
        ]
        let status = SecItemDelete(query as CFDictionary)
        guard status == errSecSuccess || status == errSecItemNotFound else {
            throw Failure.cleanup
        }
    }

    static func addRawRecord(service: String, account: String, data: Data) throws {
        let query: [String: Any] = [
            kSecClass as String: kSecClassGenericPassword,
            kSecAttrService as String: service,
            kSecAttrAccount as String: account,
            kSecValueData as String: data,
            kSecAttrAccessible as String: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
        ]
        let status = SecItemAdd(query as CFDictionary, nil)
        guard status == errSecSuccess else { throw Failure.cleanup }
    }
}
