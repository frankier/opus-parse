use std::collections::{HashSet};
use std::path::{Component, Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

fn entry_is_subtitle(entry: &DirEntry) -> bool {
    entry.file_type().is_file() &&
        entry.file_name().to_str().map(|s| s.ends_with(".xml.gz")).unwrap_or(false)
}

fn id_of_path(path: &Path) -> Option<u64> {
    let mut components = path.components();
    components.next_back();
    components.next_back().and_then(|comp|
        match comp {
            Component::Normal(path) => Some(path),
            _ => None
        })
        .and_then(|path| path.to_str())
        .and_then(|path| path.parse::<u64>().ok())
}

/// Walk the tree of all subtitles for single language
pub fn walk(path: &Path) -> Box<Iterator<Item=(u64, PathBuf)>> {
    // XXX: Returns a boxed iterator. Change to impl Iterator when in stable.
    let walker = WalkDir::new(path).into_iter();
    let mut seen = HashSet::new();
    Box::new(walker
        .filter_map(|entry_result| entry_result.ok().and_then(
            |entry| {
                if entry_is_subtitle(&entry) { Some(entry) } else { None }
            }))
        .filter_map(move |subtitle_entry| {
            let subtitle_path = subtitle_entry.path();
            let movie_id = id_of_path(subtitle_path).unwrap();
            if seen.contains(&movie_id) {
                None
            } else {
                seen.insert(movie_id);
                Some((movie_id, subtitle_path.to_owned()))
            }
        }))
}
