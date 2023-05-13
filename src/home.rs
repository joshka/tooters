use anyhow::{Context, Result};
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use html2text::render::text_renderer::{TaggedLine, TextDecorator};
use mastodon_async::{
    page::Page,
    prelude::{Card, Status},
};
use parking_lot::RwLock;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{List, ListItem, ListState},
    Frame,
};
use std::{collections::VecDeque, sync::Arc, vec};
use textwrap::Options;
use time::format_description;
use tokio::sync::mpsc::Sender;
use tracing::{debug, info, instrument};

use crate::{
    authentication,
    event::{Event, Outcome},
};

pub struct Home {
    _event_sender: Sender<Event>,
    authentication_data: Arc<RwLock<Option<authentication::State>>>,
    title: String,
    timeline_page: Option<Page<Status>>,
    timeline_items: Arc<RwLock<VecDeque<Status>>>,
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
            timeline_items: Arc::new(RwLock::new(VecDeque::new())),
            status: String::new(),
            list_state: Arc::new(RwLock::new(ListState::default())),
        }
    }

    #[instrument(name = "home", skip_all, err)]
    pub async fn start(&mut self) -> Result<()> {
        let auth = Arc::clone(&self.authentication_data);
        let auth = auth.read().clone().context("not logged in")?;
        let username = auth.account.username;
        info!(username, "logged in");
        let server = auth.config.data.base.trim_start_matches("https://");
        self.title = format!("{username}@{server}");
        self.load_home_timeline().await?;
        Ok(())
    }

    #[instrument(skip_all, fields())]
    async fn load_home_timeline(&mut self) -> Result<()> {
        info!("loading home timeline");

        let auth = Arc::clone(&self.authentication_data);
        let auth = auth.read().clone().context("not logged in")?;
        let mastodon = auth.mastodon;

        let page = mastodon
            .get_home_timeline()
            .await
            .context("failed to load timeline")?;

        info!("loaded home timeline");

        let items = Arc::clone(&self.timeline_items);
        items.write().extend(page.initial_items.clone());
        self.timeline_page = Some(page);
        self.set_list_index(0);
        Ok(())
    }

    #[instrument(name = "home::set_list_index", skip(self))]
    fn set_list_index(&mut self, index: usize) {
        let list_state = Arc::clone(&self.list_state);
        let mut list_state = list_state.write();
        list_state.select(Some(index));
    }

    #[instrument(name = "home::set_list_index", skip(self))]
    fn get_list_index(&self) -> usize {
        let list_state = Arc::clone(&self.list_state);
        let list_state = list_state.read();
        list_state.selected().unwrap_or(0)
    }

    #[instrument(name = "home::handle_event", skip_all)]
    pub async fn handle_event(&mut self, event: &Event) -> Result<Outcome> {
        match event {
            Event::Crossterm(CrosstermEvent::Key(key)) => self.handle_key(&key).await,
            _ => Ok(Outcome::Ignored),
        }
    }

    #[instrument(name = "home::handle_key", skip(self))]
    async fn handle_key(&mut self, key: &KeyEvent) -> Result<Outcome> {
        match (key.modifiers, key.code) {
            (KeyModifiers::NONE, KeyCode::Char('j')) => {
                self.scroll_down().await.context("failed to scroll down")?;
                Ok(Outcome::Handled)
            }
            (KeyModifiers::NONE, KeyCode::Char('k')) => {
                self.scroll_up().await.context("failed to scroll up")?;
                Ok(Outcome::Handled)
            }
            _ => Ok(Outcome::Ignored),
        }
    }

    #[instrument(name = "home::scroll_down", skip(self))]
    async fn scroll_down(&mut self) -> Result<()> {
        let index = self.get_list_index() + 1;
        let items = Arc::clone(&self.timeline_items);
        let mut items = items.write();
        if index >= items.len() {
            let page = self.timeline_page.as_mut().context("no current page")?;
            info!("loading next page. {}", Self::format_page_for_log(&page));
            let result = page.next_page().await.context("failed to load next page")?;
            info!("loaded next page. {}", Self::format_page_for_log(&page));
            let Some(next_items) = result else {
                debug!("attempted to scroll down when there is no next page");
                // returning ok as we just do nothing in this case rather than failing
                return Ok(());
            };
            items.extend(next_items);
        }
        drop(items);
        self.set_list_index(index);
        self.update_status(index)?;
        Ok(())
    }

    fn format_page_for_log(page: &Page<Status>) -> String {
        format!("prev: {} next:{}", page.prev_url(), page.next_url())
    }

    #[instrument(name = "home::scroll_up", skip(self))]
    async fn scroll_up(&mut self) -> Result<()> {
        let mut index = self.get_list_index();
        let items = Arc::clone(&self.timeline_items);
        let mut items = items.write();
        if index == 0 {
            let page = self.timeline_page.as_mut().context("no current page")?;
            info!("loading prev page. {}", Self::format_page_for_log(&page));
            let result = page
                .prev_page()
                .await
                .context("failed to load previous page")?;
            info!("loaded prev page. {}", Self::format_page_for_log(&page));
            let Some(prev_items) = result else {
                debug!("attempted to scroll up when there is no previous page");
                // returning ok as we just do nothing in this case rather than failing
                return Ok(());
            };
            for item in prev_items.iter().rev() {
                items.push_front(item.clone());
            }
            index = prev_items.len().saturating_sub(1);
        } else {
            index -= 1;
        }
        drop(items);
        self.set_list_index(index);
        self.update_status(index)?;
        Ok(())
    }

    #[instrument(name = "home::update_status", skip(self))]
    fn update_status(&mut self, selected: usize) -> Result<()> {
        let items = Arc::clone(&self.timeline_items);
        let items = items.read();
        let status = items.get(selected).context("no item selected")?;
        self.status = status.url.clone().unwrap_or_default();
        Ok(())
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    #[instrument(level = "trace", name = "home::draw", skip_all)]
    pub fn draw(&self, frame: &mut Frame<impl Backend>, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(120), Constraint::Max(0)])
            .split(area);
        let left = layout[0];
        let items = Arc::clone(&self.timeline_items);
        let items = items.read();
        let mut list_items = vec![];
        for status in items.iter() {
            list_items.push(ListItem::new(format_status(status, left.width)));
        }
        // this looks great on a dark theme, but not so much on a light one
        let style = Style::default().bg(Color::Rgb(24, 32, 40));
        let list = List::new(list_items).highlight_style(style);
        // .padding(1)
        // .truncate_last_item(false);
        let list_state = Arc::clone(&self.list_state);
        let mut state = list_state.write();
        frame.render_stateful_widget(list, left, &mut state);
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
        let content = html2text::parse(html.as_bytes())
            .render(width as usize, StyledDecorator::new())
            .into_lines();
        text.extend(to_spans(content));
    }

    // reblogged content
    if let Some(status) = reblog {
        let html = status.content.clone();
        let content = html2text::parse(html.as_bytes())
            .render(width as usize - 2, StyledDecorator::new())
            .into_lines();
        text.extend(to_spans(content));
    }

    // card
    if let Some(card) = &status.card {
        let content = format_card(card, width);
        text.extend(Text::styled(content, Style::default().fg(Color::DarkGray)));
    }
    // blank line until padding is merged in https://github.com/tui-rs-revival/ratatui/pull/150
    text.extend(Text::from(""));
    text
}

