use clap::Parser;
use git2::{Error, Repository};
use semver::Version as SemverVersion;
use std::process;

#[derive(Parser, Default, Debug)]
#[command(version, arg_required_else_help = true)]
/// A CLI app to bump semver tag
struct Args {
    #[arg(short = 's', long)]
    /// The scope of the version: major, minor, or patch
    scope: Option<String>,
    #[arg(short = 'o', long)]
    /// The option to be used: alpha, beta, rc, or just left it empty
    option: Option<String>,
    #[arg(short = 'p', long)]
    /// The prefix to be used: prod, stage, sandbox, dev, etc
    prefix: Option<String>,
    #[arg(short = 'd', long, action)]
    /// Dry run mode, do not create a tag
    dry_run: bool,
}

#[derive(Debug, Clone)]
struct Version {
    prefix: Option<String>,
    major: u32,
    minor: u32,
    patch: u32,
    label: Option<String>,
    rc_number: Option<u32>,
}

const SCOPE_MAJOR: &str = "major";
const SCOPE_MINOR: &str = "minor";
const SCOPE_PATCH: &str = "patch";

const OPT_ALPHA: &str = "alpha";
const OPT_BETA: &str = "beta";
const OPT_RC: &str = "rc";

impl Version {
    fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('-').collect();
        let mut prefix_and_version: (Option<String>, &str, Option<String>) = (None, "", None);

        if parts.len() > 2 {
            for (index, part) in parts.iter().enumerate() {
                if is_semver(part) {
                    prefix_and_version = (
                        Some(parts[..index].join("-")),
                        part,
                        Some(parts[index + 1..].join("-")),
                    );
                    break;
                }
            }
        } else if parts.len() == 2 {
            if is_semver(parts[0]) {
                prefix_and_version = (None, parts[0], Some(parts[1].to_string()));
            } else {
                prefix_and_version = (Some(parts[0].to_string()), parts[1], None);
            }
        } else if parts.len() == 1 {
            prefix_and_version = (None, parts[0], None);
        } else {
            return Err("Invalid parts length".to_string());
        }

        let version_parts: Vec<&str> = prefix_and_version.1.split('.').collect();
        if version_parts.len() < 3 {
            return Err("Invalid version format".to_string());
        }

        let major = version_parts[0]
            .parse::<u32>()
            .map_err(|_| "Invalid major version".to_string())?;
        let minor = version_parts[1]
            .parse::<u32>()
            .map_err(|_| "Invalid minor version".to_string())?;
        let patch = version_parts[2]
            .parse::<u32>()
            .map_err(|_| "Invalid patch version".to_string())?;

        let label = parts.last().cloned().map(|s| s.to_string());

        let rc_number = if let Some(label) = &label {
            if let Some(stripped) = label.strip_prefix("rc.") {
                stripped.parse::<u32>().ok()
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            prefix: prefix_and_version.0,
            major,
            minor,
            patch,
            label,
            rc_number,
        })
    }

    fn increment(&self, scope: Option<&str>, option: Option<&str>) -> Result<Self, String> {
        let mut new_version = self.clone();

        match scope {
            Some(SCOPE_MAJOR) => {
                new_version.major += 1;
                new_version.minor = 0;
                new_version.patch = 0;
                new_version.label = None;
            }
            Some(SCOPE_MINOR) => {
                new_version.minor += 1;
                new_version.patch = 0;
                new_version.label = None;
            }
            Some(SCOPE_PATCH) => {
                new_version.patch += 1;
                new_version.label = None;
            }
            None => {}
            _ => {
                return Err(
                    "Invalid scope. Valid scopes are: major, minor, patch, and option".to_string(),
                )
            }
        }

        match option {
            Some(OPT_ALPHA) => {
                new_version.label = Some(OPT_ALPHA.to_string());
                new_version.rc_number = None;
            }
            Some(OPT_BETA) => {
                new_version.label = Some(OPT_BETA.to_string());
                new_version.rc_number = None;
            }
            Some(OPT_RC) => {
                let new_rc_number = new_version.rc_number.unwrap_or(0) + 1;
                let new_label = format!("{}.{}", OPT_RC, new_rc_number);

                new_version.rc_number = Some(new_rc_number);
                new_version.label = Some(new_label);
            }
            None => {}
            _ => {
                return Err(
                    "Invalid option. Valid scopes are: alpha, beta, rc, or just left it empty"
                        .to_string(),
                );
            }
        }

        Ok(new_version)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        let label = if let Some(label) = &self.label {
            if label == OPT_RC {
                format!("-rc.{}", self.rc_number.unwrap_or(0))
            } else {
                format!("-{}", label)
            }
        } else {
            String::new()
        };
        let prefix = if let Some(prefix) = &self.prefix {
            format!("{}-", prefix)
        } else {
            String::new()
        };

        write!(f, "{}{}{}", prefix, version, label)
    }
}

fn is_semver(version_str: &str) -> bool {
    SemverVersion::parse(version_str).is_ok()
}

fn get_latest_git_tag(
    repo: &Repository,
    prefix: Option<&str>,
    option: Option<&str>,
) -> Result<String, Error> {
    let tags = repo.tag_names(None)?;

    let filtered_tags: Vec<&str> = tags
        .iter()
        .flatten()
        .filter(|tag| {
            if let Some(prefix) = prefix {
                tag.starts_with(prefix)
            } else {
                is_semver(tag.split('-').collect::<Vec<&str>>()[0])
            }
        })
        .collect();

    if filtered_tags.is_empty() {
        let mut tag = "0.0.0".to_string();
        if let Some(prefix) = prefix {
            tag = format!("{}-{}", prefix, tag);
        }
        if let Some(option) = option {
            tag = format!("{}-{}", tag, option);
        }
        Ok(tag.to_string())
    } else {
        let tag = filtered_tags.last().unwrap().to_string();
        Ok(tag.to_string())
    }
}

fn create_git_tag(repo: &Repository, tag: &str) -> Result<(), Error> {
    let reference = repo
        .head()?
        .resolve()?
        .target()
        .ok_or_else(|| Error::from_str("Cannot resolve HEAD"))?;
    let commit = repo.find_commit(reference)?;
    let object = commit.as_object();

    repo.tag_lightweight(tag, object, false)?;
    println!("Tag '{}' created successfully", tag);

    Ok(())
}

fn main() {
    let args = Args::parse();

    let scope = args.scope;
    let option = args.option;
    let prefix = args.prefix;
    let dry_run = args.dry_run;

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("The current directory is not a git repository: {}", e);
            process::exit(1);
        }
    };

    let current_version = match get_latest_git_tag(&repo, prefix.as_deref(), option.as_deref()) {
        Ok(tag) => tag,
        Err(e) => {
            eprintln!("Error fetching latest tag: {}", e);
            process::exit(1);
        }
    };

    match Version::parse(&current_version) {
        Ok(version) => {
            let new_version = match version.increment(scope.as_deref(), option.as_deref()) {
                Ok(new_version) => new_version,
                Err(e) => {
                    eprintln!("Error incrementing version: {}", e);
                    process::exit(1);
                }
            };

            let new_version_str = new_version.to_string();

            if dry_run {
                println!("Latest version: '{}'", current_version);
                println!("New version   : '{}'", new_version_str);
            } else if let Err(e) = create_git_tag(&repo, &new_version_str) {
                eprintln!("Error creating tag: {}", e);
                process::exit(1);
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}
