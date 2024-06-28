use std::fs::{self};
use regex::Regex;
use std::error::Error;
use clap::Parser;

//
// USAGE: cargo run -r -- c "./sample_workflow.yml" (to output updated file contents to console)
//        cargo run -r -- w "./sample_workflow.yml" (to output updated file contents to workflow file)
//

#[derive(Parser)]
struct Cli {
    mode: String, // w = re-write file, c = return string content
    workflow_filepath: String,
}

fn main() {

    // Parse input args (i.e mode and target files)
    let args = Cli::parse();

    println!("\nRunning the GH Actions dependency updater script for {}", args.workflow_filepath.as_str());

    // Read the workflow YAML source
    let yaml = read_workflow_file(args.workflow_filepath.as_str());

    // Run the << magic >>
    let updated_file_content = generate_updated_workflow_file(&yaml).unwrap();

    // If changes in versions are detected
    if updated_file_content != yaml {
        // Output based on given strategy
        output_updated_wofklow(
            updated_file_content.as_str(), 
            args.mode.as_str(), 
            args.workflow_filepath.as_str()
        );
    }
    else {
        println!("All dependencies in {} are up to date.", args.workflow_filepath.as_str());
    }
}

fn read_workflow_file(filepath: &str) -> String {
    
    let yaml_contents = fs::read_to_string(filepath)
        .expect("Should have been able to read the file");

    return yaml_contents

}

fn generate_updated_workflow_file(yaml: &String) -> Result<String, Box<dyn Error>> {

    let mut dependencies: Vec<&str> = vec![];
    let mut urls: Vec<(String, String)> = vec![]; // (marketplace_url, fallback_repo_url)
    let mut latest_versions: Vec<String> = vec![]; // same size as dependencies; associated

    let mut output_yaml = yaml.clone();

    // Find the dependencies in the yml content (i.e strings like actions/checkout@v1, mathieudutour/github-tag-action@v1, docker/login-action@v1)
    let dependency_pattern = Regex::new(r"[a-zA-Z0-9-]+/[a-zA-Z0-9-]+(/[a-zA-Z0-9-]+)?@v[0-9]+(\.[0-9]+){0,2}").unwrap();
    for cap in dependency_pattern.captures_iter(yaml.as_str()) {
        if !dependencies.contains(&cap.get(0).unwrap().as_str()) {
            dependencies.push(cap.get(0).unwrap().as_str());
        }
    }

    // For each dependency, generate the URL that points to their page on the GitHub Actions Marketplace
    let deps_iter = dependencies.iter();
    for current_dependency in deps_iter {

        // Skip empty statically allocated elements
        if current_dependency.len() == 0 {
            continue;
        }

        // Process dependency
        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let sanitised_action = action.split("@").next().unwrap();

        let repo_action_src_url = format!("https://github.com/{}/{}/releases", author, sanitised_action);
        let marketplace_action_src_url = format!("https://github.com/marketplace/actions/{}", sanitised_action);

        urls.push((marketplace_action_src_url, repo_action_src_url));
    }

    // For each repository by URL, retrieve the latest published release and version 
    let urls_iter = urls.iter();
    for url in urls_iter {

        let marketplace_url = &url.0;
        let repo_url = &url.1;

        println!("Checking ({}, {}) for new versions of the dependency...", marketplace_url, repo_url);

        let mut used_default_strategy = true;

        // Strategy 1 - Use the GitHub Actions Marketplace URL to get the latest version
        let github_http_result = reqwest::blocking::get(marketplace_url);
        let github_http_result_text = match reqwest::blocking::get(marketplace_url)?.error_for_status() {
            Ok(_res) => github_http_result?.text()?,
            Err(_err) => {
                // Strategy 2 - Fallback to using the GitHub repository source URL of the Action to get the latest version
                println!("Falling back to Strategy 2 for {}", repo_url);
                used_default_strategy = false;
                reqwest::blocking::get(repo_url)?.text()?
            }
        };

        if used_default_strategy {
            // Strategy 1 - Marketplace
            let release_version_pattern = Regex::new(r#"<span class="[^"]*mx-2[^"]*">([^<]+)</span>"#).unwrap();
            for cap in release_version_pattern.captures_iter(github_http_result_text.as_str()) {
                let latest_version = cap.get(1).unwrap().as_str();
                latest_versions.push(latest_version.to_string());
                // only consider the first result, which is the latest version
                break;
            }
        }
        else {
            // Strategy 2 - Repository
            let release_version_pattern = Regex::new(r#"<a href="[^"]+/([^/"]+)"#).unwrap();
            for cap in release_version_pattern.captures_iter(github_http_result_text.as_str()) {
                let latest_version = cap.get(1).unwrap().as_str();
                latest_versions.push(latest_version.to_string());
                // only consider the first result, which is the latest version
                break;
            }
        }
    }

    let mut index = 0;
    let deps_iter = dependencies.iter();
    for current_dependency in deps_iter {
        
        // Skip empty statically allocated elements
        if current_dependency.len() == 0 {
            continue;
        }

        // Process dependency
        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let mut action_tokens = action.split("@");
        let action_name = action_tokens.next().unwrap();
        let action_version = action_tokens.next().unwrap();
        
        let latest_version_for_dependency = latest_versions.get(index).unwrap();

        let current_version_depth = get_version_depth(action_version);

        let current = get_version_at_depth(action_version, current_version_depth);
        let latest = get_version_at_depth(latest_version_for_dependency, current_version_depth);

        if current != latest {
            let current_ocurrence = format!("{action_name}@{action_version}");
            let updated_ocurrence = format!("{action_name}@{latest_version_for_dependency}");

            println!("Update discovered for <{author}/{action_name}>: {current_ocurrence} -> {updated_ocurrence}");

            output_yaml = output_yaml.replacen(current_ocurrence.as_str(), updated_ocurrence.as_str(), 256);
        }

        index += 1;
    }

    return Ok(output_yaml.to_owned());

}

fn get_version_depth(version: &str) -> i8 {

    let mut depth = 0;

    for char in version.chars() {
        if char == '.' {
            depth += 1;
        }
    }

    return depth;

}

fn get_version_at_depth(version: &str, depth: i8) -> String {

    let mut output = "".to_owned();

    let mut current_depth = 0;
    let mut idx = 0;
    for char in version.chars() {

        if char == 'v' {
            continue;
        }

        if current_depth <= depth {
            output.push(char);
        }

        if char == '.' {
            current_depth += 1;
        }

        if current_depth > depth {
            if char == '.' {
                output.remove(idx);
            }
            break;
        }

        idx += 1;

    }

    // Trim last dot for cases where the target depth is before one


    return output.to_string();

}

fn output_updated_wofklow(yaml: &str, mode: &str, workflow_filepath: &str) {

    if mode == "w" {
        // Re-write the input file
        let updated_filepath = workflow_filepath;
        fs::write(updated_filepath, yaml).expect("Unable to write file");
        println!("Wrote the version updates to {}.", updated_filepath);
    }
    else if mode == "c" {
        // Return the output to the console, allowing it to be piped
        println!("{yaml}");
    }
    else {
        println!("Mode {} not supported.", mode);
    }

}