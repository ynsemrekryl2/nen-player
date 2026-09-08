import Foundation
import NenCore

/// What a launch input turned into, once the core has judged it.
///
/// Not a `Result`: an ordinary launch (double-click, `⌘N`-style reopen) has
/// no positional argument either, and that is not a failure worth telling the
/// user about — it is simply not a handoff. `.none` and `.rejected` are kept
/// apart so a caller never shows a message for the common case.
public enum HandoffOutcome: Equatable {
    /// A medium to open, the way `⌘O` opens one.
    ///
    /// `startPositionMs` is applied once the medium has loaded
    /// (`PlayerModel.applyHandoffStartPosition()`, NEN-081) — not here, and
    /// not at `PlayerModel.openMedia(at:startPositionMs:)`'s own `load` call,
    /// because the medium's duration is unknowable that early.
    case medium(URL, startPositionMs: UInt64?)
    /// No positional locator was present. Not an error.
    case none
    /// A locator was present but could not be opened. The user-facing message
    /// is the closed `HandoffIntake.rejectionMessage` value; no arbitrary
    /// input-derived string is carried by this outcome (K23, NEN-083).
    case rejected
}

extension HandoffOutcome: CustomStringConvertible, CustomDebugStringConvertible, CustomReflectable {
    public var description: String {
        switch self {
        case let .medium(url, startPositionMs):
            let kind = url.isFileURL ? "local" : "remote"
            let scheme = url.isFileURL ? "<none>" : (url.scheme?.lowercased() ?? "<unknown>")
            let fileExtension = url.isFileURL && !url.pathExtension.isEmpty
                ? url.pathExtension
                : "<none>"
            let position = startPositionMs.map(String.init) ?? "<none>"
            return "HandoffOutcome.medium(kind: \(kind), value: <redacted>, extension: \(fileExtension), scheme: \(scheme), startPositionMs: \(position))"
        case .none:
            return "HandoffOutcome.none"
        case .rejected:
            return "HandoffOutcome.rejected"
        }
    }

    public var debugDescription: String { description }

    public var customMirror: Mirror {
        switch self {
        case let .medium(url, startPositionMs):
            let kind = url.isFileURL ? "local" : "remote"
            let scheme = url.isFileURL ? "<none>" : (url.scheme?.lowercased() ?? "<unknown>")
            let fileExtension = url.isFileURL && !url.pathExtension.isEmpty
                ? url.pathExtension
                : "<none>"
            let position = startPositionMs.map(String.init) ?? "<none>"
            return Mirror(
                self,
                children: [
                    "kind": kind,
                    "value": "<redacted>",
                    "extension": fileExtension,
                    "scheme": scheme,
                    "startPositionMs": position,
                ]
            )
        case .none, .rejected:
            return Mirror(self, children: [:])
        }
    }
}

extension FfiHandoffLocator: @retroactive CustomStringConvertible,
    @retroactive CustomDebugStringConvertible,
    @retroactive CustomReflectable {
    public var description: String {
        switch self {
        case let .localPath(path):
            let fileExtension = URL(fileURLWithPath: path).pathExtension
            return "FfiHandoffLocator.localPath(value: <redacted>, extension: \(fileExtension.isEmpty ? "<none>" : fileExtension))"
        case let .remote(url):
            let scheme = URL(string: url)?.scheme?.lowercased() ?? "<unknown>"
            return "FfiHandoffLocator.remote(value: <redacted>, scheme: \(scheme))"
        }
    }

    public var debugDescription: String { description }

    public var customMirror: Mirror {
        switch self {
        case let .localPath(path):
            let fileExtension = URL(fileURLWithPath: path).pathExtension
            return Mirror(
                self,
                children: [
                    "kind": "local",
                    "value": "<redacted>",
                    "extension": fileExtension.isEmpty ? "<none>" : fileExtension,
                    "scheme": "<none>",
                ]
            )
        case let .remote(url):
            let scheme = URL(string: url)?.scheme?.lowercased() ?? "<unknown>"
            return Mirror(
                self,
                children: [
                    "kind": "remote",
                    "value": "<redacted>",
                    "extension": "<none>",
                    "scheme": scheme,
                ]
            )
        }
    }
}

