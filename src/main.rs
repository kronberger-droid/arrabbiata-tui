mod api;
mod app;
mod ui;

use std::io::{self, stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use tokio::sync::mpsc;

use api::{ApiRequest, ApiResult, FALLBACK_USER_ID, USER_ID};

use app::{send_notification, App, Phase};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Restore terminal on panic
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(info);
    }));

    let (tx, mut rx) = mpsc::unbounded_channel::<ApiResult>();
    let client = reqwest::Client::new();
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        // Process API responses
        while let Ok(result) = rx.try_recv() {
            match result {
                ApiResult::Success(resp) => app.handle_response(resp),
                ApiResult::Error(err) => {
                    app.loading = false;
                    app.error = Some(err);
                }
            }
        }

        // Send notification when timer expires
        if app.check_notify() {
            let msg = match app.current_type {
                Some(0) => "Work phase complete!",
                Some(1) => "Break is over!",
                _ => "Timer complete!",
            };
            send_notification(msg);
        }

        // Handle input
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                if key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('s') if app.phase == Phase::Initial && !app.loading => {
                        app.loading = true;
                        app.error = None;
                        api::spawn_request(
                            &client,
                            &tx,
                            ApiRequest {
                                user_id: USER_ID.clone(),
                                workout_type: None,
                                planned_time: None,
                                actual_time: None,
                                workout_date: None,
                            },
                        );
                    }
                    KeyCode::Char('c') if app.phase == Phase::Initial && !app.loading => {
                        app.loading = true;
                        app.error = None;
                        api::spawn_request(
                            &client,
                            &tx,
                            ApiRequest {
                                user_id: USER_ID.clone(),
                                workout_type: Some(2),
                                planned_time: None,
                                actual_time: None,
                                workout_date: None,
                            },
                        );
                    }
                    KeyCode::Char('f') if app.phase == Phase::Running => {
                        app.stop_timer();
                    }
                    KeyCode::Char('n') if app.phase == Phase::Stopped && !app.loading => {
                        app.loading = true;
                        app.error = None;
                        let user_id = app
                            .last_user_id
                            .clone()
                            .unwrap_or_else(|| FALLBACK_USER_ID.clone());
                        api::spawn_request(
                            &client,
                            &tx,
                            ApiRequest {
                                user_id,
                                workout_type: app.current_type,
                                planned_time: Some(app.planned_time_sec as i64),
                                actual_time: Some(app.elapsed_at_stop_sec as i64),
                                workout_date: None,
                            },
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
