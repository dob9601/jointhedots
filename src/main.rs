use std::{fs, process::Command};

use clap::{Arg, App};
use git2::Repository;
use serde_yaml::Mapping;
use std::path::PathBuf;

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
        let mut url = GITHUB_URL.to_string();
        url.push_str(value);

        fs::remove_dir_all(REPO_DIR).ok();
        fs::create_dir_all(REPO_DIR).expect("Could not create temporary repo directory");

        let repo = match Repository::clone(url.as_str(), REPO_DIR) {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to open: {}", e)
        };

        let mut repo_path = REPO_DIR.to_string();
        repo_path.push_str(MANIFEST_FILENAME);


        let manifest_file = fs::File::open(repo_path).expect("Unable to find a manifest file. Please ensure this is a valid jointhedots repository.");
        let data: Mapping = serde_yaml::from_reader(manifest_file).expect("Could not read YAML from manifest file");

        for (key, value) in data.iter() {
            if let serde_yaml::Value::Mapping(data) = value {
                let key_name = key.as_str().expect("JTD top level keys must be strings");

                println!("{:?}", data);
                if let Some(serde_yaml::Value::Sequence(stages)) = data.get(&serde_yaml::Value::String("pre_install".to_string())) {
                    println!("Running pre-install steps for {}", key_name);
                    for stage in stages {
                        let output = Command::new(stage.as_str().expect("'pre_install' members must be strings")).output().unwrap();
                        println!("{}", String::from_utf8_lossy(&output.stdout))
                    }
                }

                println!("Installing configuration files for {}", key_name);
                let mut config_path = REPO_DIR.to_string();
                config_path.push_str(data.get(&("file".into())).expect("Missing 'file' key in JTD entry").as_str().expect("'file' key must be a string"));
                let target_path = shellexpand::tilde(data.get(&("target".into())).expect("Missing 'target' key in JDT entry").as_str().expect("'target' key must be a string"));

                let mut target_folder: PathBuf = target_path.to_string().into();
                target_folder.pop();

                println!("{}", target_folder.to_str().expect("x"));
                println!("{}", target_path);
                fs::create_dir_all(target_folder).expect(format!("Could not create target directory for config {}", key_name).as_str());
                fs::copy(config_path, target_path.to_string()).expect(format!("Could not copy to target directory for config {}", key_name).as_str());
            }
        }
    }
}
