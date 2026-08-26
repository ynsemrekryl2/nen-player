import Foundation
import NenCore
import Testing

@testable import NenPlaybackMPV

/// NEN-022 DoD #5: nothing the adapter reports carries the medium's path.
///
/// K23 #1 and #3 forbid a media URL or a private full path from ever reaching a
/// log. The reason it has to be checked *here* rather than trusted from Rust is
/// what NEN-010 measured: **an error crossing the FFI boundary is re-printed by
/// the host language**, which never sees a Rust `Debug` impl. UniFFI generates
/// `errorDescription` as `String(reflecting: self)` — so whatever a payload
/// holds is what a Swift log statement would print.
///
/// The defence is therefore structural, not textual: every field of
/// `FfiPlaybackError` and `FfiPlaybackEvent` is a bounded enum or a number, so
/// there is nothing for reflection to leak. These tests hold that line, and the
/// twin at the bottom proves they are not vacuous.
struct RedactionTests {
    /// A path built out of the things K23 names, so a leak is unmistakable.
    static let secretPath =
        "/Users/nen-secret-user/Movies/Private Holiday/S3CR3T.Release.Name.2019.mkv"

    /// Every fragment that must not appear in anything the adapter says.
    static let forbidden = [
        "nen-secret-user", "Private Holiday", "S3CR3T", "Release.Name",
        "Movies", ".mkv", "/Users"
    ]

    private func expectNothingLeaked(_ printed: String, _ what: String) {
        for fragment in Self.forbidden {
            #expect(
                !printed.contains(fragment),
                "\(what) leaked \(fragment)"
            )
        }
    }

    @Test func aFailedLoadReportsNothingAboutWhichMedium() throws {
        let engine = try MPVPlaybackEngine()
        defer { try? engine.shutdown() }

        try engine.load(locator: Self.secretPath)

        // Wait for the failure to land, then inspect everything the adapter
        // says about it — the state, the events, and the errors it throws.
        let deadline = Date().addingTimeInterval(5)
        var events: [FfiPlaybackEvent] = []
        while Date() < deadline, engine.state() != .failed {
            events.append(contentsOf: engine.drainEvents())
            Thread.sleep(forTimeInterval: 0.005)
        }
        events.append(contentsOf: engine.drainEvents())

        #expect(engine.state() == .failed)
        #expect(!events.isEmpty, "the adapter reported nothing at all")

        for event in events {
            // `String(reflecting:)` is exactly what UniFFI's generated
            // `errorDescription` uses, so this is the worst case a careless
            // log statement could print.
            expectNothingLeaked(String(reflecting: event), "an event")
        }

        do {
            _ = try engine.positionMs()
            Issue.record("a failed load still answered position")
        } catch let error as FfiPlaybackError {
            expectNothingLeaked(String(reflecting: error), "a thrown error")
            expectNothingLeaked(error.localizedDescription, "localizedDescription")
        }
    }

    @Test func aContractReportNamesNoMediumEither() throws {
        // The kit's failure lines are rendered in Rust and handed over as
        // strings, so they are the other thing that crosses with a locator in
        // scope. A fixture that lies guarantees there are failures to inspect.
        var lying = ContractTests.fixture
        lying.locator = Self.secretPath
        lying.settleTimeoutMs = 120

        let report = runPlaybackContract(factory: MPVEngineFactory(), fixture: lying)

        #expect(!report.failures.isEmpty, "nothing failed, so nothing was checked")
        for line in report.failures {
            expectNothingLeaked(line, "a contract failure line")
        }
    }

    @Test func theCheckWouldCatchARealLeak() throws {
        // The control. Every assertion above is of the form "this string does
        // not contain that" — which passes trivially if the string is empty or
        // the fragments are wrong. A type that deliberately does carry the path
        // proves the fragments and the comparison both work.
        struct LeakyTwin: CustomStringConvertible {
            let locator: String
            var description: String { "load failed: \(locator)" }
        }

        let twin = LeakyTwin(locator: Self.secretPath)
        let printed = String(reflecting: twin)

        var leaked: [String] = []
        for fragment in Self.forbidden where printed.contains(fragment) {
            leaked.append(fragment)
        }
        #expect(
            leaked.count == Self.forbidden.count,
            "the twin only leaked \(leaked.count) of \(Self.forbidden.count) fragments"
        )
    }
}
