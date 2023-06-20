use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs::canonicalize;
use std::{env,fs};
use std::path::{PathBuf, Path};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
/// A set of tools used to manipulate a systems paths
struct Args {
    #[arg(short, long)]
    /// This is the name of the path that will be operated on (default: PATH)
    name: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Add a folder to the path
    Add {
        /// This is the directory you wish to push onto the path
        dir: PathBuf,
    },
    /// Removes a folder from the path
    Remove {
        /// This is the directory you wish to remove
        dir: Option<PathBuf>,
        /// This is the index at which remove the directory
        index: Option<usize>,
    },
    /// find files that are on the current path
    Find {
        /// This is the regex pattern you wish to use to filter out the results you want
        pattern: Option<String>,
    },
}

struct SystemPath {
    directories: Vec<String>,
}

impl SystemPath {
    pub fn seporator() -> String {
        #[cfg(target_os = "windows")]
        return String::from(";");
        return String::from(":");
    }

    pub fn from(path_name: String) -> Result<Self, String> {
        match env::var(path_name.clone()) {
            Ok(raw_path) => {
                let path_seporator = Self::seporator();
                let path: Vec<String> = raw_path
                    .split(&path_seporator)
                    .map(str::to_string)
                    .collect();

                return Ok(Self {
                    directories: path,
                });
            }

            Err(error) => {
                return Err(format!(
                    "Unable to get path {}. becuase {}",
                    path_name, error
                ));
            }
        }
    }

    pub fn add(&mut self, directory: &PathBuf) -> Result<(), String> {
        if !directory.is_dir() {
            return Err(format!("path provided is not a directory"));
        }
        let full_directory: PathBuf;

        match canonicalize(directory.as_path()) {
            Ok(fullpath) => {
                full_directory = fullpath;
            }

            Err(_) => {
                return Err(format!("Unable to convert {} to a full path",directory.display()))
            }
        }

        let directory_str = match full_directory.to_str() {
            Some(str) => String::from(str),
            None => return Err(String::from("Can not convert path to string")),
        };

        self.directories.push(directory_str);
        
        println!("{}",self.to_string()?);
        return Ok(());
    }

    pub fn remove(&mut self, directory_to_remove: &PathBuf) -> Result<(), String> {
        if !directory_to_remove.is_dir() {
            return Err(format!("path provided is not a directory"));
        }
        let directory_to_remove_str = match directory_to_remove.to_str() {
            Some(str) => String::from(str),
            None => return Err(String::from("Can not convert path to string")),
        };
        let mut index_to_remove: Option<usize> = None;
        for (index, directory) in self.directories.iter().enumerate() {
            if directory.eq(&directory_to_remove_str) {
                index_to_remove = Some(index);
            }
        }

        if let Some(index) = index_to_remove {
            return self.remove_at(&index);
        }

        return Err(format!("The given directory is not on the path"));
    }

    pub fn remove_at(&mut self, index: &usize) -> Result<(), String> {
        if self.directories.len() < index.clone() {
            return Err(format!(
                "Attempt to remove at index {} however the length of the path is only {}",
                index,
                self.directories.len()
            ));
        }
        self.directories.remove(index.clone());
        println!("{}",self.to_string()?);
        Ok(())
    }

    pub fn remove_either(
        &mut self,
        dir: &Option<PathBuf>,
        index: &Option<usize>,
    ) -> Result<(), String> {
        if let Some(directory) = dir {
            return self.remove(directory);
        } else if let Some(_index) = index {
            return self.remove_at(_index);
        }

        return Err(format!("No folder has been set to be removed"));
    }

    fn get_file_matches(pattern: Regex, directory: &String) -> Vec<String> {
        let paths = fs::read_dir(directory).unwrap();
        let mut results = Vec::new();

        for path in paths {
            let dir_entry = path.unwrap();
            let osfilename =  dir_entry.file_name();
            
            let file_name = match osfilename.to_str() {
                Some(text) => text,
                None => "",
            };

            if pattern.is_match(file_name) {
                results.push(String::from(file_name));
            }
        }
        results
    }

    pub fn find_and_print(&mut self, pattern: &Option<String>) {
        
        let name = pattern.clone().unwrap_or(String::new());
        let regex_pattern = Regex::new(&name.as_str()).unwrap();

        for directory in self.directories.iter() {
            let dir_matches = Self::get_file_matches(regex_pattern.clone(), directory);
            for file in dir_matches {
                let file_path = Path::new(directory);
                let pathbuffer = file_path.join(file);
                println!("{}",pathbuffer.display());
            }
        }
    }

