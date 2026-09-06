import SwiftUI

/// Exposes the active player scene's `PlayerModel` to the app-wide menu so
/// `PlayerCommands` can scope the seven playback shortcuts to the player
/// surface (NEN-047): the value is present only while the player window is
/// key, and `nil` while another scene (e.g. Settings) has focus.
private struct PlayerModelFocusedValueKey: FocusedValueKey {
    typealias Value = PlayerModel
}

extension FocusedValues {
    public var playerModel: PlayerModel? {
        get { self[PlayerModelFocusedValueKey.self] }
        set { self[PlayerModelFocusedValueKey.self] = newValue }
    }
}
