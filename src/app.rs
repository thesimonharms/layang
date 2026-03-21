use crossterm::event::{Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::TableState;

use crate::types::Post;

pub struct TextInput {
    chars: Vec<char>,
    pub cursor: usize,
}

impl TextInput {
    pub fn new(initial: &str) -> Self {
        let chars: Vec<char> = initial.chars().collect();
        let cursor = chars.len();
        Self { chars, cursor }
    }

    pub fn value(&self) -> String {
        self.chars.iter().collect()
    }

    pub fn insert_char(&mut self, c: char) {
        self.chars.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.chars.remove(self.cursor);
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.chars.len() {
            self.chars.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.chars.len() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.chars.len();
    }

    pub fn set(&mut self, s: &str) {
        self.chars = s.chars().collect();
        self.cursor = self.chars.len();
    }

    pub fn chars(&self) -> &[char] {
        &self.chars
    }
}

pub struct FormState {
    pub title: TextInput,
    pub excerpt: TextInput,
    pub active_field: usize,
    pub body: String,
    pub is_edit: bool,
    pub original_slug: String,
}

impl FormState {
    pub fn new() -> Self {
        Self {
            title: TextInput::new(""),
            excerpt: TextInput::new(""),
            active_field: 0,
            body: String::new(),
            is_edit: false,
            original_slug: String::new(),
        }
    }

    pub fn reset(&mut self) {
        self.title.set("");
        self.excerpt.set("");
        self.active_field = 0;
        self.body = String::new();
        self.is_edit = false;
        self.original_slug = String::new();
    }
}

pub enum StatusMessage {
    Error(String),
    Success(String),
}

pub enum AppMode {
    Loading(String),
    PostList,
    Form,
    ConfirmDelete,
}

pub struct App {
    pub posts: Vec<Post>,
    pub list_state: TableState,
    pub mode: AppMode,
    pub form: FormState,
    pub status: Option<StatusMessage>,
    pub confirm_slug: String,
}

pub enum AppAction {
    None,
    Quit,
    Refresh,
    FetchAndEdit { slug: String },
    LaunchEditorThenSubmit,
    Publish { slug: String },
    Unpublish { slug: String },
    ConfirmDeletePost { slug: String },
    DeletePost { slug: String },
    Cancel,
}

impl App {
    pub fn new() -> Self {
        let mut list_state = TableState::default();
        list_state.select(None);
        Self {
            posts: Vec::new(),
            list_state,
            mode: AppMode::Loading("Loading...".into()),
            form: FormState::new(),
            status: None,
            confirm_slug: String::new(),
        }
    }

    pub fn set_posts(&mut self, posts: Vec<Post>) {
        self.posts = posts;
        if !self.posts.is_empty() {
            // Keep selection in bounds, or reset to 0
            let current = self.list_state.selected().unwrap_or(0);
            let new_sel = current.min(self.posts.len() - 1);
            self.list_state.select(Some(new_sel));
        } else {
            self.list_state.select(None);
        }
    }

    pub fn set_status(&mut self, msg: StatusMessage) {
        self.status = Some(msg);
    }

    pub fn selected_post(&self) -> Option<&Post> {
        self.list_state.selected().and_then(|i| self.posts.get(i))
    }

    fn active_input(&mut self) -> &mut TextInput {
        if self.form.active_field == 1 {
            &mut self.form.excerpt
        } else {
            &mut self.form.title
        }
    }

    pub fn handle_event(&mut self, event: Event) -> AppAction {
        match &self.mode {
            AppMode::Loading(_) => AppAction::None,
            AppMode::PostList => self.handle_post_list_event(event),
            AppMode::Form => self.handle_form_event(event),
            AppMode::ConfirmDelete => self.handle_confirm_delete_event(event),
        }
    }

    fn handle_post_list_event(&mut self, event: Event) -> AppAction {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return AppAction::None;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.status = None;
                    if !self.posts.is_empty() {
                        let current = self.list_state.selected().unwrap_or(0);
                        let next = if current == 0 {
                            self.posts.len() - 1
                        } else {
                            current - 1
                        };
                        self.list_state.select(Some(next));
                    }
                    AppAction::None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.status = None;
                    if !self.posts.is_empty() {
                        let current = self.list_state.selected().unwrap_or(0);
                        let next = if current >= self.posts.len() - 1 {
                            0
                        } else {
                            current + 1
                        };
                        self.list_state.select(Some(next));
                    }
                    AppAction::None
                }
                KeyCode::Char('c') | KeyCode::Char('C')
                    if !key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    self.form.reset();
                    self.mode = AppMode::Form;
                    AppAction::None
                }
                KeyCode::Char('e') | KeyCode::Char('E') => {
                    if let Some(post) = self.selected_post() {
                        let slug = post.slug.clone();
                        AppAction::FetchAndEdit { slug }
                    } else {
                        AppAction::None
                    }
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    if let Some(post) = self.selected_post() {
                        let slug = post.slug.clone();
                        AppAction::Publish { slug }
                    } else {
                        AppAction::None
                    }
                }
                KeyCode::Char('u') | KeyCode::Char('U') => {
                    if let Some(post) = self.selected_post() {
                        let slug = post.slug.clone();
                        AppAction::Unpublish { slug }
                    } else {
                        AppAction::None
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if let Some(post) = self.selected_post() {
                        let slug = post.slug.clone();
                        AppAction::ConfirmDeletePost { slug }
                    } else {
                        AppAction::None
                    }
                }
                KeyCode::Char('r') | KeyCode::Char('R') => AppAction::Refresh,
                KeyCode::Char('q') | KeyCode::Char('Q') => AppAction::Quit,
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    AppAction::Quit
                }
                _ => AppAction::None,
            }
        } else {
            AppAction::None
        }
    }

    fn handle_form_event(&mut self, event: Event) -> AppAction {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return AppAction::None;
            }
            // Check for Ctrl+C first
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(KeyModifiers::CONTROL)
            {
                return AppAction::Quit;
            }

            match key.code {
                KeyCode::Tab | KeyCode::Down => {
                    self.form.active_field = (self.form.active_field + 1) % 2;
                    AppAction::None
                }
                KeyCode::BackTab | KeyCode::Up => {
                    self.form.active_field = if self.form.active_field == 0 { 1 } else { 0 };
                    AppAction::None
                }
                KeyCode::Enter => AppAction::LaunchEditorThenSubmit,
                KeyCode::Esc => {
                    self.mode = AppMode::PostList;
                    AppAction::Cancel
                }
                KeyCode::Backspace => {
                    self.active_input().backspace();
                    AppAction::None
                }
                KeyCode::Delete => {
                    self.active_input().delete_forward();
                    AppAction::None
                }
                KeyCode::Left => {
                    self.active_input().move_left();
                    AppAction::None
                }
                KeyCode::Right => {
                    self.active_input().move_right();
                    AppAction::None
                }
                KeyCode::Home => {
                    self.active_input().move_home();
                    AppAction::None
                }
                KeyCode::End => {
                    self.active_input().move_end();
                    AppAction::None
                }
                KeyCode::Char(c) => {
                    self.active_input().insert_char(c);
                    AppAction::None
                }
                _ => AppAction::None,
            }
        } else {
            AppAction::None
        }
    }

    fn handle_confirm_delete_event(&mut self, event: Event) -> AppAction {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return AppAction::None;
            }
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => {
                    let slug = self.confirm_slug.clone();
                    self.mode = AppMode::PostList;
                    AppAction::DeletePost { slug }
                }
                KeyCode::Char('n') | KeyCode::Esc => {
                    self.mode = AppMode::PostList;
                    AppAction::Cancel
                }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    AppAction::Quit
                }
                _ => AppAction::None,
            }
        } else {
            AppAction::None
        }
    }
}
