import Foundation
import Testing
@testable import NenPlayerShell

@Suite("recent media bookmark")
struct RecentMediaStoreTests {
    @Test("a security-scoped bookmark survives a fresh store instance")
    func bookmarkRoundTrip() throws {
        let suiteName = "player.nen.tests.recent-media.\(UUID().uuidString)"
        let defaults = try #require(UserDefaults(suiteName: suiteName))
        defer { defaults.removePersistentDomain(forName: suiteName) }
        let fixture = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("fixtures/media/contract-clip.mkv")
            .standardizedFileURL

        let writer = UserDefaultsRecentMediaStore(defaults: defaults)
        try writer.save(fixture)
        let reader = UserDefaultsRecentMediaStore(defaults: defaults)
        let resolvedURL = try reader.resolve()
        let resolved = try #require(resolvedURL)

        #expect(reader.displayName == "contract-clip.mkv")
        #expect(resolved.standardizedFileURL == fixture)
    }
}
