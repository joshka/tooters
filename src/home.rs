use anyhow::{bail, Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyModifiers};
use mastodon_async::{
    page::Page,
    prelude::{Card, Status},
    Mastodon,
};
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
use textwrap::Options;
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
        info!("loading home timeline");
        let page = mastodon
            .get_home_timeline()
            .await
            .context("failed to load timeline")?;
        info!("loaded home timeline");
        self.timeline_items = Some(page.initial_items.clone());
        self.timeline_page = Some(page);
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        list_state.select(Some(0));
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
                let url = status
                    .reblog
                    .as_ref()
                    .map_or(status.url.clone(), |reblog| reblog.url.clone())
                    .unwrap_or_default();
                self.status = format!("{url}");
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
            for status in timeline_items {
                items.push(ListItem::new(format_status(status, area.width)));
            }
        } else {
            items.push(ListItem::new("Loading timeline..."));
        }
        // this looks great on a dark theme, but not so much on a light one
        let style = Style::default().bg(Color::Rgb(48, 64, 96));
        let list = List::new(items).highlight_style(style);
        let list_state = Arc::clone(&self.list_state);
        let mut state = list_state.write();
        frame.render_stateful_widget(list, area, &mut state);
    }
}

fn format_status(status: &Status, width: u16) -> Text {
    // eventually this should be themed instead of hardcoded
    let header_bg = Color::Rgb(80, 96, 128);
    let subtle_fg = Color::Rgb(160, 176, 208);
    let acct_fg = Color::LightYellow;
    let display_fg = Color::LightGreen;

    let date_format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]")
        .unwrap_or_default();
    let date = status.created_at.format(&date_format).unwrap_or_default();
    let date_span = Span::styled(
        format!("{date} "),
        Style::default().bg(header_bg).fg(subtle_fg),
    );

    let account = &status.account;
    let reblog = status.reblog.as_ref();

    let acct_span = Span::styled(
        format!("{} ", account.acct),
        Style::default().bg(header_bg).fg(acct_fg),
    );
    let display_span = Span::styled(
        format!("({}) ", account.display_name),
        Style::default()
            .bg(header_bg)
            .fg(display_fg)
            .add_modifier(Modifier::ITALIC),
    );
    let reblog_span = reblog.map_or(Span::from(""), |_| {
        Span::styled("reblogging ", Style::default().bg(header_bg).fg(subtle_fg))
    });
    let reblog_acct_span = reblog.map_or(Span::from(""), |status| {
        Span::styled(
            format!("{} ", status.account.acct),
            Style::default().bg(header_bg).fg(acct_fg),
        )
    });
    let reblog_display_span = reblog.map_or(Span::from(""), |status| {
        Span::styled(
            format!("({}) ", status.account.display_name),
            Style::default()
                .bg(header_bg)
                .fg(display_fg)
                .add_modifier(Modifier::ITALIC),
        )
    });

    let mut text = Text::from(Spans::from(vec![
        date_span,
        acct_span,
        display_span,
        reblog_span,
        reblog_acct_span,
        reblog_display_span,
    ]));

    // Main content
    let html = status.content.trim().clone();
    if !html.is_empty() {
        let content = html2text::from_read(html.as_bytes(), width as usize);
        text.extend(Text::from(content));
    }

    // reblogged content
    if let Some(status) = reblog {
        let html = status.content.clone();
        let content = html2text::from_read(html.as_bytes(), (width - 2) as usize);
        // let content = textwrap::indent(&content, "▌ ");
        text.extend(Text::from(content));
    }

    // card
    if let Some(card) = &status.card {
        let content = format_card(card, width);
        text.extend(Text::styled(content, Style::default().fg(Color::DarkGray)));
    }
    text.extend(Text::raw(""));
    text
}

fn format_card(card: &Card, width: u16) -> String {
    let mut content = card.url.clone();
    if card.title.trim() != "" {
        content.push_str(format!("\ntitle: {}", card.title).as_str());
    }
    if card.description.trim() != "" {
        content.push_str(format!("\ndescription: {}", card.description).as_str());
    }
    let content = textwrap::fill(
        &content,
        Options::new((width - 2) as usize)
            .initial_indent("▌ ")
            .subsequent_indent("▌ "),
    );
    content
}
