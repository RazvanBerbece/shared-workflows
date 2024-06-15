use std::fs::{self};
use regex::Regex;

fn main() {
    println!("Running the GH Actions dependency updater script...");

    let filepath = "./.github/workflows/sample_workflow.yml";
    let yaml = read_workflow_file(filepath);

    extract_dependencies(yaml.as_str());
}

fn read_workflow_file(filepath: &str) -> String {
    
    let yaml_contents = fs::read_to_string(filepath)
        .expect("Should have been able to read the file");

    return yaml_contents

}

fn extract_dependencies(yaml: &str) {

    let mut dependencies: Vec<&str> = vec![];
    let mut urls: Vec<String> = vec![];

    // Find the dependencies in the yml content (i.e strings like actions/checkout@v1, mathieudutour/github-tag-action@v1, docker/login-action@v1)
    let dependency_pattern = Regex::new(r"[a-zA-Z0-9-]+/[a-zA-Z0-9-]+(/[a-zA-Z0-9-]+)?@v[0-9]+(\.[0-9]+){0,2}").unwrap();
    for cap in dependency_pattern.captures_iter(yaml) {
        dependencies.push(cap.get(0).unwrap().as_str())
    }

    // For each dependency, generate the URL that points to their GitHub source repository
    dependencies.into_iter().for_each(|current_dependency: &str| {

        // Skip empty statically allocated elements
        if current_dependency.len() == 0 {
            return;
        }

        // Process dependency
        let tokens = current_dependency.split("/");
        let collection: Vec<&str> = tokens.collect();
        let author = collection[0];
        let action = collection[1];
        let sanitised_action = action.split("@").next().unwrap();

        let action_src_url = format!("https://github.com/{}/{}", author, sanitised_action);
        urls.push(action_src_url.to_owned());
    });

    // For each repository by URL, retrieve the latest published release and version 
    urls.into_iter().for_each(|current_dependency_url: String| {
        println!("{current_dependency_url}");
    });

}