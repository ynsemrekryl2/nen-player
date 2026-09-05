import Cmpv
import Foundation
import NenCore

// The parts of the adapter that talk to libmpv directly: the event loop, the
// property helpers, and the error mapping. Split out so the port's own surface
// reads as the port and nothing else.
extension MPVPlaybackEngine {
    // MARK: - The event loop

    /// The one thread allowed to call `mpv_wait_event`.
    func runEventLoop() {
        while true {
            lock.lock()
            let done = shutDown
            lock.unlock()
            if done { break }

            guard let event = mpv_wait_event(handle, -1) else { continue }
            if event.pointee.event_id == MPV_EVENT_SHUTDOWN { break }
            if event.pointee.event_id == MPV_EVENT_NONE { continue }
            consume(event)
        }
        // Teardown waits on this before freeing the handle this loop reads.
        pumpFinished.signal()
    }

    /// Translates one mpv event into what the port reports.
    private func consume(_ event: UnsafeMutablePointer<mpv_event>) {
        lock.lock()
        defer { lock.unlock() }

        switch event.pointee.event_id {
        case MPV_EVENT_FILE_LOADED:
            phase = .loaded
            reloadTracksUnlocked()
            pending.append(.stateChanged(state: .ready))
            pending.append(.tracksChanged)

        case MPV_EVENT_VIDEO_RECONFIG:
            // mpv reconfigured its video output, which is the moment the
            // display size can have changed (ADR-0038 Karar 2). The event
            // carries no value: the core re-reads, so there is no size here to
            // go stale, and nothing has to be read on this thread.
            //
            // Measured: one load produces **two** of these, and the size is
            // still unreadable at the first
            // (`evidence/M3/NEN-068-measurement.md`). Both are reported and the
            // shared queue coalesces them — which is precisely why the event
            // was made coalescing rather than why the adapter should filter.
            //
            // Reported headless too: the contract kit builds engines with
            // `vo=null` and mpv emits these there as well, so the kit judges
            // the same behaviour the shell gets.
            pending.append(.videoGeometryChanged)

        case MPV_EVENT_SEEK:
            // mpv has *begun* a seek. Everything counted in `pendingSeeks` up to
            // now is being served, so the next restart is genuinely an answer.
            seekInFlight = true

        case MPV_EVENT_PLAYBACK_RESTART:
            // Fires for an unpause and for the load itself as well as for a
            // seek, so only a seek this adapter actually issued may be reported
            // as one. Measured: an unpause produces a restart with no seek
            // outstanding, and `loadfile` produces one *after* `file-loaded` —
            // that is, after `state()` already answers `Ready`.
            //
            // `pendingSeeks > 0` alone does not separate them (NEN-051). A shell
            // that seeks the moment it sees `Ready` is racing the load's own
            // restart, and whichever arrives first wins: the load's restart
            // would answer the seek with `time-pos` as it stands *before* the
            // core served it — `0 ms` — and the seek's real restart, finding the
            // count already cleared, would then report nothing at all. Requiring
            // `MPV_EVENT_SEEK` first is what makes the answer belong to the seek.
            guard pendingSeeks > 0, seekInFlight else { break }
            seekInFlight = false

            // **One restart can answer several seeks.** mpv merges seeks issued
            // faster than it can service them, so two `seek` commands in a row
            // produce a single `playback-restart` — measured with the contract's
            // back-to-back seek scenario.
            //
            // Every outstanding seek is still answered, because the caller is
            // owed an answer per request: a shell waiting on the second seek's
            // completion would otherwise wait forever. Reporting the same
            // position for all of them is not a fudge — the port defines
            // `SeekCompleted` as "the position actually reached", and that is
            // the position all of them actually reached.
            let at = positionMsUnlocked()
            for _ in 0..<pendingSeeks {
                pending.append(.seekCompleted(positionMs: at))
            }
            pendingSeeks = 0
            pending.append(.positionChanged(positionMs: at))

        case MPV_EVENT_END_FILE:
            guard let end = event.pointee.data
                .map({ $0.assumingMemoryBound(to: mpv_event_end_file.self).pointee })
            else { break }
            if stopRequested {
                stopRequested = false
                break
            }
            // An end only concerns the caller if it is *this* load's end.
            //
            // `loadfile` over an open medium ends the outgoing entry first, and
            // NEN-058 measured that end as `reason=STOP, error=0` — byte for
            // byte what a medium whose bytes are not a container produces. Read
            // as a failure it made the shell report `Dosya okunamadı.` for a
            // file that then went on to load and play, which is how the symptom
            // was first seen: the *second* medium opened in a session always
            // failed, and the sidecar beside it was never the variable.
            //
            // `playlist_entry_id` is what tells them apart, and it is mpv's own
            // answer rather than something inferred: the outgoing entry ends
            // under its own id while `loadfile` has already reported the new
            // one.
            guard end.playlist_entry_id == currentEntryId else { break }

            // Neither our `stop` nor a file we have moved on from, so this
            // medium ended by itself or failed. With `keep-open=yes` a natural
            // EOF does not produce this event at all, which is what makes the
            // remaining cases failures.
            //
            // Measured on two shapes of bad input: a missing file gives
            // `REASON_ERROR` with "loading failed"; a file whose bytes are not
            // a container gives `REASON_STOP` with no error code. Both are
            // load failures, and the reason code alone cannot tell them from
            // each other.
            phase = .failed
            let error = Self.loadFailure(from: end)
            pending.append(.stateChanged(state: .failed))
            pending.append(.failed(error: error))

        case MPV_EVENT_PROPERTY_CHANGE:
            guard let data = event.pointee.data else { break }
            let property = data.assumingMemoryBound(to: mpv_event_property.self).pointee
            switch String(cString: property.name) {
            case "time-pos":
                guard property.format == MPV_FORMAT_DOUBLE,
                      let value = property.data?.assumingMemoryBound(to: Double.self).pointee
                else { break }
                pending.append(.positionChanged(positionMs: UInt64(max(0, value * 1000))))

            case "eof-reached":
                guard property.format == MPV_FORMAT_FLAG,
                      let raw = property.data?.assumingMemoryBound(to: Int32.self).pointee
                else { break }
                let ended = raw != 0
                guard ended != atEndOfFile else { break }
                atEndOfFile = ended
                // Only the transition into the end is news. Leaving it — a
                // rewind — is reported by whatever caused it.
                if ended {
                    pending.append(.stateChanged(state: .ended))
                    pending.append(.endReached)
                }

            default:
                break
            }

        default:
            break
        }
    }

