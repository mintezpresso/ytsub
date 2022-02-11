use crate::app::{App, Selected};
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::Span;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;

pub fn draw<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .direction(Direction::Horizontal)
        .split(f.size());
    draw_channels(f, app, chunks[0]);
    draw_videos(f, app, chunks[1]);
}

fn draw_channels<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let channels = app
        .channels
        .items
        .iter()
        .map(|ch| ch.channel_name.clone())
        .map(Span::raw)
        .map(ListItem::new)
        .collect::<Vec<ListItem>>();
    let channels = List::new(channels)
        .block(
            Block::default().borders(Borders::ALL).title(Span::styled(
                "Channels",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        )
        .highlight_style(match app.selected {
            Selected::Channels => Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
            Selected::Videos => Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        });
    f.render_stateful_widget(channels, area, &mut app.channels.state);
}

fn draw_videos<B: Backend>(f: &mut Frame<B>, app: &mut App, area: Rect) {
    let videos = app
        .videos
        .items
        .iter()
        .map(|video| {
            let title = video.title.clone();
            if video.watched {
                Span::styled(title, Style::default().fg(Color::DarkGray))
            } else {
                Span::raw(title)
            }
        })
        .map(ListItem::new)
        .collect::<Vec<ListItem>>();
    let videos = List::new(videos)
        .block(
            Block::default().borders(Borders::ALL).title(Span::styled(
                if let Some(channel) = app.get_current_channel() {
                    channel.channel_name.clone()
                } else {
                    Default::default()
                },
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
        )
        .highlight_style({
            let mut style = Style::default();
            style = match app.selected {
                Selected::Channels => style.fg(Color::Blue),
                Selected::Videos => style.fg(Color::Magenta),
            };
            if let Some(video) = app.get_current_video() {
                if !video.watched {
                    style = style.add_modifier(Modifier::BOLD)
                }
            }
            style
        });
    f.render_stateful_widget(videos, area, &mut app.videos.state);
}