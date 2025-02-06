# SemTag

`semtag` is a command-line tool designed to automatically bump semantic versioning (SemVer) for projects. It determines the latest version and increments it based on the specified release type. The tool supports `major`, `minor`, and `patch` version updates, as well as optional pre-release identifiers.

```shell
$ semtag -h
A CLI app to bump semver tag

Usage: semtag [OPTIONS]

Options:
  -s, --scope <SCOPE>    The scope of the version: major, minor, or patch
  -o, --option <OPTION>  The option to be used: alpha, beta, rc, or just left it empty
  -p, --prefix <PREFIX>  The prefix to be used: prod, stage, sandbox, dev, etc
  -d, --dry-run          Dry run mode, do not create a tag
  -h, --help             Print help
  -V, --version          Print version
```

## Key Behaviors

### Basic Version Bumping

Running `semtag` with the `-s` flag and specifying `patch`, `minor`, or `major` updates the latest version accordingly:
- `-s patch` increases the third digit (e.g., 0.0.0 → 0.0.1).
- `-s minor` increases the second digit (e.g., 0.0.0 → 0.1.0).
- `-s major` increases the first digit (e.g., 0.0.0 → 1.0.0).

### Pre-release Identifiers

Using the -o flag allows appending a pre-release identifier (e.g., `alpha`, `beta`, `rc`). When a pre-release tag is used, the versioning still follows SemVer rules:
- `-s patch -o alpha` results in 0.0.1-alpha.
- `-s minor -o beta` results in 0.1.0-beta.
- `-s major -o rc` results in 1.0.0-rc.1, where rc.1 indicates the first release candidate.

### Incrementing Release Candidate (rc) Numbers

If an rc version already exists for a specific scope and prefix, semtag automatically increments the RC number:
- If the latest version is `1.0.0-rc.1` and `semtag -o rc` is executed again, the new version becomes `1.0.0-rc.2`.

### Dry Run Mode (-d)

The `-d` flag performs a dry run, displaying the computed version changes without actually applying them.

## Inspiration

This project was inspired by the following repositories:

- [https://github.com/nico2sh/semtag](https://github.com/nico2sh/semtag)
