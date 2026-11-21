//! Assembles memo's help text from one file per topic (ROADMAP R157): the
//! numbered files of `help/<language>/` in name order, the last one, the
//! privacy topic, completed with the privacy policy `PRIVACY.md` /
//! `PRIVACY.de.md` from the repository root (R99). The build script
//! (`build.rs`) runs it and embeds the result; the crate's tests check its
//! rules. Every broken rule becomes a [`HelpError`] that names the file, the
//! line and how to fix it.

use std::{fmt, fs, path::Path};

/// The topic file that receives the privacy policy, without its number.
const PRIVACY_TOPIC: &str = "privacy";

/// One topic file of a help language folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopicFile {
    /// The file name, e.g. `01-getting-started.md`.
    pub name: String,
    /// The file's content.
    pub text: String,
}

/// A broken help source rule, printed as `<location>: <problem> – <fix>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelpError {
    /// The file (and line, if any), e.g. `help/de/03-editing-notes.md:12`.
    pub location: String,
    /// What is wrong.
    pub problem: String,
    /// How to fix it.
    pub fix: String,
}

impl fmt::Display for HelpError {
    /// Writes the error as one line for the build output.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}: {} – {}",
            self.location, self.problem, self.fix
        )
    }
}

/// Reads every topic file of `folder` in name order; `label` names the
/// folder in messages, e.g. `help/de`.
///
/// # Errors
///
/// Returns every problem found: the folder cannot be read, it holds
/// something other than `.md` files (a stray file or a subfolder), or a file
/// cannot be read as UTF-8 text.
pub fn read_topic_files(folder: &Path, label: &str) -> Result<Vec<TopicFile>, Vec<HelpError>> {
    let read_error = |location: String, error: std::io::Error| HelpError {
        location,
        problem: format!("cannot read it: {error}"),
        fix: "keep one readable UTF-8 `NN-name.md` file per topic in the folder".to_owned(),
    };
    let entries = fs::read_dir(folder)
        .and_then(|entries| entries.collect::<Result<Vec<_>, _>>())
        .map_err(|error| vec![read_error(label.to_owned(), error)])?;
    let mut entries: Vec<_> = entries
        .into_iter()
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .collect();
    entries.sort();
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for (name, path) in entries {
        let location = format!("{label}/{name}");
        if !name.ends_with(".md") || !path.is_file() {
            errors.push(HelpError {
                location,
                problem: "not a topic file".to_owned(),
                fix: "only `NN-name.md` topic files belong in the folder".to_owned(),
            });
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(text) => files.push(TopicFile { name, text }),
            Err(error) => errors.push(read_error(location, error)),
        }
    }
    if errors.is_empty() {
        Ok(files)
    } else {
        Err(errors)
    }
}

/// Builds one language's help text from its topic `files` (in any order)
/// and its privacy `policy`; `folder` is the folder for messages, e.g.
/// `help/de`. Windows line ends (`\r\n`) become `\n`, as they do in the
/// policy.
///
/// # Errors
///
/// Returns every broken rule: a file name other than `NN-name.md`, numbers
/// that are not `01`, `02`, … without gaps or duplicates, a file that does
/// not start with its one `##` topic heading or holds a second `##` or a
/// `#` heading, a missing or misplaced privacy topic, or a policy without
/// its `#` title.
pub fn assemble_help(
    folder: &str,
    files: &[TopicFile],
    policy: &str,
) -> Result<String, Vec<HelpError>> {
    let mut files: Vec<&TopicFile> = files.iter().collect();
    files.sort_by(|first, second| first.name.cmp(&second.name));
    let mut errors = Vec::new();
    if files.is_empty() {
        errors.push(HelpError {
            location: folder.to_owned(),
            problem: "no topic files".to_owned(),
            fix: "add one `NN-name.md` file per topic, starting with `01`".to_owned(),
        });
    }
    for (index, file) in files.iter().enumerate() {
        let location = format!("{folder}/{}", file.name);
        check_file_name(&location, &file.name, index + 1, &mut errors);
        check_headings(&location, &file.text, &mut errors);
    }
    check_privacy_topic(folder, &files, &mut errors);
    let policy_body = policy_body(policy).ok_or_else(|| HelpError {
        location: format!("{folder} (privacy policy)"),
        problem: "the policy has no `# ` title line".to_owned(),
        fix: "start the policy with its `# ` title; the help drops it".to_owned(),
    });
    let policy_body = match policy_body {
        Ok(body) if errors.is_empty() => body,
        Ok(_) => return Err(errors),
        Err(error) => {
            errors.push(error);
            return Err(errors);
        }
    };
    let topics: Vec<String> = files
        .iter()
        .map(|file| file.text.replace("\r\n", "\n").trim_end().to_owned())
        .collect();
    let mut document = topics.join("\n\n");
    document.push_str("\n\n");
    document.push_str(&policy_body);
    Ok(document)
}

