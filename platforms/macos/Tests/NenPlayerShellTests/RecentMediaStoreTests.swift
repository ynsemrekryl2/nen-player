import Foundation
import Testing
@testable import NenPlayerShell

/// A `BookmarkVending` a test can steer: forces `bookmarkDataIsStale` true
/// (unreachable from outside Foundation) and can make the refresh it
/// triggers fail, without touching the filesystem.
private final class FakeBookmarkVending: BookmarkVending {
    var stale = false
    var refreshFails = false
    private(set) var makeBookmarkCount = 0

    func makeBookmark(for url: URL) throws -> Data {
        makeBookmarkCount += 1
        // The first call is `save`'s own bookmark; a second call only
        // happens if `resolve` refreshes a stale one.
        if makeBookmarkCount > 1, refreshFails {
            throw CocoaError(.fileWriteNoPermission)
        }
        return Data(url.path.utf8)
    }

    func resolveBookmark(_ data: Data) throws -> (url: URL, stale: Bool) {
        (URL(fileURLWithPath: String(decoding: data, as: UTF8.self)), stale)
    }
}

@Suite("recent media list")
struct RecentMediaStoreTests {
    private func fixtureURL(_ name: String) -> URL {
        URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .deletingLastPathComponent()
            .appendingPathComponent("fixtures/media/\(name)")
            .standardizedFileURL
    }

    private func makeSuite() throws -> (defaults: UserDefaults, cleanup: () -> Void) {
        let suiteName = "player.nen.tests.recent-media.\(UUID().uuidString)"
        let defaults = try #require(UserDefaults(suiteName: suiteName))
        return (defaults, { defaults.removePersistentDomain(forName: suiteName) })
    }

    @Test("entries come back most recent first")
    func mostRecentFirst() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults)

        try store.save(fixtureURL("contract-clip.mkv"))
        try store.save(fixtureURL("menu-clip.mkv"))
        try store.save(fixtureURL("aspect-4x3-clip.mkv"))

        #expect(store.entries.map(\.displayName) == [
            "aspect-4x3-clip.mkv", "menu-clip.mkv", "contract-clip.mkv"
        ])
    }

    @Test("a saved bookmark round-trips through a fresh store instance")
    func bookmarkRoundTrip() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let fixture = fixtureURL("contract-clip.mkv")

        let writer = UserDefaultsRecentMediaStore(defaults: defaults)
        try writer.save(fixture)
        let reader = UserDefaultsRecentMediaStore(defaults: defaults)
        let entry = try #require(reader.entries.first)
        let resolved = try #require(try reader.resolve(entry.id))

        #expect(entry.displayName == "contract-clip.mkv")
        #expect(resolved.standardizedFileURL == fixture)
    }

    @Test("the oldest entry is dropped once capacity is exceeded")
    func capacityDropsOldest() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults, capacity: 5)
        let names = [
            "contract-clip.mkv", "menu-clip.mkv", "aspect-4x3-clip.mkv",
            "aspect-cinema-clip.mkv", "anamorphic-clip.mkv", "glass-dark-clip.mkv"
        ]

        for name in names {
            try store.save(fixtureURL(name))
        }

        #expect(store.entries.count == 5)
        #expect(store.entries.map(\.displayName) == [
            "glass-dark-clip.mkv", "anamorphic-clip.mkv", "aspect-cinema-clip.mkv",
            "aspect-4x3-clip.mkv", "menu-clip.mkv"
        ])
    }

    @Test("re-opening an already-listed file moves it to the front without duplicating it")
    func reopenDedupsToFront() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults)

        try store.save(fixtureURL("contract-clip.mkv"))
        try store.save(fixtureURL("menu-clip.mkv"))
        try store.save(fixtureURL("contract-clip.mkv"))

        #expect(store.entries.count == 2)
        #expect(store.entries.map(\.displayName) == ["contract-clip.mkv", "menu-clip.mkv"])
    }

    @Test("a stale bookmark is refreshed and the entry survives a failed refresh")
    func staleBookmarkRefreshFailureKeepsEntry() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let bookmarks = FakeBookmarkVending()
        bookmarks.stale = true
        bookmarks.refreshFails = true
        let store = UserDefaultsRecentMediaStore(defaults: defaults, capacity: 5, bookmarks: bookmarks)
        let fixture = fixtureURL("contract-clip.mkv")
        try store.save(fixture)
        let id = try #require(store.entries.first?.id)

        let resolved = try store.resolve(id)

        #expect(resolved == fixture)
        #expect(store.entries.count == 1)
    }

    @Test("a stale bookmark that refreshes successfully replaces the stored bookmark")
    func staleBookmarkRefreshSuccessReplacesBookmark() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let bookmarks = FakeBookmarkVending()
        bookmarks.stale = true
        let store = UserDefaultsRecentMediaStore(defaults: defaults, capacity: 5, bookmarks: bookmarks)
        try store.save(fixtureURL("contract-clip.mkv"))
        let id = try #require(store.entries.first?.id)

        _ = try store.resolve(id)

        // Two calls: the initial `save` and the refresh triggered by `stale`.
        #expect(bookmarks.makeBookmarkCount == 2)
    }

    @Test("a remote locator is never written to history")
    func remoteLocatorIsRejected() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults)

        try store.save(URL(string: "https://example.invalid/movie.mkv?token=secret")!)

        #expect(store.entries.isEmpty)
    }

    @Test("the single legacy bookmark migrates into the list exactly once")
    func legacyBookmarkMigratesOnce() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let fixture = fixtureURL("contract-clip.mkv")
        let legacyBookmark = try fixture.bookmarkData(
            options: [.withSecurityScope, .securityScopeAllowOnlyReadAccess],
            includingResourceValuesForKeys: nil,
            relativeTo: nil
        )
        defaults.set(legacyBookmark, forKey: "recentMediaBookmark")
        defaults.set("contract-clip.mkv", forKey: "recentMediaDisplayName")

        let migrated = UserDefaultsRecentMediaStore(defaults: defaults)
        #expect(migrated.entries.map(\.displayName) == ["contract-clip.mkv"])
        #expect(defaults.data(forKey: "recentMediaBookmark") == nil)
        #expect(defaults.string(forKey: "recentMediaDisplayName") == nil)

        // A file later saved through the migrated store must not be wiped
        // by a second migration attempt on the next launch.
        try migrated.save(fixtureURL("menu-clip.mkv"))
        let relaunched = UserDefaultsRecentMediaStore(defaults: defaults)
        #expect(relaunched.entries.map(\.displayName) == ["menu-clip.mkv", "contract-clip.mkv"])
    }

    @Test("remove drops only the named entry")
    func removeDropsOnlyNamedEntry() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults)
        try store.save(fixtureURL("contract-clip.mkv"))
        try store.save(fixtureURL("menu-clip.mkv"))
        let toRemove = try #require(store.entries.first(where: { $0.displayName == "contract-clip.mkv" })?.id)

        store.remove(toRemove)

        #expect(store.entries.map(\.displayName) == ["menu-clip.mkv"])
    }

    @Test("clear empties the list and removes the stored key")
    func clearEmptiesList() throws {
        let (defaults, cleanup) = try makeSuite()
        defer { cleanup() }
        let store = UserDefaultsRecentMediaStore(defaults: defaults)
        try store.save(fixtureURL("contract-clip.mkv"))

        store.clear()

        #expect(store.entries.isEmpty)
        #expect(defaults.object(forKey: "recentMediaEntries") == nil)
    }
}
