use arboard::Clipboard;
use enigo::{Enigo, Key, KeyboardControllable};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use iced::{
    alignment, executor,
    widget::{button, column, container, scrollable, text, Row, Space},
    window::{self, Position},
    Application, Command, Element, Length, Settings, Subscription, Theme,
};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    env,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::sync::{mpsc, watch};
const MAX_HISTORY_SIZE: usize = 50;
const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 500;
const CLIPBOARD_CHECK_INTERVAL: Duration = Duration::from_millis(100);

mod daemon;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClipboardEntry {
    content: String,
    timestamp: u64,
}

#[derive(Debug, Clone)]
enum Message {
    ClipboardUpdated(String),
    SelectEntry(usize),
    HotkeyPressed,
    EventReceived(Event),
    ToggleWindow,
}

#[derive(Debug, Clone)]
enum Event {
    ClipboardChanged(String),
    HotkeyTriggered,
}

struct MacClip {
    entries: VecDeque<ClipboardEntry>,
    clipboard: Arc<Mutex<Clipboard>>,
    storage_path: PathBuf,
    hotkey_manager: Arc<GlobalHotKeyManager>,
    event_rx: watch::Receiver<Option<Event>>,
    tx: mpsc::UnboundedSender<Event>,
    last_clipboard_content: String,
    window_visible: bool,
}

impl Application for MacClip {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        env_logger::init();
        info!("Initializing Mac-Clip");

        let storage_dir = directories::ProjectDirs::from("com", "mac-clip", "mac-clip")
            .expect("Failed to get project directory")
            .data_dir()
            .to_path_buf();

        fs::create_dir_all(&storage_dir).expect("Failed to create storage directory");
        let storage_path = storage_dir.join("history.json");

        let entries = if storage_path.exists() {
            info!("Loading clipboard history from {}", storage_path.display());
            let data = fs::read_to_string(&storage_path).expect("Failed to read history file");
            serde_json::from_str(&data).unwrap_or_else(|_| VecDeque::new())
        } else {
            info!("No existing clipboard history found");
            VecDeque::new()
        };

        let clipboard = Arc::new(Mutex::new(
            Clipboard::new().expect("Failed to initialize clipboard"),
        ));
        let hotkey_manager =
            Arc::new(GlobalHotKeyManager::new().expect("Failed to initialize hotkey manager"));

        let hotkey = HotKey::new(Some(Modifiers::META | Modifiers::ALT), Code::KeyV);
        hotkey_manager
            .register(hotkey)
            .expect("Failed to register hotkey");
        info!("Registered global hotkey: Command + Option + V");

