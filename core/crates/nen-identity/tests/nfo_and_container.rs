//! `.nfo` sidecars and container tags against real-shaped fixtures.
//!
//! The unit tests in `nfo` and `container` cover the parsing rules; this
//! covers the files as they actually appear on disk in a Kodi or Plex library,
//! and proves that a malformed one degrades to "nothing recognized" rather
//! than to an error or a panic.

mod support;

use std::fs;

use nen_identity::container::{from_tags, to_parsed};
use nen_identity::evidence::MediaEvidence;
use nen_identity::nfo;
use nen_identity::release_name::MediaKind;

fn read_nfo(name: &str) -> String {
    let path = support::corpus_dir().join("nfo").join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

#[test]
fn a_kodi_movie_sidecar_yields_a_full_identity() {
    let parsed = nfo::parse(&read_nfo("kodi-movie.nfo"));

    assert_eq!(parsed.title.as_deref(), Some("Inception"));
    assert_eq!(parsed.year, Some(2010));
    assert_eq!(parsed.imdb_id.as_deref(), Some("tt1375666"));
    assert_eq!(parsed.tmdb_id.as_deref(), Some("27205"));
}

#[test]
fn a_kodi_episode_sidecar_yields_season_and_episode() {
    let parsed = nfo::parse(&read_nfo("kodi-episode.nfo"));

    assert_eq!(parsed.season, Some(1));
    assert_eq!(parsed.episode, Some(2));
    assert_eq!(parsed.imdb_id.as_deref(), Some("tt0959621"));
}

#[test]
fn a_url_only_sidecar_yields_just_the_id() {
    let parsed = nfo::parse(&read_nfo("url-only.nfo"));

    assert_eq!(parsed.imdb_id.as_deref(), Some("tt0111161"));
    assert_eq!(parsed.title, None);
}

#[test]
fn a_malformed_sidecar_degrades_instead_of_failing() {
    let parsed = nfo::parse(&read_nfo("malformed-unclosed.nfo"));

    assert!(
        parsed.title.is_none(),
        "an unclosed element must not yield a title"
    );
    assert!(
        parsed.year.is_none(),
        "a truncated year must not be accepted"
    );
}

#[test]
fn every_sidecar_fixture_is_readable_and_never_panics() {
    let dir = support::corpus_dir().join("nfo");
    let entries = fs::read_dir(&dir).unwrap_or_else(|err| panic!("cannot read {dir:?}: {err}"));

    let mut count = 0;
    for entry in entries {
        let path = entry.expect("cannot read directory entry").path();
        if path.extension().is_some_and(|ext| ext == "nfo") {
            let contents = fs::read_to_string(&path).unwrap_or_default();
            let _ = nfo::parse(&contents);
            count += 1;
        }
    }
    assert!(count >= 4, "expected a corpus of sidecars, found {count}");
}

#[test]
fn a_sidecar_supplies_the_identity_a_filename_cannot() {
    let evidence = MediaEvidence::for_local_file("/lib/video.mkv")
        .with_nfo(nfo::parse(&read_nfo("kodi-movie.nfo")));
    let resolved = evidence.resolve();

    assert_eq!(resolved.title.as_deref(), Some("Inception"));
    assert_eq!(resolved.year, Some(2010));
    assert_eq!(evidence.imdb_id(), Some("tt1375666"));
}

#[test]
fn a_muxed_release_name_in_the_container_title_is_parsed() {
    let metadata = from_tags(
        Some("Breaking.Bad.S01E02.1080p.WEB-DL.DDP5.1.H264-NTb"),
        None,
        Some(2_820_000),
        &["Chapter 01".into(), "Chapter 02".into()],
    );
    let parsed = to_parsed(&metadata);

    assert_eq!(parsed.title.as_deref(), Some("Breaking Bad"));
    assert_eq!(parsed.season, Some(1));
    assert_eq!(parsed.episode, Some(2));
    assert_eq!(parsed.kind, MediaKind::Series);
    assert_eq!(metadata.chapter_titles.len(), 2);
}

#[test]
fn a_deliberate_container_title_needs_no_release_tags() {
    let metadata = from_tags(Some("Arrival"), Some("2016-11-11"), None, &[]);
    let parsed = to_parsed(&metadata);

    assert_eq!(parsed.title.as_deref(), Some("Arrival"));
    assert_eq!(parsed.year, Some(2016));
    assert_eq!(parsed.kind, MediaKind::Movie);
}
