# Dependency Updater for GitHub Actions Workflows
This can parse a GH Actions .yml file in a `.github/workflows/` folder 
and output the file with the references of the dependencies updated to their latest versions, as published on GitHub.

Package provides a sample `.github` folder with a sample workflow which can be used to test the script.

### Notes 
This tool is relying on string operations to discover mismatches between the current version and the versions on GitHub. 

Also, it seems that some GitHub Actions on the marketplace are referenced in workflows with a singular number for the version `action@v1`,
whereas their release versions in GitHub might look different (e.g. GH Release `v2.4.5` vs Dependency Reference `@v2`)

# Prerequisites
1. Rust
2. Cargo

# Usage

The following commands have to be executed from the `gh_actions_dep_updater` folder.

1. `cargo run`