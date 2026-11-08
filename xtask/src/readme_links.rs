//! Every README is linked from the README above it (ROADMAP R91, Q190).
//!
//! The root README links each project, a project README links its
//! sub-projects (e.g. memo its slices), so a newcomer can reach every README
//! from the root. The test [`tests::every_readme_is_linked_from_its_parent`]
//! runs in `cargo test` (pre-push hook and CI) and names each README that
//! nothing links. The parent is the nearest README in a directory above; a
//! link may point at the README itself or at its directory.

use anyhow::{Result, bail};

use crate::workspace;

/// The file name of a README.
const README: &str = "README.md";

/// Checks the README links of the repository.
///
/// # Errors
/// Returns an error if git cannot list the files, a README cannot be read, or
/// a README is not linked from its parent; the message lists every such
/// README.
fn check() -> Result<()> {
    let readmes: Vec<String> = workspace::repository_paths()?
        .into_iter()
        .filter(|path| file_name(path) == README)
        .collect();
    let mut unlinked = Vec::new();
    for readme in &readmes {
        let Some(parent) = parent_readme(readme, &readmes) else {
            continue;
        };
        let content = std::fs::read_to_string(workspace::root().join(&parent))?;
        if !links_to(&parent, &content, readme) {
            unlinked.push(format!("{readme} (expected a link in {parent})"));
        }
    }
    if !unlinked.is_empty() {
        bail!(
            "{} README(s) are not linked from the README above them:\n  {}",
            unlinked.len(),
            unlinked.join("\n  ")
        );
    }
    Ok(())
}

/// The last segment of a `/`-separated path.
fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// The directory of a `/`-separated path, `""` for the root.
fn directory(path: &str) -> &str {
    path.rsplit_once('/').map_or("", |(directory, _)| directory)
}

/// The nearest README in a directory above `readme`'s own, or `None` for the
/// root README.
fn parent_readme(readme: &str, readmes: &[String]) -> Option<String> {
    let mut current = directory(readme);
    while !current.is_empty() {
        current = directory(current);
        let candidate = if current.is_empty() {
            README.to_owned()
        } else {
            format!("{current}/{README}")
        };
        if readmes.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}

/// Whether the Markdown `content` of the README at `from` has a link to
/// `target` (a README path) or to its directory.
fn links_to(from: &str, content: &str, target: &str) -> bool {
    let target_directory = directory(target);
    markdown_link_targets(content)
        .filter_map(|link| resolve(directory(from), link))
        .any(|resolved| resolved == target || resolved == target_directory)
}

/// The targets of inline Markdown links (`[text](target)`), without titles.
fn markdown_link_targets(content: &str) -> impl Iterator<Item = &str> {
    content.split("](").skip(1).filter_map(|rest| {
        let end = rest.find(')')?;
        rest[..end].split_whitespace().next()
    })
}

/// Resolves a relative link from the directory `base` to a repository path
/// without trailing `/` or `#fragment`. `None` for external links and for
/// links that leave the repository.
fn resolve(base: &str, link: &str) -> Option<String> {
    if link.contains("://") || link.starts_with("mailto:") || link.starts_with('#') {
        return None;
    }
    let path = link.split('#').next().unwrap_or(link);
    let mut segments: Vec<&str> = base
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    for segment in path.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop()?;
            }
            name => segments.push(name),
        }
    }
    Some(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_readme_is_linked_from_its_parent() {
        if let Err(error) = check() {
            panic!("{error:#}");
        }
    }

    #[test]
    fn the_parent_is_the_nearest_readme_above() {
        let readmes = [
            "README.md".to_owned(),
            "addons/memo/README.md".to_owned(),
            "addons/memo/slices/search/README.md".to_owned(),
        ];
        assert_eq!(
            parent_readme("addons/memo/slices/search/README.md", &readmes).as_deref(),
            Some("addons/memo/README.md")
        );
        assert_eq!(
            parent_readme("addons/memo/README.md", &readmes).as_deref(),
            Some("README.md")
        );
        assert_eq!(parent_readme("README.md", &readmes), None);
    }

    #[test]
    fn links_to_the_readme_or_its_directory_count() {
        let target = "addons/memo/shared/README.md";
        assert!(links_to(
            "addons/memo/README.md",
            "[shared](shared/README.md)",
            target
        ));
        assert!(links_to(
            "addons/memo/README.md",
            "[shared](./shared/)",
            target
        ));
        assert!(links_to(
            "README.md",
            "see [x](addons/memo/shared#build \"t\")",
            target
        ));
        assert!(!links_to(
            "README.md",
            "[other](addons/memo/README.md)",
            target
        ));
        assert!(!links_to(
            "README.md",
            "`addons/memo/shared/README.md`",
            target
        ));
    }

    #[test]
    fn external_and_escaping_links_are_ignored() {
        assert_eq!(resolve("docs", "https://example.com/README.md"), None);
        assert_eq!(resolve("docs", "#contents"), None);
        assert_eq!(resolve("", "../outside.md"), None);
        assert_eq!(
            resolve("crates/ui", "../storage/README.md").as_deref(),
            Some("crates/storage/README.md")
        );
    }
}
