use std::{fs, process::Command};

use clap::{Arg, App};
use git2::Repository;
use serde_yaml::Mapping;

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
        let data: Mapping = serde_yaml::from_reader(manifest_file).expect("Could not read YAML from manifest file");

        for (key, value) in data.iter() {
            println!("{:?}", key);
            println!("{:?}", value);
            if let serde_yaml::Value::Mapping(data) = value {
                let key_name = key.as_str().expect("JTD top level keys must be strings");

                if let Some(serde_yaml::Value::Sequence(stages)) = data.get(&serde_yaml::Value::String("pre_install".to_string())) {
                    for stage in stages {
                        Command::new(stage.as_str().expect("'pre_install' members must be strings")).output().unwrap();
                    }
                }
                println!("Running pre-install steps for {}", key_name);

                println!("Installing configuration files for {}", key_name);
                let filename = data.get(&("file".into())).expect("Missing 'file' key in JTD entry").as_str().expect("'file' key must be a string");
                let target_path = data.get(&("target".into())).expect("Missing 'target' key in JDT entry").as_str().expect("'target' key must be a string");

                let path_length = target_path.matches('/').count();
                let target_folder = target_path.clone().split_at(path_length - 1).0;
                fs::create_dir_all(target_folder).expect(format!("Could not create target directory for config {}", key_name).as_str());
                fs::copy(filename, target_path).expect(format!("Could not copy to target directory for config {}", key_name).as_str());
            }
        }
    }
}
