use std::{error::Error, fs::{self, File}, process::Command, path::Path, collections::HashMap};

use clap::{Arg, App};
use git2::Repository;
use serde::{Serialize, Deserialize};

const GITHUB_URL: &str = "https://github.com/";
const MANIFEST_FILENAME: &str = "jtd.yaml";
const REPO_DIR: &str = "/tmp/jtd/"; 

const MANIFEST_PATH: &str = "/tmp/jtd/jtd.yaml"; 

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    #[serde(flatten)] 
    data: HashMap<String, Dotfile>
}

impl IntoIterator for Config {
    type Item = (String, Dotfile);

    type IntoIter = std::collections::hash_map::IntoIter<String, Dotfile>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Dotfile {
    file: String,
    target: Box<Path>,
    pre_install: Option<Vec<String>>,
    post_install: Option<Vec<String>>
}

fn main() -> Result<(), Box<dyn Error>> {
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

        if Path::new(REPO_DIR).exists() {
            fs::remove_dir_all(REPO_DIR).expect("Could not clear temporary directory");
        }
        fs::create_dir_all(REPO_DIR).expect("Could not create temporary directory");

        let _repo = match Repository::clone(url.as_str(), REPO_DIR) {
            Ok(repo) => repo,
            Err(e) => panic!("Failed to open: {}", e)
        };

        let config: Config = serde_yaml::from_reader(File::open(MANIFEST_PATH)?).unwrap();

        for (dotfile_name, dotfile) in config.into_iter() {
            println!("Commencing install for {}", dotfile_name);

            println!("Running pre-install steps");
            if let Some(pre_install) = &dotfile.pre_install {
                run_command_vec(&pre_install)?;

            }

            println!("Installing config file");
            fs::copy(REPO_DIR.to_owned() + &dotfile.file, &dotfile.target)?;

            println!("Running post-install steps");
            if let Some(post_install) = &dotfile.post_install {
                run_command_vec(&post_install)?;
            }
        }
    }
    Ok(())
}

pub fn run_command_vec(command_vec: &[String]) -> Result<(), Box<dyn Error>>{
    for command in command_vec.iter() {
        println!("{}", command);
        let command_vec: Vec<&str> = command.split(' ').collect();
        Command::new(command_vec[0])
            .args(&command_vec[1..])
            .spawn()?;
    }
    Ok(())
}

        /*
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
                fs::create_dir_all(target_folder).map_err(|err| format!("Could not create target directory for config {}: {:?}", key_name, err))?;
                fs::copy(config_path, target_path.to_string()).map_err(|err| format!("Could not copy to target directory for config {}: {:?}", key_name, err))?;
            }
        }
        */
