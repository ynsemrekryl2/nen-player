import NenCore
import Testing

@testable import NenPlayerShell

@Suite("verified media identity presentation")
struct VerifiedMediaIdentityPresentationTests {
    @Test("a verified film uses its title and year")
    func verifiedFilmIdentityProducesCompactLabel() {
        let identity = FfiVerifiedMediaIdentity(
            title: "Inception",
            year: 2010,
            season: nil,
            episode: nil
        )

        #expect(VerifiedMediaIdentityPresentation.label(for: identity) == "Inception (2010)")
    }

    @Test("a verified series uses compact season and episode coordinates")
    func verifiedSeriesIdentityProducesSeasonEpisodeLabel() {
        let identity = FfiVerifiedMediaIdentity(
            title: "Breaking Bad",
            year: 2008,
            season: 1,
            episode: 2
        )

        #expect(
            VerifiedMediaIdentityPresentation.label(for: identity)
                == "Breaking Bad (2008) · S01E02"
        )
    }

    @Test("missing identity fields are omitted without placeholder text")
    func missingIdentityFieldsAreOmitted() {
        let titleOnly = FfiVerifiedMediaIdentity(
            title: "A Film",
            year: nil,
            season: nil,
            episode: nil
        )
        let seasonOnly = FfiVerifiedMediaIdentity(
            title: "A Series",
            year: nil,
            season: 3,
            episode: nil
        )

        #expect(VerifiedMediaIdentityPresentation.label(for: titleOnly) == "A Film")
        #expect(VerifiedMediaIdentityPresentation.label(for: seasonOnly) == "A Series · S03")
    }

    @Test("basename remains visible until a verified identity exists")
    func noIdentityKeepsBasename() {
        #expect(
            VerifiedMediaIdentityPresentation.mediaTitle(
                mediaName: "fixture-clip.mkv",
                identity: nil
            ) == "fixture-clip.mkv"
        )
        #expect(
            VerifiedMediaIdentityPresentation.mediaTitle(
                mediaName: "fixture-clip.mkv",
                identity: FfiVerifiedMediaIdentity(
                    title: "Verified Film",
                    year: 2010,
                    season: nil,
                    episode: nil
                )
        ) == "Verified Film (2010)"
        )
    }

    @Test("a verified label never adds provider or transport metadata")
    func verifiedLabelContainsOnlyApprovedFields() {
        let identity = FfiVerifiedMediaIdentity(
            title: "Verified Film",
            year: 2010,
            season: 1,
            episode: 2
        )

        let label = VerifiedMediaIdentityPresentation.label(for: identity)
        #expect(!label.contains("/"))
        #expect(!label.contains("?"))
        #expect(!label.contains("#"))
        #expect(!label.contains("file_id"))
        #expect(!label.contains("0001020304050607"))
    }

    @Test("identity transition keeps the existing 0.24 second ease-out contract")
    func identityChangeUsesExpectedTransitionContract() {
        #expect(VerifiedMediaIdentityPresentation.transitionDuration == 0.24)
    }
}
