# Dependency Updater for GitHub Actions Workflows
This can parse a GitHub Actions `.yml` file in a target folder 
and output the file contents with updated version references for the external dependencies used in the workflow script.

This package provides a sample GHA `.yml` workflow file which can be used to test the script.

### Notes 
This tool is relying on string operations to discover mismatches between the current version of an Action in the `.yml` file and its latest version on GitHub. It's important to note that some GitHub Actions on the marketplace are referenced in workflows with a singular number for the version, whereas their release versions in GitHub might look different (e.g. GH Release `v2.4.5` vs Dependency Reference `@v2`).

Also, the tool is limited when it comes to understanding context in the `.yml` files, hence all ocurrences of an 'outdated' Actions `uses` statement will be replaced with an updated reference.

# Prerequisites
1. Rust
2. Cargo
3. A target `.yml` Actions workflow file

# Usage

The following commands have to be executed from the `gh_actions_dep_updater` folder.

1. `cargo run -- c "./sample_workflow.yml"` (to output updated file contents to the console)
2. `cargo run -- w "./sample_workflow.yml"` (to output updated file contents to the input workflow file)