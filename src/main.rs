mod app;
mod channel;
mod database;
mod input;
mod ui;

use app::App;
use crossterm::event;
use crossterm::event::{Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use futures_util::StreamExt;
use std::io::stdout;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use tokio::runtime::Runtime;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use ui::draw;

fn main() {
    enable_raw_mode().unwrap();
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(stdout);
    let app = Arc::new(Mutex::new(App::new()));
    let mut terminal = Terminal::new(backend).unwrap();
    database::initialize_db(&app.lock().unwrap().conn);
    app.lock().unwrap().add_new_channels();
    app.lock().unwrap().set_mode_subs();
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    loop {
        terminal
            .draw(|f| draw(f, &mut app.lock().unwrap()))
            .unwrap();
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                if let KeyCode::Char('q') = key.code {
                    break;
                } else if let KeyCode::Char('u') = key.code {
                    let current_channel_id = app
                        .lock()
                        .unwrap()
                        .get_current_channel()
                        .unwrap()
                        .channel_id
                        .clone();
                    let cloned_app = app.clone();
                    std::thread::spawn(move || {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(refresh_channel(&cloned_app, current_channel_id));
                        cloned_app.lock().unwrap().load_videos();
                    });
                } else if let KeyCode::Char('r') = key.code {
                    let cloned_app = app.clone();
                    std::thread::spawn(move || {
                        let rt = Runtime::new().unwrap();
                        rt.block_on(refresh_channels(&cloned_app));
                        cloned_app.lock().unwrap().load_videos();
                    });
                } else {
                    input::handle_key(key, &mut app.lock().unwrap());
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}

async fn refresh_channel(app: &Arc<Mutex<App>>, channel_id: String) {
    let instance = app.lock().unwrap().instance();
    app.lock().unwrap().start_refreshing_channel(&channel_id);
    let app = app.clone();
    tokio::task::spawn_blocking(move || {
        let videos_json = instance.get_videos_of_channel(&channel_id);
        app.lock().unwrap().add_videos(videos_json, &channel_id);
        app.lock().unwrap().complete_refreshing_channel(&channel_id);
    });
}

async fn refresh_channels(app: &Arc<Mutex<App>>) {
    app.lock()
        .unwrap()
        .channels
        .items
        .iter_mut()
        .for_each(|channel| channel.set_to_be_refreshed());
    let channel_ids = app.lock().unwrap().channel_ids.clone();
    let instance = app.lock().unwrap().instance();
    let streams = futures_util::stream::iter(channel_ids).map(|channel_id| {
        let instance = instance.clone();
        app.lock().unwrap().start_refreshing_channel(&channel_id);
        let app = app.clone();

        tokio::task::spawn_blocking(move || {
            let videos_json = instance.get_videos_of_channel(&channel_id);
            app.lock().unwrap().add_videos(videos_json, &channel_id);
            app.lock().unwrap().complete_refreshing_channel(&channel_id);
        })
    });
    let mut buffered = streams.buffer_unordered(num_cpus::get());
    while buffered.next().await.is_some() {}
}