    private static func loadFailure(from reason: mpv_event_end_file) -> FfiPlaybackError {
        if reason.error == MPV_ERROR_LOADING_FAILED.rawValue {
            return .LoadFailed(reason: .notFound)
        }
        if reason.error == MPV_ERROR_UNKNOWN_FORMAT.rawValue
            || reason.error == MPV_ERROR_NOTHING_TO_PLAY.rawValue {
            return .LoadFailed(reason: .unsupportedFormat)
        }
        return .LoadFailed(reason: .unreadable)
    }

    // MARK: - Track list

    /// Re-reads `track-list` into the adapter's own model.
    ///
    /// Read field by field rather than by parsing the JSON `track-list`: the
    /// individual properties are a documented interface, and a JSON parse would
    /// be a second place for the shape of that document to matter.
    ///
    /// **Externally added tracks are skipped** (ADR-0013 Karar 5). A document
    /// this adapter injected is a subtitle track as far as mpv is concerned,
    /// and reporting it would be wrong twice over: the catalog would list a
    /// source it never saw — the exact shape of the defect NEN-058 found in
    /// `sub-auto` — and the id would collide, because mpv gives an external
    /// track `ff-index = 0`, which is the video track's number
    /// (`evidence/M3/NEN-027-injection-measurement.md`). The port's `TrackId`
    /// *is* the ff-index, so the collision is not cosmetic.
    func reloadTracksUnlocked() {
        trackList = []
        guard let count = try? int("track-list/count") else { return }
        for index in 0..<count {
            let prefix = "track-list/\(index)"
            if (try? flag("\(prefix)/external")) == true { continue }
            guard let type = try? string("\(prefix)/type"),
                  let kind = Self.kind(of: type),
                  let ffIndex = try? int("\(prefix)/ff-index"),
                  let mpvId = try? int("\(prefix)/id")
            else { continue }
            let codec = (try? string("\(prefix)/codec")) ?? "unknown"
            trackList.append(
                Track(
                    ffIndex: UInt32(max(0, ffIndex)),
                    mpvId: mpvId,
                    kind: kind,
                    language: try? string("\(prefix)/lang"),
                    codec: codec,
                    isDefault: (try? flag("\(prefix)/default")) ?? false,
                    title: try? string("\(prefix)/title")
                )
            )
        }
    }

