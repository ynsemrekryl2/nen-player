import CryptoKit
import Foundation

/// The one thing that reaches the screen. Never the path (ADR-0031 Karar 2).
public struct RecentMediaEntry: Identifiable, Equatable, Sendable {
    public let id: UUID
    public let displayName: String

    public init(id: UUID, displayName: String) {
        self.id = id
        self.displayName = displayName
    }
}

public protocol RecentMediaStoring: AnyObject {
    var entries: [RecentMediaEntry] { get }
    func save(_ url: URL) throws
    func resolve(_ id: RecentMediaEntry.ID) throws -> URL?
    func remove(_ id: RecentMediaEntry.ID)
    func clear()
}

/// Vends and resolves security-scoped bookmarks.
///
/// A seam rather than calling `URL` directly: `bookmarkDataIsStale` cannot be
/// forced true from outside Foundation, so the stale-refresh paths in
/// `UserDefaultsRecentMediaStore` are tested against a fake conforming to
/// this protocol instead.
protocol BookmarkVending {
    func makeBookmark(for url: URL) throws -> Data
    /// Returns the resolved URL and whether the bookmark was stale.
    func resolveBookmark(_ data: Data) throws -> (url: URL, stale: Bool)
}

struct FoundationBookmarkVending: BookmarkVending {
    func makeBookmark(for url: URL) throws -> Data {
        try url.bookmarkData(
            options: [.withSecurityScope, .securityScopeAllowOnlyReadAccess],
            includingResourceValuesForKeys: nil,
            relativeTo: nil
        )
    }

    func resolveBookmark(_ data: Data) throws -> (url: URL, stale: Bool) {
        var stale = false
        let url = try URL(
            resolvingBookmarkData: data,
            options: .withSecurityScope,
            relativeTo: nil,
            bookmarkDataIsStale: &stale
        )
        return (url, stale)
    }
}

/// Persists up to `capacity` security-scoped bookmarks, most recent first.
///
/// A full history, not a single slot — the limitation `RecentMediaStoring`'s
/// earlier, one-entry shape carried (NEN-042). Only local files are
/// remembered: a remote (http/https) locator can carry a token in its query
/// and is never written to history (NEN-042 → YAPILMAYACAK).
public final class UserDefaultsRecentMediaStore: RecentMediaStoring {
    private enum Key {
        static let entries = "recentMediaEntries"
        static let migrated = "recentMediaMigrated"
        // The single-slot keys this store replaces (pre-NEN-042).
        static let legacyBookmark = "recentMediaBookmark"
        static let legacyDisplayName = "recentMediaDisplayName"
    }

    /// What is actually stored per entry. `key` is a digest of the
    /// standardized path — used only to deduplicate re-opened files, never
    /// shown or logged (K23 #8). The path itself is not retained; the
    /// bookmark is the sole capability that can recover it.
    private struct Record: Codable {
        var id: UUID
        var displayName: String
        var bookmark: Data
        var key: String
    }

    private let defaults: UserDefaults
    private let capacity: Int
    private let bookmarks: BookmarkVending

    public convenience init(
        defaults: UserDefaults = .standard,
        capacity: Int = 5
    ) {
        self.init(defaults: defaults, capacity: capacity, bookmarks: FoundationBookmarkVending())
    }

    init(
        defaults: UserDefaults = .standard,
        capacity: Int = 5,
        bookmarks: BookmarkVending
    ) {
        self.defaults = defaults
        self.capacity = capacity
        self.bookmarks = bookmarks
        migrateLegacyEntryIfNeeded()
    }

    public var entries: [RecentMediaEntry] {
        records.map { RecentMediaEntry(id: $0.id, displayName: $0.displayName) }
    }

    public func save(_ url: URL) throws {
        // Remote media is never written to history — its locator can carry a
        // token in its query string (NEN-042 → YAPILMAYACAK).
        guard url.isFileURL else { return }
        let bookmark = try bookmarks.makeBookmark(for: url)
        let key = Self.pathKey(for: url)
        var updated = records.filter { $0.key != key }
        updated.insert(
            Record(id: UUID(), displayName: url.lastPathComponent, bookmark: bookmark, key: key),
            at: 0
        )
        if updated.count > capacity {
            updated.removeLast(updated.count - capacity)
        }
        records = updated
    }

    public func resolve(_ id: RecentMediaEntry.ID) throws -> URL? {
        var current = records
        guard let index = current.firstIndex(where: { $0.id == id }) else { return nil }
        let (url, stale) = try bookmarks.resolveBookmark(current[index].bookmark)
        if stale {
            // The refresh needs the security scope open, and its own failure
            // must not cost the caller a URL that already resolved — the
            // resolved-to-nothing case is the only one that drops the entry
            // (NEN-042, closing the defect NEN-050 deferred).
            let scoped = url.startAccessingSecurityScopedResource()
            defer { if scoped { url.stopAccessingSecurityScopedResource() } }
            if let refreshed = try? bookmarks.makeBookmark(for: url) {
                current[index].bookmark = refreshed
                records = current
            }
        }
        return url
    }

    public func remove(_ id: RecentMediaEntry.ID) {
        records.removeAll { $0.id == id }
    }

    public func clear() {
        defaults.removeObject(forKey: Key.entries)
    }

    private var records: [Record] {
        get {
            guard let data = defaults.data(forKey: Key.entries) else { return [] }
            return (try? PropertyListDecoder().decode([Record].self, from: data)) ?? []
        }
        set {
            let data = try? PropertyListEncoder().encode(newValue)
            defaults.set(data, forKey: Key.entries)
        }
    }

    /// A path's dedup key — never surfaced, never logged (K23 #8).
    private static func pathKey(for url: URL) -> String {
        let digest = SHA256.hash(data: Data(url.standardizedFileURL.path.utf8))
        return digest.map { String(format: "%02x", $0) }.joined()
    }

    /// Moves the single bookmark this store used to keep into the list, once.
    /// Run before `entries`/`save`/... can be reached from `init`, on the same
    /// terms as `UserDefaultsSubtitlePreferenceStore`'s one-time seed.
    private func migrateLegacyEntryIfNeeded() {
        guard !defaults.bool(forKey: Key.migrated) else { return }
        defaults.set(true, forKey: Key.migrated)
        defer {
            defaults.removeObject(forKey: Key.legacyBookmark)
            defaults.removeObject(forKey: Key.legacyDisplayName)
        }
        guard let bookmark = defaults.data(forKey: Key.legacyBookmark),
              let displayName = defaults.string(forKey: Key.legacyDisplayName),
              let (url, _) = try? bookmarks.resolveBookmark(bookmark)
        else { return }
        records = [
            Record(id: UUID(), displayName: displayName, bookmark: bookmark, key: Self.pathKey(for: url))
        ]
    }
}
