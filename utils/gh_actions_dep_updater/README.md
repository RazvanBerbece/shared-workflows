# Dependency Updater for GitHub Actions Workflows
This can parse a GitHub Actions `.yml` file in a target folder 
and output the file contents with updated version references for the external dependencies used in the workflow script.

This package provides a sample GHA `.yml` workflow file which can be used to test the script.

### Notes 
This tool is relying on string operations to discover mismatches between the current version of an Action in the `.yml` file and its latest version on GitHub (Marketplace or the source repository, using whichever is available).

Also, the tool is limited when it comes to understanding context in the `.yml` files, hence sometimes unexpected ocurrences of an 'outdated' Actions `uses` statement will be replaced with an updated reference. 

It's recommended to test the workflow with the updated references before releasing if possible.

# Prerequisites
1. Rust
2. Cargo
3. A target `.yml` Actions workflow file

# Usage

The following commands have to be executed from the `gh_actions_dep_updater` folder.

1. `cargo run -- c "./sample_workflow.yml"` (to output updated file contents to the console)
2. `cargo run -- w "./sample_workflow.yml"` (to output updated file contents to the input workflow file)