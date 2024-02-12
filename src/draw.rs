use tui::{
    backend::{Backend, CrosstermBackend}, layout::{Alignment, Constraint, Direction, Layout, Margin}, style::{Color, Modifier, Style}, symbols::line::VERTICAL, text::{Span, Spans}, widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    }, Terminal,Frame};
use std::{fs::{metadata, File}, path::Path};
use std::fs;

use crate::AppInfo;
use crate::filesystem;

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &mut AppInfo::App){
    let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Length(2),
                        Constraint::Min(2),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                ).split(size);

            //Top Bar
            let top_bar = render_directory_display(&app.current_directory);
            f.render_widget(top_bar, chunks[0]);
            //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            
            //Main Content
            
            match app.active_menu_item {
                AppInfo::MenuItem::Home => {
                    let file_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                        ].as_ref(),
                    ).split(chunks[1]);
                    
                    let (left, right) = render_file_widget(app);
                    
                    f.render_stateful_widget(left, file_chunks[0], &mut app.directory_list_state);
                    f.render_widget(right, file_chunks[1]);
                },
                AppInfo::MenuItem::Text => {
                    let top = render_search_bar(&app);
                    f.render_widget(top, chunks[1]);

                },
                AppInfo::MenuItem::Search => {
                    let file_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(60),
                        Constraint::Percentage(40),
                        ].as_ref(),
                    ).split(chunks[1]);
                    
                    let (left, right) = render_search_results_widget(app);
                    
                    f.render_stateful_widget(left, file_chunks[0], &mut app.directory_list_state);
                    f.render_widget(right, file_chunks[1]);
                },
                AppInfo::MenuItem::MakeFile => {

                }
            }
            //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
            
            //Bottom Bar
            let bottom_bar = render_bottom_bar();
            f.render_widget(bottom_bar, chunks[2])
            //////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
}


pub fn is_directory_empty(path: &String) -> bool {
    if let Ok(entries) = fs::read_dir(path) {
        return entries.count() == 0;
    }
    // If an error occurs while reading the directory, you can handle it accordingly.
    false
}

pub fn path_exists(path: &String) -> bool{
    fs::metadata(path).is_ok()
}


pub fn strip_directory(path: &String) -> String{
    path.split('/').last().unwrap().to_string()
}

pub fn render_directory<'a>(app: &AppInfo::App) -> Result<(List<'a>, String), Box<dyn std::error::Error>> {
    let md = fs::metadata(&app.current_directory)?;

    if is_directory_empty(&app.current_directory) || !md.is_dir() {
        let list = List::new(Vec::new());
        let selected_dir = "";
        return Ok((list, selected_dir.to_string()));
    }

    let pets = Block::default()
        .borders(Borders::RIGHT)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);

    let curr_dir = filesystem::util::get_files_in_directory(&app.current_directory)?;

    let items: Vec<_> = curr_dir
        .iter()
        .map(|file| {
            let tmp = strip_directory(file);
            ListItem::new(Spans::from(vec![Span::styled(
                tmp.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = curr_dir
        .get(
            app.directory_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .map(|pet| pet.clone())
        .unwrap_or_else(|| String::new());

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    Ok((list, selected_pet))
}


pub fn render_search_results<'a>(app: &AppInfo::App) -> Result<(List<'a>, String), Box<dyn std::error::Error>> {
    let pets = Block::default()
        .borders(Borders::RIGHT)
        .style(Style::default().fg(Color::White))
        .border_type(BorderType::Plain);

    let search_term = app.message.clone();
    let hash_search_results = filesystem::util::search_hash_map(search_term, &app.loaded_files).unwrap();

    let items: Vec<_> = hash_search_results
        .iter()
        .map(|file| {
            let tmp = strip_directory(file);
            ListItem::new(Spans::from(vec![Span::styled(
                tmp.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = hash_search_results
        .get(
            app.search_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .map(|pet| pet.clone())
        .unwrap_or_else(|| String::new());

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );
    Ok((list, selected_pet))
}


pub fn render_search_results_widget<'a>(app: &mut AppInfo::App) -> (List<'a>, Paragraph<'a>){

    let (directory_widget, selected_dir) = match render_search_results(app){
        Ok(data) => data,
        Err(e) => panic!("test"),
    };
    
    if path_exists(&selected_dir){
        app.selected_file.clear();
        app.selected_file.push_str(&selected_dir);
    }
    
    let info_bar = render_details(&app.selected_file).unwrap();

    (directory_widget, info_bar)
}


pub fn render_search_bar<'a>( app: &'a AppInfo::App) -> (Paragraph<'a>){
    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            AppInfo::InputMode::Normal => Style::default(),
            AppInfo::InputMode::Typing => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL));

    input
}



pub fn render_file_widget<'a>(app: &mut AppInfo::App) -> (List<'a>, Paragraph<'a>){

    let (directory_widget, selected_dir) = match render_directory(app){
        Ok(data) => data,
        Err(e) => panic!("{:?}", e),
    };
    
    if path_exists(&selected_dir){
        app.selected_file.clear();
        app.selected_file.push_str(&selected_dir);
    }
    


    let info_bar = render_details(&app.selected_file).unwrap();

    (directory_widget, info_bar)
}

pub fn render_directory_display<'a>( directory: &String) -> Paragraph<'a> {
    Paragraph::new(directory.to_string())
    .style(Style::default().fg(Color::LightGreen))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::BOTTOM)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}

pub fn render_bottom_bar<'a>() -> Paragraph<'a> {
    Paragraph::new("N Create    C Copy  X Cut   V Paste   R Rename  D Delete    O Open / Search ")
        .style(Style::default().fg(Color::LightGreen))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .style(Style::default().fg(Color::White))
        )
}


pub fn render_details<'a>(path: &String) -> Result<Paragraph<'a>, Box<dyn std::error::Error>>{
    let md   = metadata(path).unwrap();

    let file_name = filesystem::util::file_name(path).unwrap();
    let file_type = filesystem::util::is_path_file(&md).unwrap();
    let file_size = filesystem::util::get_size_in_mb(&md).unwrap();
    let file_modify_time = filesystem::util::last_modified_time(&md).unwrap();

    let file_name_string = format!("File Name: {}",file_name);
    let file_type_string = format!("File Type: {}",file_type);
    let file_size_string = format!("File Size: {}MB",file_size);
    let file_modify_time_string = format!("Last Time Modified: {}",file_modify_time);

    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(file_name_string)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(file_type_string)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(file_size_string)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(file_modify_time_string)]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
            "Rust-CLI FileExplorer",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default());
        Ok(home)
}
