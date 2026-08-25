//! The layered evidence walk (NEN-018 DoD #4, ADR-0009 Karar 6).
//!
//! Two things are checked here: that each layer wins exactly where it should
//! in the fixed order, and that the local-path and remote-URL corpora resolve
//! the way their goldens record.
//!
//! ```text
//! UPDATE_GOLDEN=1 cargo test -p nen-identity --test resolution_layers
//! ```

mod support;

use nen_identity::container::from_tags;
use nen_identity::evidence::{HandoffMetadata, MediaEvidence};
use nen_identity::nfo;
use nen_identity::release_name::MediaKind;

/// Evidence carrying a distinct, recognizable title at every single layer.
/// Each test below removes the layers above the one it is checking, so the
/// winner names the layer that produced it.
fn all_layers() -> MediaEvidence {
    MediaEvidence::for_local_file("/lib/DirLayer (2005)/FileLayer.2004.1080p.BluRay.mkv")
        .with_declared_name("DeclaredLayer.2003.1080p.mkv")
        .with_container(from_tags(Some("ContainerLayer"), Some("2002"), None, &[]))
        .with_nfo(nfo::parse(
            "<movie><title>NfoLayer</title><year>2001</year></movie>",
        ))
        .with_handoff(HandoffMetadata {
            title: Some("HandoffLayer".into()),
            year: Some(2000),
            ..HandoffMetadata::default()
        })
}

#[test]
fn handoff_wins_over_everything() {
    assert_eq!(
        all_layers().resolve().title.as_deref(),
        Some("HandoffLayer")
    );
}

#[test]
fn the_sidecar_wins_when_there_is_no_handoff() {
    let evidence = MediaEvidence::for_local_file("/lib/DirLayer (2005)/FileLayer.2004.1080p.mkv")
        .with_container(from_tags(Some("ContainerLayer"), Some("2002"), None, &[]))
        .with_nfo(nfo::parse(
            "<movie><title>NfoLayer</title><year>2001</year></movie>",
        ));
    assert_eq!(evidence.resolve().title.as_deref(), Some("NfoLayer"));
}

#[test]
fn the_container_wins_when_there_is_no_sidecar() {
    let evidence = MediaEvidence::for_local_file("/lib/DirLayer (2005)/FileLayer.2004.1080p.mkv")
        .with_container(from_tags(Some("ContainerLayer"), Some("2002"), None, &[]));
    assert_eq!(evidence.resolve().title.as_deref(), Some("ContainerLayer"));
}

#[test]
fn the_filename_wins_when_the_container_says_nothing() {
    let evidence =
        MediaEvidence::for_local_file("/lib/DirLayer (2005)/FileLayer.2004.1080p.BluRay.mkv");
    assert_eq!(evidence.resolve().title.as_deref(), Some("FileLayer"));
}

#[test]
fn the_declared_name_stands_in_for_a_missing_filename() {
    let evidence = MediaEvidence::for_remote_url("https://h/d/ABC/0")
        .with_declared_name("DeclaredLayer.2003.1080p.mkv");
    assert_eq!(evidence.resolve().title.as_deref(), Some("DeclaredLayer"));
}

#[test]
fn the_directory_wins_when_the_filename_says_nothing() {
    let evidence = MediaEvidence::for_local_file("/lib/DirLayer (2005)/video.mkv");
    assert_eq!(evidence.resolve().title.as_deref(), Some("DirLayer"));
}

#[test]
fn the_url_path_is_the_last_thing_tried() {
    let evidence = MediaEvidence::for_remote_url("https://h/UrlLayer.2006.1080p/stream.mkv");
    assert_eq!(evidence.resolve().title.as_deref(), Some("UrlLayer"));
}

#[test]
fn nothing_at_all_resolves_to_unknown() {
    let evidence = MediaEvidence::for_local_file("/tmp/video.mkv");
    let resolved = evidence.resolve();

    assert_eq!(resolved.kind, MediaKind::Unknown);
    assert_eq!(resolved.title, None);
    assert!(evidence.identity_candidates().is_empty());
}

/// The winning layer owns identity, but a lower one may still contribute a
/// coordinate it did not have. A folder named for the series and a file named
/// for the episode is the ordinary library layout.
#[test]
fn a_lower_layer_contributes_coordinates_without_taking_the_title() {
    let evidence = MediaEvidence::for_local_file("/lib/Breaking Bad (2008)/S01E02.mkv");
    let resolved = evidence.resolve();

    assert_eq!(resolved.title.as_deref(), Some("Breaking Bad"));
    assert_eq!(resolved.year, Some(2008));
    assert_eq!(resolved.season, Some(1));
    assert_eq!(resolved.episode, Some(2));
    assert_eq!(
        resolved.kind,
        MediaKind::Series,
        "an episode number settles the kind whichever layer supplied it"
    );
}

#[test]
fn a_higher_layer_is_never_overwritten_by_a_lower_one() {
    let resolved = all_layers().resolve();
    let printed = format!("{:?}", resolved.title);

    for loser in [
        "NfoLayer",
        "ContainerLayer",
        "FileLayer",
        "DeclaredLayer",
        "DirLayer",
    ] {
        assert!(!printed.contains(loser), "{loser} overwrote the handoff");
    }
    assert_eq!(resolved.year, Some(2000), "the handoff's year must survive");
}

#[test]
fn the_local_path_corpus_matches_its_golden() {
    let path = support::fixture("dir-layouts.tsv");
    let rendered = support::entries(&path)
        .iter()
        .map(|line| {
            let evidence = MediaEvidence::for_local_file(line);
            support::render(line, &evidence.resolve())
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    support::assert_golden(&path, &rendered);
}

#[test]
fn the_url_corpus_matches_its_golden() {
    let path = support::fixture("url-hints.tsv");
    let rendered = support::entries(&path)
        .iter()
        .map(|line| {
            let hints = nen_identity::url_hints::path_hints(line).join(" | ");
            let evidence = MediaEvidence::for_remote_url(line);
            format!("{}\t[{hints}]", support::render(line, &evidence.resolve()))
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";

    support::assert_golden(&path, &rendered);
}

/// NEN-018 DoD #5. The corpus deliberately contains token-bearing queries and
/// fragments; none of it may survive into hints, identity or candidates.
#[test]
fn no_query_or_host_survives_anywhere_in_the_url_corpus() {
    for url in support::entries(&support::fixture("url-hints.tsv")) {
        let evidence = MediaEvidence::for_remote_url(&url);
        let resolved = evidence.resolve();

        let everything = format!(
            "{:?} {:?} {:?} {:?}",
            evidence,
            nen_identity::url_hints::path_hints(&url),
            resolved.title,
            evidence
                .identity_candidates()
                .iter()
                .map(|candidate| candidate.title.clone())
                .collect::<Vec<_>>(),
        );

        for forbidden in [
            "SUPERSECRET",
            "alice",
            "token",
            "auth=",
            "example.com",
            "127.0.0.1",
        ] {
            assert!(
                !everything.contains(forbidden),
                "{url}: {forbidden:?} leaked into {everything}"
            );
        }
    }
}
