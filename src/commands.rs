use crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Quit,
    Language,
    Up,
    Down,
    Top,
    Bottom,
    PageUp,
    PageDown,
    NewTask,
    EditTask,
    ToggleDone,
    TogglePin,
    SetTag,
    DeleteTask,
    DeleteAll,
    Search,
    ClearFilters,
    FilterTag(usize),
    SetDueDate,
    Help,
    None,
}

pub fn key_to_command(key: KeyCode) -> Command {
    match key {
        KeyCode::Char('q') | KeyCode::Char('й') => Command::Quit,
        KeyCode::Char('l') | KeyCode::Char('L') | KeyCode::Char('д') | KeyCode::Char('Д') => Command::Language,
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('л') => Command::Up,
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('о') => Command::Down,
        KeyCode::Home | KeyCode::Char('g') | KeyCode::Char('п') => Command::Top,
        KeyCode::End | KeyCode::Char('G') | KeyCode::Char('П') => Command::Bottom,
        KeyCode::PageUp => Command::PageUp,
        KeyCode::PageDown => Command::PageDown,
        KeyCode::Char('n') | KeyCode::Char('т') => Command::NewTask,
        KeyCode::Char('e') | KeyCode::Char('у') => Command::EditTask,
        KeyCode::Char(' ') => Command::ToggleDone,
        KeyCode::Char('p') | KeyCode::Char('з') => Command::TogglePin,
        KeyCode::Char('t') | KeyCode::Char('е') => Command::SetTag,
        KeyCode::Char('d') | KeyCode::Char('в') => Command::DeleteTask,
        KeyCode::Char('D') | KeyCode::Char('В') => Command::DeleteAll,
        KeyCode::Char('/') | KeyCode::Char('.') => Command::Search,
        KeyCode::Esc => Command::ClearFilters,
        KeyCode::Char(c) if ('1'..='9').contains(&c) => {
            let idx = (c as usize) - ('1' as usize);
            Command::FilterTag(idx)
        }
        KeyCode::Char('M') | KeyCode::Char('m') | KeyCode::Char('ь') | KeyCode::Char('Ь') => Command::SetDueDate,
        KeyCode::Char('?') => Command::Help,
        KeyCode::Char('h') | KeyCode::Char('р') => Command::Help,
        _ => Command::None,
    }
}