/// Compares the file names of two language folders: every topic exists in
/// both, so both languages keep the same topics in the same order (R95).
pub fn check_same_topics(
    (first_folder, first_names): (&str, &[String]),
    (second_folder, second_names): (&str, &[String]),
) -> Vec<HelpError> {
    let missing = |names: &[String], others: &[String], folder: &str, other_folder: &str| {
        names
            .iter()
            .filter(|name| !others.contains(name))
            .map(|name| HelpError {
                location: format!("{folder}/{name}"),
                problem: format!("no file of the same name in {other_folder}"),
                fix: format!("add {other_folder}/{name} or rename the file to match"),
            })
            .collect::<Vec<_>>()
    };
    let mut errors = missing(first_names, second_names, first_folder, second_folder);
    errors.extend(missing(
        second_names,
        first_names,
        second_folder,
        first_folder,
    ));
    errors
}

/// Checks that `name` is `NN-name.md` with the two-digit `number` its
/// position in name order gives.
fn check_file_name(location: &str, name: &str, number: usize, errors: &mut Vec<HelpError>) {
    let expected = format!("{number:02}");
    let parsed = name
        .strip_suffix(".md")
        .and_then(|stem| stem.split_once('-'))
        .filter(|(digits, slug)| {
            digits.len() == 2
                && digits.bytes().all(|byte| byte.is_ascii_digit())
                && is_kebab_case(slug)
        });
    match parsed {
        None => errors.push(HelpError {
            location: location.to_owned(),
            problem: "the file name is not `NN-name.md`".to_owned(),
            fix: format!(
                "rename it to a two-digit number and a kebab-case name, e.g. `{expected}-getting-started.md`"
            ),
        }),
        Some((digits, _)) if digits != expected => errors.push(HelpError {
            location: location.to_owned(),
            problem: format!("number {digits}, but it is topic {expected} in name order"),
            fix: "number the files 01, 02, … without gaps or duplicates; the numbers set the topic order"
                .to_owned(),
        }),
        Some(_) => {}
    }
}

