use std::fs::{self};
use regex::Regex;
use std::error::Error;

fn main() {
    println!("Running the GH Actions dependency updater script...");

    let filepath = "./.github/workflows/sample_workflow.yml";
    let yaml = read_workflow_file(filepath);

    let _extract_dependencies = extract_dependencies(yaml.as_str());
}

fn read_workflow_file(filepath: &str) -> String {
    
    let yaml_contents = fs::read_to_string(filepath)
        .expect("Should have been able to read the file");

    return yaml_contents

}

fn extract_dependencies(yaml: &str) -> Result<(), Box<dyn Error>> {

    let mut dependencies: Vec<&str> = vec![];
    let mut urls: Vec<String> = vec![];
    let mut latest_versions: Vec<String> = vec![]; // same size as dependencies; associated

    // Find the dependencies in the yml content (i.e strings like actions/checkout@v1, mathieudutour/github-tag-action@v1, docker/login-action@v1)
    let dependency_pattern = Regex::new(r"[a-zA-Z0-9-]+/[a-zA-Z0-9-]+(/[a-zA-Z0-9-]+)?@v[0-9]+(\.[0-9]+){0,2}").unwrap();
    for cap in dependency_pattern.captures_iter(yaml) {
        dependencies.push(cap.get(0).unwrap().as_str())
    }

    // For each dependency, generate the URL that points to their GitHub source repository
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

        let action_src_url = format!("https://github.com/{}/{}/releases", author, sanitised_action);
        urls.push(action_src_url.to_owned());
    }

    // For each repository by URL, retrieve the latest published release and version 
    let urls_iter = urls.iter();
    for url in urls_iter {

        // println!("{url}");

        let github_http_result = reqwest::blocking::get(url)?.text()?;

        let release_version_pattern = Regex::new(r#"<a href="[^"]+/([^/"]+)"#).unwrap();
        for cap in release_version_pattern.captures_iter(github_http_result.as_str()) {
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

        // Process dependency
        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let mut action_tokens = action.split("@");
        let action_name = action_tokens.next().unwrap();
        let action_version = action_tokens.next().unwrap();

        let current_uses = format!("{author}/{action_name}@{action_version}");
        
        let latest_dependency = latest_versions.get(index).unwrap();

        let latest_uses = format!("{author}/{action_name}@{latest_dependency}");

        println!("Installed: {current_uses}");
        println!("Available: {latest_uses}");

        index += 1;
    }

    Ok(())

}