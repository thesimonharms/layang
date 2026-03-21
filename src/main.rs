mod api;
mod app;
mod config;
mod types;
mod ui;

use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;

use api::ApiClient;
use app::{App, AppAction, AppMode, StatusMessage};
use types::Post;

async fn fetch_all_posts(api: &ApiClient) -> Result<Vec<Post>> {
    let (drafts, published) = tokio::join!(api.list_drafts(), api.list_posts());
    let mut posts = drafts.unwrap_or_default();
    posts.extend(published.unwrap_or_default());
    Ok(posts)
}

fn generate_excerpt(body: &str) -> String {
    // Find first non-empty paragraph that isn't a heading or code fence
    let text = body
        .lines()
        .find(|line| {
            let t = line.trim();
            !t.is_empty() && !t.starts_with('#') && !t.starts_with("```")
        })
        .unwrap_or("")
        .trim();

    // Strip inline markdown: **bold**, *italic*, `code`, [text](url)
    let mut s = text.to_string();
    // [text](url) -> text
    while let (Some(ob), Some(cb)) = (s.find('['), s.find("](")) {
        if cb > ob {
            if let Some(end) = s[cb..].find(')') {
                let link_text = s[ob + 1..cb].to_string();
                s = format!("{}{}{}", &s[..ob], link_text, &s[cb + end + 1..]);
                continue;
            }
        }
        break;
    }
    // Remove ** and * and `
    s = s.replace("**", "").replace('`', "").replace('*', "");

    // Truncate to 160 chars on a word boundary
    if s.len() <= 160 {
        s
    } else {
        let truncated = &s[..160];
        match truncated.rfind(' ') {
            Some(pos) => format!("{}...", &truncated[..pos]),
            None => format!("{}...", truncated),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Load config (may exit early on first run)
    let config = config::load_or_create_config()?;

    // 2. Prompt for token with rpassword
    let token = rpassword::prompt_password("Token: ")?;

    // 3. Create ApiClient
    let api = ApiClient::new(config.api_url.clone(), token);

    // 4. Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 5. Create App, set initial mode to Loading
    let mut app = App::new();
    app.mode = AppMode::Loading("Loading posts...".into());

    // 6. Fetch initial post list
    terminal.draw(|f| ui::render(f, &app))?;
    match fetch_all_posts(&api).await {
        Ok(posts) => {
            app.set_posts(posts);
            app.mode = AppMode::PostList;
        }
        Err(e) => {
            app.mode = AppMode::PostList;
            app.set_status(StatusMessage::Error(e.to_string()));
        }
    }

    // 7. Main event loop
    let result = run_event_loop(&mut terminal, &mut app, &api, &config).await;

    // 8. On exit: restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    api: &ApiClient,
    config: &config::Config,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render(f, app))?;

        let has_event = tokio::task::block_in_place(|| {
            crossterm::event::poll(std::time::Duration::from_millis(100))
        })?;

        if !has_event {
            continue;
        }

        let event = tokio::task::block_in_place(|| crossterm::event::read())?;

        match app.handle_event(event) {
            AppAction::None => {}
            AppAction::Quit => break,
            AppAction::Refresh | AppAction::Cancel => {
                app.mode = AppMode::Loading("Refreshing...".into());
                terminal.draw(|f| ui::render(f, app))?;
                match fetch_all_posts(&api).await {
                    Ok(posts) => {
                        app.set_posts(posts);
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Success("Refreshed.".into()));
                    }
                    Err(e) => {
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
            }
            AppAction::FetchAndEdit { slug } => {
                app.mode = AppMode::Loading("Fetching post...".into());
                terminal.draw(|f| ui::render(f, app))?;
                match api.get_post(&slug).await {
                    Ok(post) => {
                        app.form.title.set(&post.title);
                        app.form.excerpt.set(post.excerpt.as_deref().unwrap_or(""));
                        app.form.active_field = 0;
                        app.form.body = post.body.unwrap_or_default();
                        app.form.is_edit = true;
                        app.form.original_slug = slug;
                        app.mode = AppMode::Form;
                    }
                    Err(e) => {
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
            }
            AppAction::LaunchEditorThenSubmit => {
                let prefill = app.form.body.clone();

                // Suspend TUI
                disable_raw_mode()?;
                execute!(io::stdout(), LeaveAlternateScreen)?;

                // Write temp file
                let tmp = {
                    use std::io::Write;
                    let mut f = tempfile::Builder::new().suffix(".md").tempfile()?;
                    f.write_all(prefill.as_bytes())?;
                    f.into_temp_path()
                };

                std::process::Command::new(&config.editor)
                    .arg(tmp.as_os_str())
                    .status()?;

                let body = std::fs::read_to_string(&tmp)?.replace("\r\n", "\n");
                drop(tmp);

                // Resume TUI
                enable_raw_mode()?;
                execute!(io::stdout(), EnterAlternateScreen)?;
                terminal.clear()?;

                // Submit to API
                let title = app.form.title.value();
                let form_excerpt = app.form.excerpt.value();
                let excerpt = if form_excerpt.is_empty() {
                    generate_excerpt(&body)
                } else {
                    form_excerpt
                };

                app.mode = AppMode::Loading("Saving...".into());
                terminal.draw(|f| ui::render(f, app))?;

                if app.form.is_edit {
                    let orig = app.form.original_slug.clone();
                    match api.update_post(&orig, &title, &body, &excerpt).await {
                        Ok(_) => {
                            app.set_status(StatusMessage::Success("Saved.".into()));
                        }
                        Err(e) => {
                            app.set_status(StatusMessage::Error(e.to_string()));
                        }
                    }
                } else {
                    match api.create_post(&title, &body, &excerpt).await {
                        Ok(post) => {
                            app.mode = AppMode::Loading("Publishing...".into());
                            terminal.draw(|f| ui::render(f, app))?;
                            match api.publish_post(&post.slug).await {
                                Ok(_) => {
                                    app.set_status(StatusMessage::Success("Published.".into()));
                                }
                                Err(e) => {
                                    app.set_status(StatusMessage::Error(format!("Saved but publish failed: {}", e)));
                                }
                            }
                        }
                        Err(e) => {
                            app.set_status(StatusMessage::Error(e.to_string()));
                        }
                    }
                }

                // Refresh list
                match fetch_all_posts(&api).await {
                    Ok(posts) => {
                        app.set_posts(posts);
                    }
                    Err(e) => {
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
                app.mode = AppMode::PostList;
            }
            AppAction::Publish { slug } => {
                app.mode = AppMode::Loading("Publishing...".into());
                terminal.draw(|f| ui::render(f, app))?;
                match api.publish_post(&slug).await {
                    Ok(_) => {
                        app.set_status(StatusMessage::Success("Published.".into()));
                    }
                    Err(e) => {
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
                match fetch_all_posts(&api).await {
                    Ok(posts) => {
                        app.set_posts(posts);
                        app.mode = AppMode::PostList;
                    }
                    Err(e) => {
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
            }
            AppAction::Unpublish { slug } => {
                app.mode = AppMode::Loading("Unpublishing...".into());
                terminal.draw(|f| ui::render(f, app))?;
                match api.unpublish_post(&slug).await {
                    Ok(_) => {
                        app.set_status(StatusMessage::Success("Unpublished.".into()));
                    }
                    Err(e) => {
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
                match fetch_all_posts(&api).await {
                    Ok(posts) => {
                        app.set_posts(posts);
                        app.mode = AppMode::PostList;
                    }
                    Err(e) => {
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
            }
            AppAction::ConfirmDeletePost { slug } => {
                app.confirm_slug = slug;
                app.mode = AppMode::ConfirmDelete;
            }
            AppAction::DeletePost { slug } => {
                app.mode = AppMode::Loading("Deleting...".into());
                terminal.draw(|f| ui::render(f, app))?;
                match api.delete_post(&slug).await {
                    Ok(_) => {
                        app.set_status(StatusMessage::Success("Deleted.".into()));
                    }
                    Err(e) => {
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
                match fetch_all_posts(&api).await {
                    Ok(posts) => {
                        app.set_posts(posts);
                        app.mode = AppMode::PostList;
                    }
                    Err(e) => {
                        app.mode = AppMode::PostList;
                        app.set_status(StatusMessage::Error(e.to_string()));
                    }
                }
            }
        }
    }

    Ok(())
}
