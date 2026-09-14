import Foundation
import NenCore

/// The single presentation contract for an exact provider identity.
///
/// The basename remains the provisional title until the verified result
/// arrives. Only the bounded title/year/episode fields cross this boundary;
/// provider ids, hashes and URLs have no presentation path.
public enum VerifiedMediaIdentityPresentation {
    /// Matches the existing chrome ease-out transition (`PlayerRootView`).
    public static let transitionDuration: TimeInterval = 0.24

    public static func label(for identity: FfiVerifiedMediaIdentity) -> String {
        var label = identity.title
        if let year = identity.year {
            label += " (\(year))"
        }

        let season = identity.season.map { String(format: "S%02d", Int($0)) } ?? ""
        let episode = identity.episode.map { String(format: "E%02d", Int($0)) } ?? ""
        let coordinates = season + episode
        if !coordinates.isEmpty {
            label += " · \(coordinates)"
        }
        return label
    }

    /// Returns the verified compact label when available, otherwise the
    /// current basename. `nil` means there is no media to show.
    public static func mediaTitle(
        mediaName: String?,
        identity: FfiVerifiedMediaIdentity?
    ) -> String? {
        if let identity {
            return label(for: identity)
        }
        return mediaName
    }
}
