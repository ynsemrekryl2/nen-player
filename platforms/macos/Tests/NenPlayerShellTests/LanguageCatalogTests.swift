import Testing
@testable import NenPlayerShell

@Suite("language catalog")
struct LanguageCatalogTests {
    @Test("common languages are present, named in their own tongue")
    func commonLanguagesAreNamed() {
        let codes = LanguageCatalog.allCodes
        #expect(codes.contains("en"))
        #expect(codes.contains("fr"))
        #expect(codes.contains("tr"))
        #expect(SubtitleMenuPresentation.endonym(for: "en") == "English")
        #expect(SubtitleMenuPresentation.endonym(for: "fr") == "Français")
        #expect(SubtitleMenuPresentation.endonym(for: "tr") == "Türkçe")
    }

    @Test("no code repeats")
    func noDuplicates() {
        let codes = LanguageCatalog.allCodes
        #expect(codes.count == Set(codes).count)
    }

    @Test("every listed code is one Foundation can actually name")
    func everyCodeIsNamed() {
        for code in LanguageCatalog.allCodes {
            #expect(
                SubtitleMenuPresentation.endonym(for: code) != code.uppercased(),
                "\(code) has no real endonym and should have been filtered out"
            )
        }
    }

    @Test("the list is sorted by its own endonym")
    func sortedByEndonym() {
        let codes = LanguageCatalog.allCodes
        let endonyms = codes.map { SubtitleMenuPresentation.endonym(for: $0) }
        #expect(endonyms == endonyms.sorted())
    }
}
