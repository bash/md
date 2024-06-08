use std::path::PathBuf;

pub(crate) fn is_changelog(path: &PathBuf) -> bool {
    path.file_stem()
        .and_then(|s| s.to_str())
        .is_some_and(is_changelog_file)
}

// Inspired by GitLab's changelog detection:
// https://gitlab.com/gitlab-org/gitlab/-/blob/b26ef9a00781635e4359d1169473b3f3bcaeaadf/lib/gitlab/file_detector.rb#L12
fn is_changelog_file(file_stem: &str) -> bool {
    let file_stem = file_stem.to_lowercase();
    file_stem.starts_with("changelog")
        || file_stem.starts_with("history")
        || file_stem.starts_with("changes")
        || file_stem.starts_with("news")
        || file_stem.starts_with("release-notes")
        || file_stem.starts_with("release_notes")
}