/// Returns whether `text` is lowercase ASCII words joined by single `-`.
fn is_kebab_case(text: &str) -> bool {
    !text.is_empty()
        && text.split('-').all(|word| {
            !word.is_empty()
                && word
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

/// Checks that `text` starts with its one `## ` topic heading and holds no
/// second `## ` and no `# ` heading.
fn check_headings(location: &str, text: &str, errors: &mut Vec<HelpError>) {
    let first_line = text.lines().next().unwrap_or_default();
    let has_title = first_line
        .strip_prefix("## ")
        .is_some_and(|title| !title.trim().is_empty());
    if !has_title {
        errors.push(HelpError {
            location: format!("{location}:1"),
            problem: "the file does not start with its `## ` topic heading".to_owned(),
            fix: "put the topic title as `## Title` on the first line".to_owned(),
        });
    }
    for (index, line) in text.lines().enumerate().skip(1) {
        let line_location = format!("{location}:{}", index + 1);
        if line.starts_with("## ") {
            errors.push(HelpError {
                location: line_location,
                problem: format!("a second topic heading `{line}`"),
                fix: "each file holds exactly one topic; move it into its own numbered file or make it a `###` section"
                    .to_owned(),
            });
        } else if line.starts_with("# ") {
            errors.push(HelpError {
                location: line_location,
                problem: format!("a `#` heading `{line}`"),
                fix: "the help has no `#` level; use `###` for a section".to_owned(),
            });
        }
    }
}

/// Checks that the privacy topic exists and comes last, so the policy
/// appended to it closes the help.
fn check_privacy_topic(folder: &str, files: &[&TopicFile], errors: &mut Vec<HelpError>) {
    let is_privacy = |file: &&TopicFile| {
        file.name
            .strip_suffix(".md")
            .and_then(|stem| stem.split_once('-'))
            .is_some_and(|(_, slug)| slug == PRIVACY_TOPIC)
    };
    match files.iter().position(is_privacy) {
        None => errors.push(HelpError {
            location: folder.to_owned(),
            problem: format!("no `NN-{PRIVACY_TOPIC}.md` topic"),
            fix: format!(
                "add `NN-{PRIVACY_TOPIC}.md` as the last topic; the privacy policy is appended to it"
            ),
        }),
        Some(position) if position + 1 != files.len() => errors.push(HelpError {
            location: format!("{folder}/{}", files[position].name),
            problem: "the privacy topic is not the last topic".to_owned(),
            fix: "give it the highest number; the privacy policy is appended to the help's end"
                .to_owned(),
        }),
        Some(_) => {}
    }
}

/// The policy below its `# ` title, its `##` sections turned into `###`
/// and the link to the other language cut from its date line, since both
/// languages have their own help; `None` without a title.
fn policy_body(policy: &str) -> Option<String> {
    let mut lines = policy.lines().skip_while(|line| !line.starts_with("# "));
    lines.next()?;
    let mut body = String::new();
    for line in lines {
        if let Some(section) = line.strip_prefix("## ") {
            body.push_str("### ");
            body.push_str(section);
        } else if let Some((date, _other_language)) = line.split_once(" · [") {
            body.push_str(date);
        } else {
            body.push_str(line);
        }
        body.push('\n');
    }
    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A topic file named `name` with `text`.
    fn topic(name: &str, text: &str) -> TopicFile {
        TopicFile {
            name: name.to_owned(),
            text: text.to_owned(),
        }
    }

    /// A minimal policy with a title, a date line and one section.
    const POLICY: &str = "# Privacy\n\nLast updated: 2026-11-11 · [Deutsche Fassung](PRIVACY.de.md)\n\n## Data\n\nText.\n";

    /// The `problem`s of a failed assembly.
    fn problems(result: Result<String, Vec<HelpError>>) -> Vec<String> {
        result
            .expect_err("the help should be rejected")
            .into_iter()
            .map(|error| error.to_string())
            .collect()
    }

    #[test]
    fn joins_topics_in_name_order_and_appends_the_policy() {
        let files = [
            topic("02-privacy.md", "## Privacy\n\nLead.\n"),
            topic("01-start.md", "## Start\n\nText.\n\n"),
        ];
        let document = assemble_help("help/en", &files, POLICY);
        assert_eq!(
            document.as_deref(),
            Ok(
                "## Start\n\nText.\n\n## Privacy\n\nLead.\n\n\nLast updated: 2026-11-11\n\n### Data\n\nText.\n"
            )
        );
    }

    #[test]
    fn rejects_a_second_topic_and_a_title_heading() {
        let files = [
            topic("01-start.md", "## Start\n\n# Old title\n\n## More\n"),
            topic("02-privacy.md", "## Privacy\n"),
        ];
        let problems = problems(assemble_help("help/de", &files, POLICY));
        assert_eq!(problems.len(), 2);
        assert!(problems[0].starts_with("help/de/01-start.md:3: a `#` heading"));
        assert!(problems[1].starts_with("help/de/01-start.md:5: a second topic heading"));
    }

    #[test]
    fn rejects_a_file_without_its_topic_heading() {
        let files = [
            topic("01-start.md", "Text.\n"),
            topic("02-privacy.md", "## Privacy\n"),
        ];
        let problems = problems(assemble_help("help/en", &files, POLICY));
        assert_eq!(problems.len(), 1);
        assert!(problems[0].starts_with("help/en/01-start.md:1: the file does not start"));
    }

    #[test]
    fn rejects_bad_names_gaps_and_duplicates() {
        let files = [
            topic("01-start.md", "## Start\n"),
            topic("01-again.md", "## Again\n"),
            topic("05-privacy.md", "## Privacy\n"),
            topic("notes.md", "## Notes\n"),
        ];
        let problems = problems(assemble_help("help/en", &files, POLICY));
        assert_eq!(problems.len(), 4, "{problems:#?}");
        assert!(problems[0].contains("01-start.md: number 01, but it is topic 02"));
        assert!(problems[1].contains("05-privacy.md: number 05, but it is topic 03"));
        assert!(problems[2].contains("notes.md: the file name is not `NN-name.md`"));
        assert!(problems[3].contains("05-privacy.md: the privacy topic is not the last"));
    }

    #[test]
    fn requires_the_privacy_topic_last() {
        let misplaced = [
            topic("01-privacy.md", "## Privacy\n"),
            topic("02-start.md", "## Start\n"),
        ];
        let problems_misplaced = problems(assemble_help("help/en", &misplaced, POLICY));
        assert!(problems_misplaced[0].contains("is not the last topic"));
        let missing = [topic("01-start.md", "## Start\n")];
        let problems_missing = problems(assemble_help("help/en", &missing, POLICY));
        assert!(problems_missing[0].contains("no `NN-privacy.md` topic"));
    }

    #[test]
    fn requires_a_policy_title() {
        let files = [topic("01-privacy.md", "## Privacy\n")];
        let problems = problems(assemble_help("help/en", &files, "## Data\n"));
        assert!(problems[0].contains("the policy has no `# ` title line"));
    }

    #[test]
    fn rejects_an_empty_file_and_an_empty_topic_heading() {
        let files = [
            topic("01-start.md", ""),
            topic("02-more.md", "## \n\nText.\n"),
            topic("03-privacy.md", "## Privacy\n"),
        ];
        let problems = problems(assemble_help("help/en", &files, POLICY));
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(problems[0].starts_with("help/en/01-start.md:1: the file does not start"));
        assert!(problems[1].starts_with("help/en/02-more.md:1: the file does not start"));
    }

    #[test]
    fn rejects_uppercase_and_umlaut_names() {
        let files = [
            topic("01-Start.md", "## Start\n"),
            topic("02-übersicht.md", "## Übersicht\n"),
            topic("03-privacy.md", "## Privacy\n"),
        ];
        let problems = problems(assemble_help("help/de", &files, POLICY));
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(problems[0].starts_with("help/de/01-Start.md: the file name is not"));
        assert!(problems[1].starts_with("help/de/02-übersicht.md: the file name is not"));
    }

    #[test]
    fn turns_windows_line_ends_into_unix_ones() {
        let files = [
            topic("01-start.md", "## Start\r\n\r\nText.\r\n"),
            topic("02-privacy.md", "## Privacy\r\n"),
        ];
        let document = assemble_help("help/en", &files, &POLICY.replace('\n', "\r\n"));
        assert_eq!(
            document.as_deref(),
            Ok(
                "## Start\n\nText.\n\n## Privacy\n\n\nLast updated: 2026-11-11\n\n### Data\n\nText.\n"
            )
        );
    }

    #[test]
    fn reads_topic_files_in_name_order() {
        let folder = tempfile::tempdir().expect("temp folder");
        fs::write(folder.path().join("02-privacy.md"), "## Privacy\n").expect("write");
        fs::write(folder.path().join("01-start.md"), "## Start\n").expect("write");
        let files = read_topic_files(folder.path(), "help/en").expect("readable folder");
        assert_eq!(
            files,
            [
                topic("01-start.md", "## Start\n"),
                topic("02-privacy.md", "## Privacy\n"),
            ]
        );
    }

    #[test]
    fn rejects_a_stray_file_a_subfolder_and_an_unreadable_file() {
        let folder = tempfile::tempdir().expect("temp folder");
        fs::write(folder.path().join("01-start.md"), "## Start\n").expect("write");
        fs::write(folder.path().join("02-broken.md"), [0xff, 0xfe]).expect("write");
        fs::write(folder.path().join("notes.txt"), "draft").expect("write");
        fs::create_dir(folder.path().join("old.md")).expect("subfolder");
        let errors = read_topic_files(folder.path(), "help/de").expect_err("broken folder");
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(messages.len(), 3, "{messages:#?}");
        assert!(messages[0].starts_with("help/de/02-broken.md: cannot read it"));
        assert!(messages[1].starts_with("help/de/notes.txt: not a topic file"));
        assert!(messages[2].starts_with("help/de/old.md: not a topic file"));
    }

    #[test]
    fn rejects_a_missing_language_folder() {
        let folder = tempfile::tempdir().expect("temp folder");
        let errors =
            read_topic_files(&folder.path().join("fr"), "help/fr").expect_err("missing folder");
        assert_eq!(errors.len(), 1);
        assert!(errors[0].to_string().starts_with("help/fr: cannot read it"));
    }

    #[test]
    fn reports_topics_missing_in_the_other_language() {
        let english = ["01-start.md".to_owned(), "02-privacy.md".to_owned()];
        let german = ["01-start.md".to_owned(), "02-datenschutz.md".to_owned()];
        let errors = check_same_topics(("help/en", &english), ("help/de", &german));
        let messages: Vec<String> = errors.iter().map(ToString::to_string).collect();
        assert_eq!(messages.len(), 2);
        assert!(
            messages[0].starts_with("help/en/02-privacy.md: no file of the same name in help/de")
        );
        assert!(
            messages[1]
                .starts_with("help/de/02-datenschutz.md: no file of the same name in help/en")
        );
    }
}