        let (tx, mut rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = watch::channel(None);
        let tx_clone = tx.clone();

        // Event processor thread
        let event_tx_clone = event_tx.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                while let Some(event) = rx.recv().await {
                    let _ = event_tx_clone.send(Some(event));
                }
            });
        });

        // Hotkey listener thread
        std::thread::spawn(move || {
            info!("Starting hotkey listener thread");
            for event in GlobalHotKeyEvent::receiver() {
                if let global_hotkey::HotKeyState::Pressed = event.state {
                    info!("Hotkey pressed");
                    let _ = tx_clone.send(Event::HotkeyTriggered);
                }
            }
        });

        // Clipboard monitor thread
        let clipboard_clone = Arc::clone(&clipboard);
        let tx_clipboard = tx.clone();
        thread::spawn(move || {
            info!("Starting clipboard monitor thread");
            let mut last_content = String::new();
            loop {
                thread::sleep(CLIPBOARD_CHECK_INTERVAL);

                if let Ok(mut clipboard) = clipboard_clone.lock() {
                    if let Ok(content) = clipboard.get_text() {
                        if !content.is_empty() && content != last_content {
                            info!("Detected clipboard change: {}", content);
                            last_content = content.clone();
                            let _ = tx_clipboard.send(Event::ClipboardChanged(content));
                        }
                    }
                }
            }
        });

        let last_clipboard_content = clipboard.lock().unwrap().get_text().unwrap_or_default();

        info!("Initial clipboard content: {}", last_clipboard_content);

        (
            MacClip {
                entries,
                clipboard,
                storage_path,
                hotkey_manager,
                event_rx,
                tx,
                last_clipboard_content,
                window_visible: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Mac Clip")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::EventReceived(event) => {
                match event {
                    Event::ClipboardChanged(content) => {
                        info!("Processing clipboard change");
                        if content.trim().is_empty() {
                            return Command::none();
                        }

                        let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();

                        if self.entries.front().map(|e| &e.content) != Some(&content) {
                            let entry = ClipboardEntry {
                                content: content.clone(),
                                timestamp,
                            };

                            self.entries.push_front(entry);
                            if self.entries.len() > MAX_HISTORY_SIZE {
                                self.entries.pop_back();
                            }

                            if let Ok(json) = serde_json::to_string(&self.entries) {
                                if let Err(e) = fs::write(&self.storage_path, json) {
                                    error!("Failed to save history: {}", e);
                                }
                            }
                        }
                    }
                    Event::HotkeyTriggered => {
                        info!("Processing hotkey event");
                        self.window_visible = !self.window_visible;
                        return Command::perform(async {}, |_| Message::ToggleWindow);
                    }
                }
                Command::none()
            }
            Message::ClipboardUpdated(content) => {
                if let Ok(mut clipboard) = self.clipboard.lock() {
                    let _ = clipboard.set_text(&content);
                }
                Command::none()
            }
            Message::SelectEntry(index) => {
                info!("Selected entry at index {}", index);
                if let Some(entry) = self.entries.get(index) {
                    let content = entry.content.clone();
                    self.window_visible = false;

                    // First update the clipboard content
                    if let Ok(mut clipboard) = self.clipboard.lock() {
                        if let Err(e) = clipboard.set_text(&content) {
                            error!("Failed to set clipboard content: {}", e);
                        } else {
                            info!("Set clipboard content from history");
                            self.last_clipboard_content = content.clone();

                            // Then simulate Command+V to paste
                            let mut enigo = Enigo::new();
                            enigo.key_down(Key::Meta);
                            enigo.key_click(Key::Layout('v'));
                            enigo.key_up(Key::Meta);
                        }
                    }
                }
                Command::batch(vec![Command::perform(async {}, |_| Message::ToggleWindow)])
            }
            Message::HotkeyPressed => {
                self.window_visible = !self.window_visible;
                Command::perform(async {}, |_| Message::ToggleWindow)
            }
            Message::ToggleWindow => {
                if !self.window_visible {
                    Command::batch(vec![
                        window::change_mode(window::Mode::Hidden),
                    ])
                } else {
                    Command::batch(vec![
                        window::change_mode(window::Mode::Windowed),
                        window::gain_focus(),
                    ])
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        if !self.window_visible {
            return container(Space::new(Length::Fill, Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        info!("Rendering window with {} entries", self.entries.len());
        let mut content = column![].spacing(5).padding(10);

        content = content.push(
            text("Clipboard History")
                .size(18)
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Center),
        );

        content = content.push(Space::new(Length::Fill, Length::Fixed(5.0)));

        if self.entries.is_empty() {
            content = content.push(
                container(
                    text("No clipboard history yet. Copy some text!")
                        .width(Length::Fill)
                        .size(14)
                        .horizontal_alignment(alignment::Horizontal::Center),
                )
                .padding(10)
                .style(iced::theme::Container::Box),
            );
        } else {
            for (i, entry) in self.entries.iter().enumerate() {
                let entry_text = if entry.content.len() > 50 {
                    format!("{}...", &entry.content[..50].replace('\n', "↵"))
                } else {
                    entry.content.replace('\n', "↵")
                };

                let entry_row = Row::new().push(
                    button(
                        text(&entry_text)
                            .size(12)
                            .horizontal_alignment(alignment::Horizontal::Left),
                    )
                    .width(Length::Fill)
                    .padding(8)
                    .style(iced::theme::Button::Secondary)
                    .on_press(Message::SelectEntry(i)),
                );

                content = content.push(entry_row);
            }
        }

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(10)
            .style(iced::theme::Container::Box)
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        struct EventReceiver;

        let rx = self.event_rx.clone();
        iced::subscription::unfold(
            std::any::TypeId::of::<EventReceiver>(),
            rx,
            move |mut rx| async move {
                if let Ok(Some(event)) = rx.changed().await.map(|_| rx.borrow().clone()) {
                    (Message::EventReceived(event), rx)
                } else {
                    (Message::HotkeyPressed, rx) // Dummy message that won't be used
                }
            },
        )
    }
}

fn main() -> iced::Result {
    env_logger::init();

    // Check if --daemon flag is provided
    if env::args().any(|arg| arg == "--daemon") {
        if let Err(e) = daemon::setup_daemon() {
            eprintln!("Failed to setup daemon: {}", e);
        } else {
            println!("Mac-Clip daemon setup complete. The application will now start automatically when you log in.");
            return Ok(());
        }
    }

    MacClip::run(Settings {
        window: window::Settings {
            size: (WINDOW_WIDTH, WINDOW_HEIGHT),
            position: Position::Centered,
            visible: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}
