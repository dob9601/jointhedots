use std::fs;

use clap::{Arg, App};
use git2::Repository;

const GITHUB_URL: &str = "https://github.com/";
const MANIFEST_FILENAME: &str = "jtd.yaml";
const REPO_DIR: &str = "/tmp/jtd/"; 

fn main() {
    let matches = App::new("jointhedots")
        .version("0.0.1")
        .author("Daniel O. <dob9601@gmail.com>")
        .about("Get your dotfiles on a new system with speed.")
        .arg(Arg::new("repository")
            .about("The GitHub repository to pull dotfiles from.")
            .required(true)
            .index(1)
        )
        .get_matches();

    if let Some(value) = matches.value_of("repository") {
        let mut url = GITHUB_URL.to_string().clone();
        url.push_str(value);

        fs::remove_dir_all(REPO_DIR).ok();
        fs::create_dir_all(REPO_DIR).expect("Could not create temporary repo directory");

        let repo = match Repository::clone(url.as_str(), REPO_DIR) {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to open: {}", e)
        };

        let mut filepath = REPO_DIR.to_string();
        filepath.push_str(MANIFEST_FILENAME);


        let manifest_file = fs::File::open(filepath).expect("Unable to find a manifest file. Please ensure this is a valid jointhedots repository.");
        let data: serde_yaml::Mapping = serde_yaml::from_reader(manifest_file).expect("Could not read YAML from manifest file");

        println!("{:?}", data)
    }
}