    pub fn to_string(&self) -> Result<String, String> {
        let mut directory_iterator = self.directories.iter();
        let mut return_value = match directory_iterator.next() {
            Some(text) => String::from(text),
            None => String::new()
        };

        for directory in directory_iterator {
            return_value += Self::seporator().as_str();
            return_value += directory.as_str();
        }
        Ok(return_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from() {
        #[cfg(target_os = "windows")]
        let test_paths = vec!["C:\\Users;C:\\Windows"];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec!["/bin:/lib:/temp"];

        for test_path in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            assert_eq!(sys_path.to_string().unwrap(),test_path);

        }
    }

    #[test]
    fn add() {
        #[cfg(target_os = "windows")]
        let test_paths = vec![("C:\\Users;C:\\Windows","C:\\Program Files","C:\\Users;C:\\Windows;C:\\Program Files")];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec![("/bin:/lib:/temp","/proc","/bin:/lib:/temp:/proc")];

        for (test_path,added_dir, expected_path) in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let mut sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            let path = PathBuf::from(added_dir);
            sys_path.add(&path).unwrap();
            
            assert_eq!(sys_path.to_string().unwrap(),expected_path);

        }
    }

    #[test]
    fn remove() {
        #[cfg(target_os = "windows")]
        let test_paths = vec![("C:\\Users;C:\\Windows","C:\\Program Files","C:\\Users;C:\\Windows;C:\\Program Files")];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec![("/bin:/lib:/temp:/proc","/proc","/bin:/lib:/temp")];

        for (test_path,added_dir, expected_path) in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let mut sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            let path = PathBuf::from(added_dir);
            sys_path.remove(&path).unwrap();
            
            assert_eq!(sys_path.to_string().unwrap(),expected_path);

        }
    }

    #[test]
    fn remove_at() {
        #[cfg(target_os = "windows")]
        let test_paths = vec![("C:\\Users;C:\\Windows","C:\\Program Files","C:\\Users;C:\\Windows;C:\\Program Files")];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec![("/bin:/lib:/temp:/proc","/bin:/lib:/temp")];

        for (test_path, expected_path) in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let mut sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            sys_path.remove_at(&3).unwrap();
            
            assert_eq!(sys_path.to_string().unwrap(),expected_path);

        }
    }

    #[test]
    fn remove_either() {
        #[cfg(target_os = "windows")]
        let test_paths = vec![("C:\\Users;C:\\Windows","C:\\Program Files","C:\\Users;C:\\Windows;C:\\Program Files")];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec![("/bin:/lib:/temp:/proc","/bin:/lib:/temp")];

        for (test_path, expected_path) in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let mut sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            sys_path.remove_either(&None, &Some(3)).unwrap();
            
            assert_eq!(sys_path.to_string().unwrap(),expected_path);

        }

        #[cfg(target_os = "windows")]
        let test_paths = vec![("C:\\Users;C:\\Windows","C:\\Program Files","C:\\Users;C:\\Windows;C:\\Program Files")];
        #[cfg(not(target_os = "windows"))]
        let test_paths = vec![("/bin:/lib:/temp:/proc","/proc","/bin:/lib:/temp")];

        for (test_path,added_dir, expected_path) in test_paths {
            env::set_var("_TEST_CAM_", test_path);

            let mut sys_path = SystemPath::from(String::from("_TEST_CAM_")).unwrap();
            let path = PathBuf::from(added_dir);
            sys_path.remove_either(&Some(path),&None).unwrap();
            
            assert_eq!(sys_path.to_string().unwrap(),expected_path);

        }
    }

    #[test]
    fn get_file_matches() {

    }

    #[test]
    fn find_and_print() {

    }

}

fn main() -> Result<(), String> {
    let args = Args::parse();
    let path_name = args.name.unwrap_or(String::from("PATH"));
    let mut path = SystemPath::from(path_name)?;

    match &args.command {
        Command::Add { dir } => path.add(dir)?,
        Command::Remove { dir, index } => path.remove_either(dir, index)?,
        Command::Find { pattern } => path.find_and_print(pattern),
    }

    Ok(())
}
