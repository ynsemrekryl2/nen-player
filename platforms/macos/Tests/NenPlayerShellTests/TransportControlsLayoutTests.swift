import AppKit
import NenCore
import NenPlaybackMPV
import SwiftUI
import Testing

@testable import NenPlayerShell

@Suite("Transport controls layout")
@MainActor
struct TransportControlsLayoutTests {
    @Test("edge controls stay inside short and screen-wide transports")
    func edgeControlsStayInsideTheTransport() throws {
        let cases: [(width: CGFloat, position: UInt64, duration: UInt64, remaining: Bool, title: String)] = [
            (693, 38_000, 90_000, false, "Türkçe"),
            (693, 3_600_000, 7_200_000, false, "Türkçe — Yönetmenin Uzun Altyazı Seçimi"),
            (693, 3_600_000, 7_200_000, true, "Türkçe — Yönetmenin Uzun Altyazı Seçimi"),
            (1_470, 38_000, 5_717_931, false, "Türkçe"),
            (1_470, 3_600_000, 7_200_000, true, "Türkçe — Yönetmenin Uzun Altyazı Seçimi"),
        ]

        for testCase in cases {
            let measurement = try measure(
                width: testCase.width,
                position: testCase.position,
                duration: testCase.duration,
                showsRemaining: testCase.remaining,
                subtitleTitle: testCase.title
            )

            #expect(
                measurement.playPause.minX >= measurement.window.minX
                    + TransportControls.horizontalPadding - 0.5
            )
            #expect(
                measurement.fullScreen.maxX <= measurement.window.maxX
                    - TransportControls.horizontalPadding + 0.5
            )
            #expect(measurement.seek.width >= 76)
        }
    }

    private func measure(
        width: CGFloat,
        position: UInt64,
        duration: UInt64,
        showsRemaining: Bool,
        subtitleTitle: String
    ) throws -> Measurement {
        let session = FakeSession()
        session.currentPosition = position
        session.currentDuration = duration
        session.currentState = .ready
        session.subtitleTracks = [
            FfiTrackDescriptor(
                id: 1,
                kind: .subtitle,
                language: "tr",
                codec: "subrip",
                isDefault: false,
                title: subtitleTitle
            )
        ]

        let model = PlayerModel(
            recentStore: MemoryRecentStore(),
            startsPolling: false,
            managesCursor: false,
            preferenceStore: MemoryPreferenceStore(),
            sessionFactory: { _ in session }
        )
        model.attach(to: MPVVideoView.makePlaybackSurface())
        model.openMedia(at: URL(fileURLWithPath: "/fixtures/media/layout.mkv"))
        model.consume([.stateChanged(state: .ready)])
        if showsRemaining {
            model.toggleDurationMode()
        }
        let subtitle = try #require(model.subtitleMenu.dropFirst().first?.entries.first?.token)
        model.selectSubtitle(token: subtitle)

        var presentedPanel: PresentedPanel?
        var frames: [TransportLayoutElement: CGRect] = [:]
        let bar = TransportControls(
            model: model,
            presentedPanel: Binding(
                get: { presentedPanel },
                set: { presentedPanel = $0 }
            ),
            onInteractionOutsidePanel: {},
            onToggleFullScreen: {},
            onLayout: { frames = $0 }
        )
        .frame(width: width, height: TransportControls.height)

        let host = NSHostingView(rootView: bar)
        host.frame = CGRect(x: 0, y: 0, width: width, height: TransportControls.height)
        host.layoutSubtreeIfNeeded()
        RunLoop.current.run(until: Date().addingTimeInterval(0.1))

        defer {
            model.shutdown()
        }

        return Measurement(
            window: host.bounds,
            playPause: try #require(frames[.playPause], "play/pause frame was not reported"),
            seek: try #require(frames[.seek], "seek frame was not reported"),
            fullScreen: try #require(frames[.fullScreen], "full-screen frame was not reported")
        )
    }

    private struct Measurement {
        let window: CGRect
        let playPause: CGRect
        let seek: CGRect
        let fullScreen: CGRect
    }
}
