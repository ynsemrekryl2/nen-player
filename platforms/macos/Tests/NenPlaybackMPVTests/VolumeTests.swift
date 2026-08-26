import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-054's evidence for the half of `setVolume` a test can reach.
///
/// The device path (`ao-volume`) needs a live audio output, and these engines
/// are headless with `ao=null`, so what runs here is the fallback. That is the
/// half worth pinning down automatically: it must hold the user's level while
/// no output exists, because the level set there is what the user keeps
/// hearing until the device takes over.
struct VolumeTests {
    @Test func withoutAnAudioOutputTheFilterChainHoldsTheLevel() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        // The precondition this test rests on: no audio output, no `ao-volume`.
        #expect(throws: FfiPlaybackError.self) { try engine.double("ao-volume") }

        try engine.setVolume(volume: 0.5)
        #expect(try engine.double("volume") == 50)

        try engine.setVolume(volume: 0)
        #expect(try engine.double("volume") == 0)

        try engine.setVolume(volume: 1)
        #expect(try engine.double("volume") == 100)
    }
}
