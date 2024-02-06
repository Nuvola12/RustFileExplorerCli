use walkdir::WalkDir;
use std::{ alloc::System, collections::HashMap};
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use tokio;

pub mod explorer;

fn strip_directory(path: &str) -> String{
    path
        .replace("\\", "/")
        .split('/')
        .last()
        .unwrap()
        .to_string()
}


pub fn fill_hashmap(path: &str) -> Result<HashMap<String, String>,  std::io::Error>{
    let mut files: HashMap<String,String> = HashMap::new();

    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) =>{
                if entry.file_type().is_file() {

                    let file_path_str = entry.path().to_string_lossy().to_string();

                    let file_name_str= strip_directory(&file_path_str);


                    files.insert(file_name_str, file_path_str);
                }
                
            },
            Err(e) => eprintln!("Error: {}", e),
        }
    }

    Ok(files)
} 

pub fn search_hash_map(file_name: String, files_hash_map: &HashMap<String, String>) -> Result<Vec<String>, std::io::Error>{
    let mut res = Vec::new();

    for (key, _) in &*files_hash_map {

        if key.contains(&file_name){
            res.push(key.clone());
        }
    }

    Ok(res)
}