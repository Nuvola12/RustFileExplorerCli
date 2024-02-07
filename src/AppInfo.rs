use std::{collections::HashMap, fs::metadata};

use tui::{
    backend::{Backend, CrosstermBackend}, layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Modifier, Style}, symbols::line::VERTICAL, text::{Span, Spans}, widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    }, Terminal,Frame
};

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
    pub input_mode: InputMode,
    pub message: String,
    pub current_directory: String,
    pub active_menu_item: MenuItem,
    pub directory_list_state: ListState,
    pub search_list_state: ListState,
    pub selected_file: String,
    pub loaded_files: HashMap<String, String>,
    pub input_type: InputType,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Normal,
            message: String::new(),
            current_directory: String::new(),
            active_menu_item: MenuItem::Home,
            directory_list_state: ListState::default(),
            search_list_state: ListState::default(),
            selected_file: String::new(),
            loaded_files: HashMap::new(),
            input_type: InputType::None,
        }
    }
}