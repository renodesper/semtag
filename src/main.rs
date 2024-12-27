use git2::{Error, Repository};
use std::env;
use std::process;

#[derive(Debug, Clone)]
struct Version {
    prefix: Option<String>,
    major: u32,
    minor: u32,
    patch: u32,
    label: Option<String>,
    rc_number: Option<u32>,
}

impl Version {
    fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('-').collect();

        let prefix_and_version = if parts[0].contains('.') {
            (None, parts[0])
        } else {
            let mut segments = parts[0].splitn(2, '.');
            (
                segments.next().map(|s| format!("-{}", s)),
                segments.next().unwrap_or(parts[0]),
            )
        };

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

        let label = parts.get(1).cloned().map(|s| s.to_string());
        let rc_number = if let Some(label) = &label {
            if label.starts_with("rc.") {
                label[3..].parse::<u32>().ok()
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

    fn increment(&self, scope: &str, option: Option<&str>) -> Result<Self, String> {
        if scope.is_empty() {
            return Err(
                "Usage: increment(scope: &str, option: Option<&str>)\nScopes: major, minor, patch"
                    .to_string(),
            );
        }

        let mut new_version = self.clone();

        match scope {
            "major" => {
                new_version.major += 1;
                new_version.minor = 0;
                new_version.patch = 0;
            }
            "minor" => {
                new_version.minor += 1;
                new_version.patch = 0;
            }
            "patch" => {
                new_version.patch += 1;
            }
            _ => return Err("Invalid scope. Valid scopes are: major, minor, patch".to_string()),
        }

        match option {
            Some("alpha") => {
                new_version.label = Some("alpha".to_string());
                new_version.rc_number = None;
            }
            Some("beta") => {
                new_version.label = Some("beta".to_string());
                new_version.rc_number = None;
            }
            Some("candidate") => {
                new_version.label = Some("rc".to_string());
                new_version.rc_number = Some(new_version.rc_number.unwrap_or(0) + 1);
            }
            None => {
                new_version.label = None;
                new_version.rc_number = None;
            }
            _ => return Err(
                "Invalid option. Valid scopes are: alpha, beta, candidate, or just left it empty"
                    .to_string(),
            ),
        }

        Ok(new_version)
    }

    fn to_string(&self) -> String {
        let version = format!("{}.{}.{}", self.major, self.minor, self.patch);
        let label = if let Some(label) = &self.label {
            if label == "rc" {
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

        format!("{}{}{}", prefix, version, label)
    }
}

fn get_latest_git_tag(repo: &Repository, prefix: Option<&str>) -> Result<String, Error> {
    let tags = repo.tag_names(None)?;

    let filtered_tags: Vec<&str> = tags
        .iter()
        .filter_map(|t| t)
        .filter(|tag| {
            if let Some(prefix) = prefix {
                tag.starts_with(prefix)
            } else {
                true
            }
        })
        .collect();

    if let Some(tag) = filtered_tags.last() {
        Ok(tag.to_string())
    } else {
        Err(Error::from_str("No tags found in the repository"))
    }
}

fn create_git_tag(repo: &Repository, tag: &str) -> Result<(), Error> {
    let reference = repo
        .head()?
        .resolve()?
        .target()
        .ok_or_else(|| Error::from_str("Cannot resolve HEAD"))?;
    let commit = repo.find_commit(reference)?;
    let object = commit.as_object(); // Convert commit to object

    repo.tag_lightweight(tag, &object, false)?;
    println!("Tag '{}' created successfully", tag);

    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"-h".to_string()) {
        println!("Usage: {} <scope> [option] [prefix]", args[0]);
        println!("\nscopes:");
        println!("  major: Bump the major version and reset minor and patch to 0");
        println!("  minor: Bump the minor version and reset patch to 0");
        println!("  patch: Bump the patch version");
        println!("\noptions:");
        println!("  alpha: Append '-alpha' to the version");
        println!("  beta: Append '-beta' to the version");
        println!(
            "  candidate: Append '-rc.#' to the version, where # is incremented or starts at 0"
        );
        println!("\nExamples:");
        println!("  {} major alpha", args[0]);
        println!("  {} minor beta prefix", args[0]);
        println!("  {} patch candidate", args[0]);
        process::exit(0);
    }

    if args.contains(&"-v".to_string()) {
        println!("{} version 0.1.0", args[0]);
        process::exit(0);
    }

    if args.len() < 2 || args.len() > 4 {
        eprintln!("Invalid arguments. Use '-h' for usage information.");
        process::exit(1);
    }

    let scope = &args[1];
    let option = args.get(2).map(|s| s.as_str());
    let prefix = args.get(3).map(|s| s.as_str());

    let repo = match Repository::open(".") {
        Ok(repo) => repo,
        Err(e) => {
            eprintln!("Failed to open repository: {}", e);
            process::exit(1);
        }
    };

    let current_version = match get_latest_git_tag(&repo, prefix) {
        Ok(tag) => tag,
        Err(e) => {
            eprintln!("Error fetching latest tag: {}", e);
            process::exit(1);
        }
    };

    match Version::parse(&current_version) {
        Ok(version) => {
            let new_version = match version.increment(scope, option) {
                Ok(new_version) => new_version,
                Err(e) => {
                    eprintln!("Error incrementing version: {}", e);
                    process::exit(1);
                }
            };
            let new_version_str = new_version.to_string();

            if let Err(e) = create_git_tag(&repo, &new_version_str) {
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
