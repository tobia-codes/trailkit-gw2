//! Build script: assembles memo's help text from one file per topic in
//! `help/en/` and `help/de/` plus the privacy policy (ROADMAP R157, R99) and
//! writes `help-en.md` and `help-de.md` to `OUT_DIR`, where
//! `src/help_document.rs` embeds them. A broken help source stops the build
//! with one line per problem that names the file, the line and the fix.

#[path = "src/help_assembly.rs"]
mod help_assembly;

use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

use help_assembly::{HelpError, TopicFile, assemble_help, check_same_topics};

/// The help languages: the folder below `help/`, the policy file in the
/// repository root and the output file name.
const LANGUAGES: [(&str, &str, &str); 2] = [
    ("en", "PRIVACY.md", "help-en.md"),
    ("de", "PRIVACY.de.md", "help-de.md"),
];

/// Assembles both languages, or prints every problem and exits with an error.
fn main() {
    let (Some(crate_folder), Some(output_folder)) =
        (env::var_os("CARGO_MANIFEST_DIR"), env::var_os("OUT_DIR"))
    else {
        eprintln!(
            "trailkit-memo's build script runs only under Cargo, which sets CARGO_MANIFEST_DIR and OUT_DIR"
        );
        process::exit(1);
    };
    let (crate_folder, output_folder) = (PathBuf::from(crate_folder), PathBuf::from(output_folder));
    let repository_root = crate_folder.join("../../..");
    println!("cargo:rerun-if-changed=help");

    let mut errors = Vec::new();
    let mut names_by_language = Vec::new();
    for (language, policy_file, output_file) in LANGUAGES {
        let folder_label = format!("help/{language}");
        let policy_path = repository_root.join(policy_file);
        println!("cargo:rerun-if-changed={}", policy_path.display());
        let files = match read_topic_files(&crate_folder.join(&folder_label), &folder_label) {
            Ok(files) => files,
            Err(error) => {
                errors.push(error);
                continue;
            }
        };
        names_by_language.push((
            folder_label.clone(),
            files
                .iter()
                .map(|file| file.name.clone())
                .collect::<Vec<_>>(),
        ));
        let policy = match fs::read_to_string(&policy_path) {
            Ok(policy) => policy,
            Err(error) => {
                errors.push(HelpError {
                    location: policy_file.to_owned(),
                    problem: format!("cannot read the privacy policy: {error}"),
                    fix: "restore the file in the repository root".to_owned(),
                });
                continue;
            }
        };
        match assemble_help(&folder_label, &files, &policy) {
            Ok(document) => {
                if let Err(error) = fs::write(output_folder.join(output_file), document) {
                    errors.push(HelpError {
                        location: output_file.to_owned(),
                        problem: format!("cannot write the help: {error}"),
                        fix: "check the build folder's free space and permissions".to_owned(),
                    });
                }
            }
            Err(language_errors) => errors.extend(language_errors),
        }
    }
    if let [(first_folder, first_names), (second_folder, second_names)] =
        names_by_language.as_slice()
    {
        errors.extend(check_same_topics(
            (first_folder, first_names),
            (second_folder, second_names),
        ));
    }

    if !errors.is_empty() {
        eprintln!("The help source of trailkit-memo is broken (ROADMAP R157):");
        for error in &errors {
            eprintln!("  {error}");
        }
        process::exit(1);
    }
}

/// Reads every file of `folder`; `label` names it in messages.
///
/// # Errors
///
/// Returns a [`HelpError`] if the folder or a file cannot be read, or the
/// folder holds something other than `.md` files.
fn read_topic_files(folder: &Path, label: &str) -> Result<Vec<TopicFile>, HelpError> {
    let read_error = |location: String, error: std::io::Error| HelpError {
        location,
        problem: format!("cannot read it: {error}"),
        fix: "keep one readable UTF-8 `NN-name.md` file per topic in the folder".to_owned(),
    };
    let entries = fs::read_dir(folder).map_err(|error| read_error(label.to_owned(), error))?;
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| read_error(label.to_owned(), error))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let location = format!("{label}/{name}");
        if !name.ends_with(".md") || !entry.path().is_file() {
            return Err(HelpError {
                location,
                problem: "not a topic file".to_owned(),
                fix: "only `NN-name.md` topic files belong in the folder".to_owned(),
            });
        }
        let text = fs::read_to_string(entry.path()).map_err(|error| read_error(location, error))?;
        files.push(TopicFile { name, text });
    }
    Ok(files)
}
