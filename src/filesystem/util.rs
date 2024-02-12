use std::path::Path;
use std::fs::{self, metadata, Metadata};

use walkdir::WalkDir;
use std::collections::HashMap;

use chrono::offset::Utc;
use chrono::DateTime;
use std::time::SystemTime;

use std::env;



pub fn update_current_directory(selected_directory: &String, current_directory: &mut String) {
    current_directory.clear();
    current_directory.push_str(&selected_directory);
}

pub fn move_up_in_path(path_str: &String) -> Result<Option<String>, std::io::Error>{
    let path =  Path::new(path_str);

    Ok(path.parent()
        .map(|parent|
             parent.to_string_lossy()
             .into_owned()))
}

pub fn is_path_file(metadata: &fs::Metadata) -> Result<String, std::io::Error> {
    let is_path_file = metadata.is_file();

    if is_path_file{
        return Ok("File".to_string());
    }
    return Ok("Folder".to_string());
}

pub fn get_size_in_mb(metadata: &fs::Metadata) -> Result<f64, std::io::Error> {
    let size_in_bytes = metadata.len();
    let size_in_mb = size_in_bytes as f64 / (1024.0 * 1024.0);
    Ok(size_in_mb)
}

pub fn last_modified_time(metadata: &fs::Metadata) -> Result<String, std::io::Error> {

    let modified_time = metadata.modified()?;

    let datetime: DateTime<Utc> = modified_time.into();
    let res = format!("{}", datetime.format("%d/%m/%Y %T"));

    Ok(res)
}

pub fn file_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
}


pub fn get_files_in_directory(path: &String) -> Result<Vec<String>, std::io::Error> {
    let paths: fs::ReadDir = fs::read_dir(path)?;

    let mut curr_dir = Vec::new();

    for path in paths {
        let curr = path.unwrap().path();
        let mut my_str = curr
            .into_os_string()
            .into_string()
            .unwrap()
            .replace("\\", "/");


        curr_dir.push(my_str);
        
    }

    Ok(curr_dir)
}


pub fn strip_directory(path: &str) -> String{
    path
        .replace("\\", "/")
        .split('/')
        .last()
        .unwrap()
        .to_string()
}

pub fn get_cahce_fodler() -> Result<String, std::io::Error>{
    let exe_path = env::current_exe()?;
    let mut exe_dir = exe_path.to_str().unwrap().to_string();

    exe_dir = move_up_in_path(&exe_dir).unwrap().unwrap();
    let cache_dir = format!("{}\\cache", exe_dir);

    Ok(cache_dir)
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