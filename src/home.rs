use anyhow::{bail, Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use mastodon_async::{page::Page, prelude::Status, Mastodon};
use parking_lot::RwLock;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{List, ListItem, ListState},
    Frame,
};
use std::sync::Arc;
use time::format_description;
use tokio::sync::mpsc::Sender;
use tracing::{debug, error, info, instrument};

use crate::{
    authentication,
    event::{Event, Outcome},
};

pub struct Home {
    _event_sender: Sender<Event>,
    authentication_data: Arc<RwLock<Option<authentication::State>>>,
    title: String,
    timeline_page: Option<Page<Status>>,
    timeline_items: Option<Vec<Status>>,
    status: String,
    list_state: Arc<RwLock<ListState>>,
}

impl Home {
    pub fn new(
        event_sender: Sender<Event>,
        authentication_data: Arc<RwLock<Option<authentication::State>>>,
    ) -> Self {
        Self {
            _event_sender: event_sender,
            authentication_data,
            title: String::new(),
            timeline_page: None,
            timeline_items: None,
            status: String::new(),
            list_state: Arc::new(RwLock::new(ListState::default())),
        }
    }

    #[instrument(name = "home", skip_all)]
    pub async fn start(&mut self) -> Result<()> {
        let auth = Arc::clone(&self.authentication_data);
        let auth = auth.read().clone(); // easy way to avoid holding the lock over the await below
        if let Some(auth) = auth {
            info!(username = ?auth.account.username, "logged in");
            let username = auth.account.username.clone();
            let server = auth.config.data.base.trim_start_matches("https://");
            self.title = format!("{username}@{server}");
            let mastodon = auth.mastodon;
            self.load_home_timeline(mastodon).await?;
        } else {
            self.title = "Not logged in".to_string();
            bail!("not logged in");
        }
        Ok(())
    }

    #[instrument(skip_all, fields())]
    async fn load_home_timeline(&mut self, mastodon: Mastodon) -> Result<()> {
        let page = mastodon
            .get_home_timeline()
            .await
            .context("failed to load timeline")?;
        info!("loaded timeline page");
        self.timeline_items = Some(page.initial_items.clone());
        self.timeline_page = Some(page);
        Ok(())
    }

    #[instrument(name = "home::handle_event", skip_all)]
    pub async fn handle_event(&mut self, event: &Event) -> Outcome {
        match event {
            Event::Crossterm(event) => {
                if let CrosstermEvent::Key(key) = *event {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Char('j')) => {
                            if let Err(err) = self.scroll_down().await {
                                error!("failed to scroll down: {:#}", err);
                            }
                        }
                        (KeyModifiers::NONE, KeyCode::Char('k')) => {
                            if let Err(err) = self.scroll_up().await {
                                error!("failed to scroll up: {:#}", err);
                            }
                        }
                        _ => return Outcome::Ignored,
                    }
                }
                Outcome::Handled
            }
            _ => Outcome::Ignored,
        }
    }

    async fn scroll_down(&mut self) -> Result<()> {
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        let mut index = list_state.selected().map_or(0, |s| s + 1);
        if let Some(items) = &self.timeline_items {
            if index >= items.len() {
                info!("loading next page");
                if let Some(page) = self.timeline_page.as_mut() {
                    self.timeline_items = page.next_page().await?;
                    index = 0;
                }
            }
        }
        list_state.select(Some(index));
        self.update_status(index);
        Ok(())
    }

    async fn scroll_up(&mut self) -> Result<()> {
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        let mut index = list_state.selected().unwrap_or(0);
        if let Some(_) = &self.timeline_items {
            if index == 0 {
                info!("loading previous page");
                if let Some(page) = self.timeline_page.as_mut() {
                    if let Some(prev_items) = page.prev_page().await? {
                        index = prev_items.len().saturating_sub(1);
                        self.timeline_items = Some(prev_items);
                    } else {
                        // tried to go back but there was no previous page
                        debug!("no previous page");
                    }
                }
            } else {
                index -= 1;
            }
        }
        list_state.select(Some(index));
        self.update_status(index);
        Ok(())
    }

    fn update_status(&mut self, selected: usize) {
        if let Some(items) = &self.timeline_items {
            if let Some(status) = items.get(selected) {
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

    #[instrument(name = "home::draw", skip_all)]
    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let mut items = vec![];
        if let Some(timeline_items) = &self.timeline_items {
            // debugging for width and selected item
            // items.push(ListItem::new(
            //     "12345678901234567890123456789012345678901234567890123456789012345678901234567890",
            // ));
            // items.push(ListItem::new(format!("{}", self.selected)));
            for status in timeline_items {
                items.push(ListItem::new(format_status(status, area.width)));
            }
        } else {
            items.push(ListItem::new("Loading timeline..."));
        }
        // this looks great on a dark theme, but not so much on a light one
        let style = Style::default().bg(Color::Rgb(16, 32, 64));
        let list = List::new(items).highlight_style(style);
        // let mut state = ListState::default();
        // state.select(Some(self.selected));
        let list_state = Arc::clone(&self.list_state);
        let mut state = list_state.write();
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
