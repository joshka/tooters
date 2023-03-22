use crossterm::event::{KeyEvent, EventStream, KeyCode, KeyModifiers};
use futures::StreamExt;
use tokio::{sync::mpsc, time::interval};

use crate::view::{LoginView, View};

pub async fn run() -> AppResult<()> {
    let mut app = App::new();
    crossterm::terminal::enable_raw_mode()?;
    let mut key_events = EventStream::new();
    loop {
        tokio::select! {
            _ = app.run() => {
                println!("App exited");
                break;
            }
            Some(Ok(crossterm::event::Event::Key(key_event))) = key_events.next() => {
                match (key_event.modifiers, key_event.code) {
                    (crossterm::event::KeyModifiers::CONTROL, KeyCode::Char('c')) |
                    (KeyModifiers::NONE, KeyCode::Char('q'))
                    => {
                        app.tx.send(Event::Quit).await?;
                    }
                    _ => {}
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, PartialEq)]
pub enum Event {
    Tick,
    Start,
    Quit,
    Key(KeyEvent),
    LoggedIn,
    LoggedOut,
}

pub struct App {
    rx: mpsc::Receiver<Event>,
    tx: mpsc::Sender<Event>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> App {
        let (tx, rx) = mpsc::channel(100);
        Self { rx, tx }
    }

    pub async fn run(&mut self) -> AppResult<()> {
        self.start().await?;
        self.handle_events().await?;
        self.drain_events().await?;
        Ok(())
    }

    async fn start(&self) -> AppResult<()> {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(crate::TICK_DURATION);
            loop {
                interval.tick().await;
                if let Err(err) = tx.send(Event::Tick).await {
                    eprintln!("Error sending tick: {}", err);
                }
            }
        });
        self.tx.send(Event::Start).await?;
        Ok(())
    }

    async fn drain_events(&mut self) -> AppResult<()> {
        self.rx.close();
        while (self.rx.recv().await).is_some() {}
        Ok(())
    }

    async fn handle_events(&mut self) -> AppResult<()> {
        while let Some(event) = self.rx.recv().await {
            match event {
                Event::Tick => {
                    println!("Tick");
                }
                Event::Quit => {
                    println!("Quit");
                    break;
                }
                Event::Start => {
                    println!("Start");
                    let view = View::Login(LoginView::new(self.tx.clone()));
                    view.run().await;
                }
                Event::LoggedOut => {
                    println!("Logged out");
                }
                Event::LoggedIn => {
                    println!("Logged in");
                }
                Event::Key(event) => {
                    println!("Key: {:?}", event);
                },
            }
        }
        Ok(())
    }
}

// pub async fn run(&mut self) -> AppResult<()> {
//     // let mut events = EventStream::new();
//     // let mut interval = interval(Duration::from_millis(1000));
//     // let handle = self.current_view.run();
//     // tokio::spawn(async move { handle.await; });
//     loop {
//         tokio::select! {
//             _ = interval.tick() => {
//                 self.draw().await?;
//             },
//             event = events.next() => {
//                 if let Some(Ok(Event::Key(key_event))) = event {
//                     if key_event.code == KeyCode::Char('q') {
//                         self.tx.send(Event::Quit).await?;
//                     }
//                 }
//             },
//             app_event = self.rx.recv() => {
//                 match app_event {
//                     Some(Event::Quit) => break,
//                     Some(Event::LoggedIn(username)) => {
//                         self.current_view = Box::new(LoggedInView::new(username));
//                         self.start = Instant::now();
//                     },
//                     Some(Event::LoggedOut(reason)) => {
//                         self.current_view = Box::new(LoggedOutView::new(reason));
//                         self.start = Instant::now();
//                     },
//                     None => break,
//                 }
//             }
//         }
//     }
//     Ok(())
// }

// pub async fn draw(&mut self) -> AppResult<()> {
//     self.ui.draw(|frame| {
//         let size = frame.size();
//         let layout = Layout::default()
//             .direction(Direction::Vertical)
//             .constraints([
//                 Constraint::Length(1),
//                 Constraint::Min(1),
//                 Constraint::Length(1),
//             ])
//             .split(size);

//         let text = Spans::from(vec![
//             Span::styled("Tooters", Style::default().add_modifier(Modifier::BOLD)),
//             Span::raw(" | "),
//             Span::styled(self.current_view.title(), Style::default().fg(Color::Gray)),
//         ]);
//         let title_bar =
//             Paragraph::new(text).style(Style::default().fg(Color::White).bg(Color::Blue));
//         frame.render_widget(title_bar, layout[0]);

//         let output = Paragraph::new(format!(
//             "Elapsed: {:?} millis",
//             self.start.elapsed().as_millis()
//         ));
//         frame.render_widget(output, layout[1]);
//     })?;
//     Ok(())
// }
// }