    private static func kind(of type: String) -> FfiTrackKind? {
        switch type {
        case "audio": return .audio
        case "sub": return .subtitle
        default: return nil   // video, and anything a container invents
        }
    }

    /// Drops the document this adapter injected, if there is one.
    ///
    /// Best effort by design: the id is mpv's, and the only ways it stops being
    /// valid are the two that already cleared it here — a new `loadfile` and a
    /// `stop`. A refusal therefore means the track is gone anyway, which is the
    /// state this call wanted.
    func removeInjectedSubtitle() {
        lock.lock()
        let id = injectedSubtitleId
        injectedSubtitleId = MPVPlaybackEngine.noTrack
        lock.unlock()
        guard id != MPVPlaybackEngine.noTrack else { return }
        try? command(["sub-remove", String(id)])
    }

    // MARK: - State helpers

    func pausedUnlocked() -> Bool {
        (try? flag("pause")) ?? false
    }

    func positionMsUnlocked() -> UInt64 {
        guard let seconds = try? double("time-pos"), seconds.isFinite, seconds > 0 else {
            return 0
        }
        return UInt64((seconds * 1000).rounded())
    }

    // MARK: - Lifecycle helpers

    func requireLive() throws {
        lock.lock()
        defer { lock.unlock() }
        if shutDown { throw FfiPlaybackError.ShutDown }
    }

    func requireMedia() throws {
        lock.lock()
        defer { lock.unlock() }
        if shutDown { throw FfiPlaybackError.ShutDown }
        switch phase {
        case .idle: throw FfiPlaybackError.NotLoaded
        case .failed:
            // A failed load left nothing loaded, so the honest refusal is the
            // same one an untouched engine gives.
            throw FfiPlaybackError.NotLoaded
        case .loading, .loaded: break
        }
    }

    /// Runs `body` under the lock after the usual lifecycle checks.
    func mutate(requireLoaded: Bool = true, _ body: () -> Void) throws {
        if requireLoaded {
            try requireMedia()
        } else {
            try requireLive()
        }
        lock.lock()
        defer { lock.unlock() }
        body()
    }

    func shutdownOnce() {
        lock.lock()
        if shutDown {
            lock.unlock()
            return
        }
        shutDown = true
        phase = .idle
        pending.removeAll()
        lock.unlock()

        // Wakes the pump out of its blocking wait, then tears the core down.
        // `mpv_terminate_destroy` frees the handle, so it may run exactly once
        // — which is what the flag above is for. The port requires `shutdown`
        // to be idempotent; libmpv does not offer that, so the adapter does.
        // Wake the pump out of its blocking wait and wait for it to leave the
        // loop *before* freeing the handle it is reading — `mpv_terminate_destroy`
        // deallocates the handle, and destroying one while another thread sits
        // in `mpv_wait_event` on it is a use-after-free.
        mpv_wakeup(handle)
        _ = pumpFinished.wait(timeout: .now() + 2)
        detachVideoView()
        mpv_terminate_destroy(handle)
    }

    private func detachVideoView() {
        guard let videoView else { return }
        if Thread.isMainThread {
            MainActor.assumeIsolated {
                videoView.detach()
            }
        } else {
            DispatchQueue.main.sync {
                videoView.detach()
            }
        }
    }
}
