use std::sync::{Arc, RwLock};

use anyhow::{anyhow, bail, Context};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use mastodon_async::prelude::Status;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{List, ListItem, ListState},
    Frame,
};
use time::format_description;
use tokio::sync::mpsc::Sender;
use tracing::info;

use crate::{
    authentication,
    event::{Event, Outcome},
};

pub struct Home {
    _event_sender: Sender<Event>,
    authentication_data: Arc<RwLock<Option<authentication::State>>>,
    title: String,
    timeline: Option<Vec<Status>>,
    selected: usize,
    status: String,
}

impl Home {
    pub const fn new(
        event_sender: Sender<Event>,
        authentication_data: Arc<RwLock<Option<authentication::State>>>,
    ) -> Self {
        Self {
            _event_sender: event_sender,
            authentication_data,
            title: String::new(),
            timeline: None,
            selected: 0,
            status: String::new(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        info!("Starting home component");
        let auth = self
            .authentication_data
            .read()
            .map_err(|e| anyhow!("failed to read authentication data. {e}"))?;
        if let Some(auth) = auth.as_ref() {
            let username = auth.account.username.clone();
            let server = auth.config.data.base.trim_start_matches("https://");
            self.title = format!("{username}@{server}");
            let page = auth
                .mastodon
                .get_home_timeline()
                .await
                .context("failed to load timeline")?;
            self.timeline = Some(page.initial_items);
        } else {
            self.title = "Not logged in".to_string();
            bail!("not logged in");
        }
        Ok(())
    }

    pub fn handle_event(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Crossterm(event) => {
                if let CrosstermEvent::Key(key) = *event {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            self.scroll_down();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            self.scroll_up();
                        }
                        _ => return Outcome::Ignored,
                    }
                }
                Outcome::Handled
            }
            _ => Outcome::Ignored,
        }
    }

    fn scroll_down(&mut self) {
        self.selected += 1;
        self.update_status();
    }

    fn scroll_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.update_status();
    }

    fn update_status(&mut self) {
        if let Some(timeline) = &self.timeline {
            if let Some(status) = timeline.get(self.selected) {
                let date_format =
                    format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
                        .unwrap_or_default();
                let date = status.created_at.format(&date_format).unwrap_or_default();
                let url = status
                    .reblog
                    .as_ref()
                    .map_or(status.url.clone(), |reblog| reblog.url.clone())
                    .unwrap_or_default();
                self.status = format!("({date}) {url}");
            }
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let mut items = vec![];
        if let Some(timeline) = &self.timeline {
            // debugging for width and selected item
            // items.push(ListItem::new(
            //     "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
            // ));
            // items.push(ListItem::new(format!("{}", self.selected)));
            for status in timeline {
                items.push(ListItem::new(format_status(status, area.width)));
            }
        } else {
            items.push(ListItem::new("Loading timeline..."));
        }
        // this looks great on a dark theme, but not so much on a light one
        let style = Style::default().bg(Color::Rgb(16, 32, 64));
        let list = List::new(items).highlight_style(style);
        let mut state = ListState::default();
        state.select(Some(self.selected));
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn format_status(status: &Status, width: u16) -> Text {
    let account = &status.account;
    let reblog = status.reblog.as_ref();
    let acct = reblog.map_or(account.acct.clone(), |reblog| reblog.account.acct.clone());
    let display_name = reblog.map_or(account.display_name.clone(), |reblog| {
        reblog.account.display_name.clone()
    });
    let mut text = Text::from(Spans::from(vec![
        Span::styled(format!("{acct} "), Style::default().fg(Color::Yellow)),
        Span::styled(
            format!("({display_name})"),
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        ),
    ]));
    let html = reblog.map_or(status.content.clone(), |reblog| reblog.content.clone());
    let content = html2text::from_read(html.as_bytes(), width as usize);
    text.extend(Text::from(content));
    text.extend(Text::raw(""));
    text
}
