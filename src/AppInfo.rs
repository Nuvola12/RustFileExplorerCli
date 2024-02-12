use std::collections::HashMap;

use tui::widgets::ListState;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Typing,
}


pub enum InputType{
    None,
    Searching,
    MakeFile,
    RenameFile,
}
/// App holds the state of the application

pub enum MenuItem{
    Home,
    Text,
    Search,
    MakeFile,
} 

impl From<MenuItem> for usize{
    fn from(input: MenuItem) -> usize{
        match input {
            MenuItem::Home => 0,
            MenuItem::Text => 1,
            MenuItem::Search => 2,
            MenuItem::MakeFile => 3,
        }
    }
}

pub struct App {
    pub input: String,
    pub message: String,
    pub selected_file: String,
    pub current_directory: String,
    pub input_type: InputType,
    pub input_mode: InputMode,
    pub active_menu_item: MenuItem,
    pub directory_list_state: ListState,
    pub search_list_state: ListState,
    pub loaded_files: HashMap<String, String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            message: String::new(),
            selected_file: String::new(),
            current_directory: String::new(),
            input_type: InputType::None,
            input_mode: InputMode::Normal,
            active_menu_item: MenuItem::Home,
            directory_list_state: ListState::default(),
            search_list_state: ListState::default(),
            loaded_files: HashMap::new(),
        }
    }
}