extension FfiHandoffRequest: @retroactive CustomStringConvertible,
    @retroactive CustomDebugStringConvertible,
    @retroactive CustomReflectable {
    public var description: String {
        "FfiHandoffRequest(locator: \(locator), startPositionMs: \(startPositionMs.map(String.init) ?? "<none>"))"
    }

    public var debugDescription: String { description }

    public var customMirror: Mirror {
        Mirror(
            self,
            children: [
                "locator": locator,
                "startPositionMs": startPositionMs.map(String.init) ?? "<none>",
            ]
        )
    }
}

/// Turns a raw launch (`argv`, an opened document's URL, or a `nenplayer://`
/// string) into something [`PlayerModel.openMedia(at:)`] can open.
///
/// ADR-0043 Karar 4: every judgement about the input is the core's
/// (`nen_app::handoff`, reached through [`parseHandoffArgv`]/
/// [`parseHandoffUrl`]); this type only turns the answer into a `URL` and a
/// user-facing sentence, on the same closed message
/// [`PlayerModel.openMedia(at:)`] already shows for a source it cannot open.
public enum HandoffIntake {
    /// What is shown for any locator the core would not open. Deliberately
    /// the same sentence `PlayerModel.openMedia(at:)` already uses for a
    /// source it refuses — no new user-facing surface, and no path or scheme
    /// name leaks into it (`security-policy.md` §1).
    public static let rejectionMessage = PlaybackPresentation.unsupportedMediaSourceMessage

    /// A process launch's `CommandLine.arguments` (or an equivalent array).
    ///
    /// The overwhelmingly common case — a plain launch, or a relaunch via
    /// Dock/`applicationShouldHandleReopen` — carries no positional argument,
    /// so `.none` is the expected answer, not an edge case.
    public static func fromArgv(_ argv: [String]) -> HandoffOutcome {
        do {
            return outcome(for: try parseHandoffArgv(argv: argv))
        } catch FfiHandoffRejection.NoLocator {
            return .none
        } catch {
            return .rejected
        }
    }

    /// An opened document's `file://` URL, or a `nenplayer://` string built by
    /// a sender — both arrive through the same `application(_:open:)`
    /// callback, so both are resolved here.
    ///
    /// Unlike `fromArgv`, this path is never an ordinary launch: AppKit only
    /// calls it because something was actually handed over, so even
    /// `NoLocator` (which `parseHandoffUrl` cannot itself return, but a future
    /// core change might) is shown rather than swallowed.
    public static func fromURL(_ raw: String) -> HandoffOutcome {
        do {
            let request = try parseHandoffUrl(url: raw)
            return outcome(for: request)
        } catch {
            return .rejected
        }
    }

    private static func outcome(for request: FfiHandoffRequest) -> HandoffOutcome {
        guard let url = url(for: request.locator) else {
            return .rejected
        }
        return .medium(url, startPositionMs: request.startPositionMs)
    }

    private static func url(for locator: FfiHandoffLocator) -> URL? {
        switch locator {
        case let .localPath(path):
            return URL(fileURLWithPath: path)
        case let .remote(url):
            return URL(string: url)
        }
    }
}

/// Reconciles a handoff with `PlayerModel`'s own lifecycle.
///
/// `PlayerModel` is constructed by SwiftUI at the same time as the AppKit
/// delegate, not before it, so a handoff observed at launch has nowhere to
/// land yet. Worse, the two AppKit callbacks that can carry one do not fire
/// in the order a caller would guess: measured at a cold open-with launch,
/// `application(_:open:)` runs **before** `applicationDidFinishLaunching`,
/// not after. That means the ordinary launch's `.none` (read from `argv` in
/// the latter) arrives *after* a real handoff already queued from the
/// former — and must not erase it.
///
/// `@MainActor` because the model it eventually calls into is.
@MainActor
public final class HandoffCoordinator {
    private var pending: HandoffOutcome?
    private weak var model: PlayerModel?

    public init() {}

    /// Called from either AppKit entry point, in whatever order they fire.
    public func deliver(_ outcome: HandoffOutcome) {
        guard let model else {
            // A second real handoff arriving before the model exists is rare,
            // but when it happens the newer one wins — the same "one medium
            // at a time" reasoning `application(_:open:)` already applies to
            // its own `urls` array.
            if pending == nil || outcome != .none {
                pending = outcome
            }
            return
        }
        model.handleHandoff(outcome)
    }

    /// Called once the model exists (`NenPlayerApp`'s window content
    /// appearing). Replays whatever was queued, if anything.
    public func attach(_ model: PlayerModel) {
        self.model = model
        guard let outcome = pending else { return }
        pending = nil
        model.handleHandoff(outcome)
    }
}
