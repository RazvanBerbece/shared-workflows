use std::fs;
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
    let mut urls: Vec<String> = vec![]; // list of Marketplace URLs
    let mut latest_versions: Vec<String> = vec![]; // same size as dependencies; associated

    let mut output_yaml = yaml.clone();

    // Find the dependencies in the yml content (i.e strings like actions/checkout@v1, mathieudutour/github-tag-action@v1, docker/login-action@v1)
    let dependency_pattern = Regex::new(r"[a-zA-Z0-9-]+/[a-zA-Z0-9-]+(/[a-zA-Z0-9-]+)?@v?[0-9]+(\.[0-9]+){0,2}").unwrap();
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

        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let sanitised_action = action.split("@").next().unwrap();

        // Find the GitHub Marketplace URL from the source repository page 
        // (as all Actions repositories contain a hyperlink to the Marketplace homepage for the workflow)
        let source_repo_response = reqwest::blocking::get(format!("https://github.com/{}/{}", author, sanitised_action))?.text()?;
        let marketplace_url_pattern = Regex::new(r#"<a [^>]*href="/([^"]+)"[^>]*>.*?<span [^>]*>.*?View on Marketplace.*?</span>.*?</a>"#).unwrap();
        for cap in marketplace_url_pattern.captures_iter(source_repo_response.as_str()) {
            let marketplace_url = format!("https://github.com/{}", cap.get(1).unwrap().as_str());
            urls.push(marketplace_url);
            break;
        }
    }

    // For each repository by URL, retrieve the latest published release and version 
    let urls_iter = urls.iter();
    for url in urls_iter {

        let target_url = url;

        println!("Checking {} for new versions of the dependency...", target_url);

        let github_http_result = reqwest::blocking::get(target_url).unwrap();
        let status = github_http_result.status().as_u16();
        let response_text = github_http_result.text()?;
        if status == 404 {
            println!("Status 404 for: {}. Skipping.", target_url);
            continue;
        }

        let release_version_pattern = Regex::new(r#"<span class="[^"]*mx-2[^"]*">([^<]+)</span>"#).unwrap();
        for cap in release_version_pattern.captures_iter(response_text.as_str()) {
            let latest_version = cap.get(1).unwrap().as_str();
            latest_versions.push(latest_version.to_string());
            // only consider the first result, which is the latest version
            break;
        }
    }

    let mut index = 0;
    let deps_iter = dependencies.iter();
    for current_dependency in deps_iter {
        
        // Skip empty statically allocated elements
        if current_dependency.len() == 0 {
            continue;
        }

        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let mut action_tokens = action.split("@");
        let action_name = action_tokens.next().unwrap();
        let current_version = action_tokens.next().unwrap();
        
        let latest_version_for_dependency = latest_versions.get(index).unwrap();

        if current_version != latest_version_for_dependency {
            let current_ocurrence = format!("{action_name}@{current_version}");
            let updated_ocurrence = format!("{action_name}@{latest_version_for_dependency}");

            println!("Update discovered for {author}/{action_name} : {current_ocurrence} -> {updated_ocurrence}");

            output_yaml = output_yaml.replacen(current_ocurrence.as_str(), updated_ocurrence.as_str(), 256);
        }

        index += 1;
    }

    return Ok(output_yaml.to_owned());

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