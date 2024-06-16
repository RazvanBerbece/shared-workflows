# Dependency Updater for GitHub Actions Workflows
This can parse GH Actions .yml files in a target `.github/workflows/` folder 
and generate a file with the references of the external dependencies updated to their latest versions as published on GitHub.

Package provides a sample `.github` folder with a sample workflow which can be used to test the script.

# Prerequisites
1. Rust
2. Cargo

# Usage

The following commands have to be executed from the `gh_actions_dep_updater` folder.

1. `cargo run`