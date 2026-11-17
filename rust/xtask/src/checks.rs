//! Checks that guard the repository's rules: branch names, file length
//! hints, path names and README links (ROADMAP R134-style grouping, Q497).

pub mod check_branch;
pub mod check_file_length;
pub mod check_names;
#[cfg(test)]
pub mod readme_links;
