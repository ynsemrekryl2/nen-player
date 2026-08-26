import Foundation

public protocol RecentMediaStoring: AnyObject {
    var displayName: String? { get }
    func save(_ url: URL) throws
    func resolve() throws -> URL?
    func clear()
}

/// Persists one security-scoped bookmark. A full history belongs to NEN-042.
public final class UserDefaultsRecentMediaStore: RecentMediaStoring {
    private enum Key {
        static let bookmark = "recentMediaBookmark"
        static let displayName = "recentMediaDisplayName"
    }

    private let defaults: UserDefaults

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    public var displayName: String? {
        defaults.string(forKey: Key.displayName)
    }

    public func save(_ url: URL) throws {
        let bookmark = try url.bookmarkData(
            options: [.withSecurityScope, .securityScopeAllowOnlyReadAccess],
            includingResourceValuesForKeys: nil,
            relativeTo: nil
        )
        defaults.set(bookmark, forKey: Key.bookmark)
        // Only the basename is kept for presentation. The bookmark itself is
        // opaque capability data and is never printed or surfaced.
        defaults.set(url.lastPathComponent, forKey: Key.displayName)
    }

    public func resolve() throws -> URL? {
        guard let bookmark = defaults.data(forKey: Key.bookmark) else {
            return nil
        }
        var stale = false
        let url = try URL(
            resolvingBookmarkData: bookmark,
            options: .withSecurityScope,
            relativeTo: nil,
            bookmarkDataIsStale: &stale
        )
        if stale {
            try save(url)
        }
        return url
    }

    public func clear() {
        defaults.removeObject(forKey: Key.bookmark)
        defaults.removeObject(forKey: Key.displayName)
    }
}