#[derive(Debug, Default)]
struct StyledDecorator {
    links: Vec<String>,
}
impl StyledDecorator {
    fn new() -> StyledDecorator {
        Self { links: vec![] }
    }
}

impl TextDecorator for StyledDecorator {
    type Annotation = Style;

    fn decorate_link_start(&mut self, url: &str) -> (String, Self::Annotation) {
        self.links.push(url.to_string());
        (
            "[".to_string(),
            Style::default()
                .fg(Color::Rgb(192, 192, 208))
                .add_modifier(Modifier::UNDERLINED),
        )
    }

    fn decorate_link_end(&mut self) -> String {
        let link = self.links.pop().unwrap();
        format!("]({link})")
    }

    fn decorate_em_start(&mut self) -> (String, Self::Annotation) {
        (
            "*".to_string(),
            Style::default().add_modifier(Modifier::ITALIC),
        )
    }

    fn decorate_em_end(&mut self) -> String {
        "*".to_string()
    }

    fn decorate_strong_start(&mut self) -> (String, Self::Annotation) {
        (
            "**".to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        )
    }

    fn decorate_strong_end(&mut self) -> String {
        "**".to_string()
    }

    fn decorate_strikeout_start(&mut self) -> (String, Self::Annotation) {
        (
            "".to_string(),
            Style::default().add_modifier(Modifier::CROSSED_OUT),
        )
    }

    fn decorate_strikeout_end(&mut self) -> String {
        "".to_string()
    }

    fn decorate_code_start(&mut self) -> (String, Self::Annotation) {
        (
            "`".to_string(),
            Style::default().fg(Color::White).bg(Color::Rgb(0, 16, 32)),
        )
    }

    fn decorate_code_end(&mut self) -> String {
        "`".to_string()
    }

    fn decorate_preformat_first(&mut self) -> Self::Annotation {
        Style::default()
    }
    fn decorate_preformat_cont(&mut self) -> Self::Annotation {
        Style::default()
    }

    fn decorate_image(&mut self, src: &str, title: &str) -> (String, Self::Annotation) {
        (format!("![{title}]({src})"), Style::default())
    }

    fn header_prefix(&mut self, level: usize) -> String {
        "#".repeat(level) + " "
    }

    fn quote_prefix(&mut self) -> String {
        "> ".to_string()
    }

    fn unordered_item_prefix(&mut self) -> String {
        "* ".to_string()
    }

    fn ordered_item_prefix(&mut self, i: i64) -> String {
        format!("{i}. ")
    }

    fn make_subblock_decorator(&self) -> Self {
        StyledDecorator::new()
    }

    fn finalise(self) -> Vec<TaggedLine<Self::Annotation>> {
        vec![]
    }
}

fn to_spans<'a>(lines: Vec<TaggedLine<Vec<Style>>>) -> impl Iterator<Item = Spans<'a>> {
    lines.into_iter().map(|line| {
        let mut spans = vec![];
        for tagged in line.tagged_strings() {
            let mut style = Style::default();
            for annotation in tagged.tag.iter() {
                style = style.patch(*annotation);
            }
            spans.push(Span::styled(tagged.s.clone(), style));
        }
        Spans::from(spans)
    })
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
