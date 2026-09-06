import Testing

@testable import NenPlaybackMPV

@Suite("Render timing policy")
struct RenderTimingPolicyTests {
    @Test("normal drawing waits for the target frame time")
    func normalDrawingBlocksForTargetTime() {
        #expect(MPVRenderTimingPolicy.shouldBlockForTargetTime(inLiveResize: false))
    }

    @Test("live resize does not wait for the target frame time")
    func liveResizeDoesNotBlockForTargetTime() {
        #expect(!MPVRenderTimingPolicy.shouldBlockForTargetTime(inLiveResize: true))
    }
}
