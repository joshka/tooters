use crate::{
    component::{EventOutcome, RootComponent},
    event::{Event, Events},
    ui::UI,
};
use crossterm::event::{
    Event::Key,
    KeyCode::{self, Char},
};
use tracing::{debug, info, trace};

pub async fn run() -> anyhow::Result<()> {
    let mut app = App::default();
    app.run().await?;
    Ok(())
}

struct App {
    events: Events,
    ui: UI,
    root: RootComponent,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let events = Events::new();
        let root = RootComponent::new(events.tx.clone());
        Self {
            events,
            ui: UI::new(),
            root,
        }
    }
    pub async fn run(&mut self) -> anyhow::Result<()> {
        info!("Running");
        self.ui.start()?;
        self.events.start()?;
        self.root.start().await;
        loop {
            self.ui.draw(|f| {
                self.root.draw(f, f.size());
            })?;
            match self.events.next().await {
                Some(Event::Quit) => {
                    info!("Received quit event");
                    break;
                }
                Some(Event::Tick) => {
                    trace!("Received tick event");
                    self.root.handle_event(&Event::Tick).await;
                }
                Some(event) => {
                    if self.root.handle_event(&event).await == EventOutcome::Consumed {
                        debug!("Event consumed by main component");
                        continue;
                    }
                    if let Event::CrosstermEvent(Key(key)) = event {
                        if key.code == Char('q') || key.code == KeyCode::Esc {
                            debug!("Received quit key");
                            break;
                        }
                    }
                }
                _ => {
                    debug!("Received unknown event");
                }
            }
        }
        debug!("Shutting down");
        Ok(())
    }
}
// impl Tooters {
//     pub fn new(tui: Tui) -> Self {
//         let view = Arc::new(Mutex::new(View::Login(LoginView::new())));
//         let (sender, receiver) = mpsc::channel(100);
//         Self {
//             tui,
//             view,
//             sender,
//             receiver,
//         }
//     }

//     pub async fn start(&mut self) -> anyhow::Result<()> {
//         info!("Starting app");
//         self.tui.start()?;

//         let tick_handle = spawn_tick_task(self.sender.clone());
//         let keyboard_handle = spawn_keyboard_task(self.sender.clone());

//         // run the view logic
//         self.handle_events().await?;
//         // view_task.await?;

//         self.drain_events().await?;
//         keyboard_handle.abort();
//         tick_handle.abort();
//         Ok(())
//     }

//     fn set_view(&mut self, new_view: View) {
//         debug!(?new_view, "changing view");
//         let view = Arc::clone(&self.view);
//         *view.lock().unwrap() = new_view;
//     }

//     /// Handle events.
//     /// This is the main event loop.
//     async fn handle_events(&mut self) -> anyhow::Result<()> {
//         while let Some(event) = self.receiver.recv().await {
//             debug!(?event, "Handling event");
//             match event {
//                 Event::Tick => {}
//                 Event::LoggedIn(login_details) => {
//                     self.set_view(View::Home(HomeView::from(login_details)));
//                 }
//                 Event::LoggedOut => {
//                     self.set_view(View::Login(LoginView::new()));
//                 }
//                 Event::CrosstermEvent(event) => {
//                     if let CrosstermEvent::Key(key_event) = event {
//                         if key_event.code == KeyCode::Esc {
//                             break;
//                         }
//                         self.test_changing_views(key_event);
//                     }
//                     // let mut view = Arc::clone(self.view);
//                     // let mut view = view.lock();
//                     // view.handle_event(event);
//                 } // Event::MastodonError(_err) => {}
//             }
//             self.draw()?;
//         }
//         Ok(())
//     }

//     fn test_changing_views(&mut self, key_event: crossterm::event::KeyEvent) {
//         // Just for testing
//         match key_event.code {
//             KeyCode::Char('l') => {
//                 self.set_view(View::Login(LoginView::new()));
//             }
//             KeyCode::Char('h') => {
//                 let data = Data {
//                     base: "https://example.com".into(),
//                     client_id: "adbc01234".into(),
//                     client_secret: "0987dcba".into(),
//                     redirect: "urn:ietf:wg:oauth:2.0:oob".into(),
//                     token: "fedc5678".into(),
//                 };
//                 let mastodon_client = mastodon_async::Mastodon::from(data);
//                 let home_view = HomeView {
//                     username: "test".into(),
//                     url: "https://example.com".into(),
//                     mastodon_client,
//                     timeline: None,
//                     selected: 0,
//                     status: "Testing...".into(),
//                 };
//                 self.set_view(View::Home(home_view));
//             }
//             _ => {}
//         }
//     }

//     // fn run_view(&mut self) {
//     // *current_view = view;
//     // let sender = self.sender.clone();
//     // let view = self.view.clone();
//     // tokio::spawn(async move {
//     //     let mut view = view.lock().await;
//     //     view.run(sender).await.unwrap_or_default()
//     // })
//     // }

//     /// Drain the event queue.
//     /// This is used to ensure that all events are processed before exiting.
//     async fn drain_events(&mut self) -> anyhow::Result<()> {
//         self.receiver.close();
//         while (self.receiver.recv().await).is_some() {}
//         Ok(())
//     }
// }
