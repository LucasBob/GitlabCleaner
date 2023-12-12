# GitLab Repo Cleaner

[![Latest release](https://img.shields.io/github/v/release/LucasBob/GitlabCleaner)](https://github.com/LucasBob/GitlabCleaner/releases/latest) 

## Overview

The GitLab Repo Cleaner is a command-line tool developed in Rust that utilizes the GitLab API to assist in cleaning up repositories. It is designed to simplify and automate the process of managing and maintaining GitLab repositories by performing various cleaning tasks.

### Technical Details

- **Actor Model:** The GitLab Repo Cleaner leverages the Actor model for concurrency, allowing efficient and scalable execution of cleanup tasks. I rapidly got rid of the `actix` crate that implements this architecture in favor of a much more lightweight one... [tiny-tokio-actor](https://github.com/fdeantoni/tiny-tokio-actor), which fixed issues I was having with `reqwest`...
- **Command-Line Interface:** The tool uses CLAP (Command Line Argument Parser) to provide a user-friendly interface for handling command-line arguments and options.
- **No unit tests yet** Sorry about that... I'm learning here...

## Features

- **Jobs cleanup:** Delete jobs that are older than a given number of days (and all attached logs & artifacts)
- **TODO Branches cleanup:** Delete merged branches, stale branches.
- **TODO Merge requests cleanup:** Delete stale merge requests & attached branches.
- **TODO Issues cleanup:** Close old issues.
- **TODO Tags cleanup:** Close old tags & releases.
- **TODO Dry run:** Implement a dry run feature 'cause it's all scary...
- **?**...

## Installation

Either get the version that suits you here : [![Latest release](https://img.shields.io/github/v/release/LucasBob/GitlabCleaner)](https://github.com/LucasBob/GitlabCleaner/releases/latest) 
Or build it for yourself (for fun or for contributing <3) by following the steps below:

1. Ensure you have Rust installed. If not, install Rust using [rustup](https://rustup.rs/).
2. Clone this repository: `git clone https://github.com/your_username/gitlab-repo-cleaner.git`
3. Navigate to the project directory: `cd gitlab-repo-cleaner`
4. Build the project: `cargo build --release`
5. Find the executable in the `target/release` directory.

Steps 4 & 5 car be replaced with the execution of `cargo install --path .` which basically installs the program on your machine and allows direct `gitlab-cleaner` commands to be run. 

## Configuration

To authenticate with GitLab, ensure you have a personal access token with the necessary permissions (e.g., api, read_repository, write_repository). Set the token as an environment variable (GITLAB_TOKEN).
Please make sure to define the url for your gitlab instance using `GITLAB_URL` environment variable as well; It should look like `https://your.company.domain.com/api/v4`.


## Usage

To use the GitLab Repo Cleaner, execute the following command:`gitlab-cleaner <arguments>`
Please see the below arguments table: 

| Argument                | Short | Long           | Description                                                            | Default Value |
|-------------------------|-------|----------------|------------------------------------------------------------------------|---------------|
| `project`               | `-p`  | `--project`    | The name of the project to search for.                                 |     |
| `group`               | `-g`  | `--group`    | The name of the group in which to look for the prohect                                 |     |
| `target`                | `-t`  | `--target`     | The target component(s) of the project to clean. For now, it can only be `jobs` | `jobs`        |
| `expiration_in_days`    |       |                | The expiration date of the component(s) to clean (in days).            | `365`         |

