#[macro_use]
extern crate log;

use clipboard::ClipboardContext;
use clipboard::ClipboardProvider;
use druid::im::Vector;
use druid::widget::{Button, CrossAxisAlignment, Flex, Label, LineBreaking, List, Scroll};
use druid::{
    AppDelegate, AppLauncher, Color, Command, Data, DelegateCtx, Env, Handled, Lens,
    LocalizedString, RenderContext, Selector, SingleUse, Target, Widget, WidgetExt, WindowDesc,
};
use futures::{stream, StreamExt};
use jager::stats_processing::CharacterStats;
use reqwest;
use std::boxed::Box;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const PROPERTIES: &[(&str, f64)] = &[("Name", 200.0), ("KDR", 100.0), ("Info", 200.0)];
const HEADER_BACKGROUND: Color = Color::grey8(0xCC);
const JAGER_URL: &str = "http://localhost:8000";

const CLEAR: Selector = Selector::new("CLEAR");
const NEW_CHARACTERS: Selector<Vector<Character>> = Selector::new("NEW_CHARACTERS");
const CHARACTER_UPDATE: Selector<Character> = Selector::new("CHARACTER_UPDATE");

#[derive(Clone, Data, Lens, Debug)]
pub struct Character {
    pub name: String,
    pub valid: bool,
    pub in_progress: bool,
    pub stats: Option<CharacterStats>,
}

impl Character {
    fn get_name(&self) -> String {
        self.name.to_string()
    }
}

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub characters: Vector<Character>,
}

fn make_list_item() -> impl Widget<Character> {
    Flex::row()
        .with_child(
            Label::dynamic(|d: &Character, _| d.get_name())
                .with_text_color(Color::BLACK)
                .fix_width(PROPERTIES[0].1),
        )
        .with_default_spacer()
        .with_child(
            Label::dynamic(|d: &Character, _| {
                if let Some(stats) = &d.stats {
                    format!(
                        "{}:{}",
                        stats.kill_loss_ratio.kills, stats.kill_loss_ratio.losses
                    )
                } else {
                    "".to_string()
                }
            })
            .with_text_color(Color::BLACK)
            .fix_width(PROPERTIES[1].1),
        )
        .with_default_spacer()
        .with_child(
            Label::dynamic(|d: &Character, _| {
                if d.in_progress {
                    "Loading".to_string()
                } else if !d.valid {
                    "Error".to_string()
                } else {
                    "".to_string()
                }
            })
            .with_text_color(Color::RED)
            .fix_width(PROPERTIES[2].1),
        )
        .with_default_spacer()
}

fn ui_character_list() -> impl Widget<AppState> {
    let mut header = Flex::row().with_child(
        Label::new(PROPERTIES[0].0)
            .fix_width(PROPERTIES[0].1)
            .background(HEADER_BACKGROUND),
    );
    for (name, size) in PROPERTIES.iter().skip(1) {
        header.add_default_spacer();
        header.add_child(
            Label::new(*name)
                .fix_width(*size)
                .background(HEADER_BACKGROUND),
        );
    }
    Scroll::new(
        Flex::column()
            .cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(header)
            .with_default_spacer()
            .with_flex_child(
                Scroll::new(List::new(make_list_item).lens(AppState::characters)).vertical(),
                1.0,
            )
            .background(Color::WHITE),
    )
    .horizontal()
    .padding(10.0)
}

fn build_root_widget() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Label::new("CTRL+A, CTRL+C Local to fetch stats"))
                .with_flex_spacer(1.0),
        )
        .with_flex_child(ui_character_list(), 1.0)
}

fn get_characters_from_clipboard(clipboard: &String) -> Vector<Character> {
    let characters: Vector<Character> = clipboard
        .lines()
        .map(|name| Character {
            name: name.to_string().clone(),
            valid: true,
            in_progress: true,
            stats: None,
        })
        .collect();
    characters
}

fn get_character_stats_url(name: &str) -> String {
    format!("{}/character_stats/{}", JAGER_URL, name)
}

async fn get_character_stats(character: Character) -> Character {
    let request_url = get_character_stats_url(&character.name);
    info!("Sending request to {}", request_url);
    match reqwest::get(request_url).await {
        Ok(response) => {
            let stats_result = response.json::<CharacterStats>().await;
            match stats_result {
                Ok(stats) => {
                    info!("Fetched character info for {}", character.name);
                    Character {
                        name: character.name,
                        valid: true,
                        in_progress: false,
                        stats: Some(stats),
                    }
                }
                Err(e) => {
                    error!(
                        "Character deserialize failed for {}: {:?}",
                        character.name, e
                    );
                    Character {
                        name: character.name,
                        valid: false,
                        in_progress: false,
                        stats: None,
                    }
                }
            }
        }
        Err(e) => {
            error!("Cannot fetch character {}: {:?}", character.name, e);
            Character {
                name: character.name,
                valid: false,
                in_progress: false,
                stats: None,
            }
        }
    }
}

async fn get_characters_stats(characters: Vector<Character>, tx_pipe: UnboundedSender<Character>) {
    info!("Starting character info fetch");
    let mut bodies = stream::iter(characters)
        .map(|character| async move {
            info!("Fetching for character {}", character.name);
            let character_state = get_character_stats(character).await;
            character_state
        })
        .buffer_unordered(10);
    while let Some(item) = bodies.next().await {
        tx_pipe.send(item).unwrap();
    }
}

async fn clipboard_watcher(event_sink: druid::ExtEventSink) {
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let mut clip_contents = ctx.get_contents().unwrap();
    loop {
        let new_clip_contents = ctx.get_contents().unwrap();
        // Only do stuff if clipboard contents change
        if new_clip_contents != clip_contents {
            clip_contents = new_clip_contents;
            // Get a list of characters from clipboard (this dosn't check validity yet)
            let characters = get_characters_from_clipboard(&clip_contents);
            let characters_clone = characters.clone();
            // Clear the existing app state
            event_sink.submit_command(CLEAR, (), Target::Auto).unwrap();
            // Submit list of new characters to the UI
            event_sink
                .submit_command(NEW_CHARACTERS, characters, Target::Auto)
                .unwrap();
            // Set up channel to receive character stats on asynchronously
            let (tx, mut rx): (UnboundedSender<Character>, UnboundedReceiver<Character>) =
                unbounded_channel();
            // Start fetching character stats
            tokio::task::spawn(async move { get_characters_stats(characters_clone, tx).await });
            // Listen for character stat update
            while let Some(character) = rx.recv().await {
                event_sink
                    .submit_command(CHARACTER_UPDATE, character, Target::Auto)
                    .unwrap();
            }
        }
    }
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> Handled {
        info!("Got command {:?}", cmd);
        if let Some(characters) = cmd.get(NEW_CHARACTERS) {
            data.characters = characters.clone();
            return Handled::Yes;
        }
        if let Some(character) = cmd.get(CHARACTER_UPDATE) {
            let mut iter_clone = data.characters.clone();
            let mut char_list = data.characters.clone();
            if let Some(char_index) = iter_clone
                .into_iter()
                .position(|li| li.name == character.name)
            {
                char_list.set(char_index, character.clone());
                data.characters = char_list
            }
            return Handled::Yes;
        }
        if let Some(_) = cmd.get(CLEAR) {
            data.characters = Vector::new();
            return Handled::Yes;
        }
        Handled::No
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
    let clip_contents = ctx.get_contents().unwrap();
    println!("Contents: {}", clip_contents);
    let characters = get_characters_from_clipboard(&clip_contents);
    let initial_state: AppState = AppState { characters };
    let window = WindowDesc::new(build_root_widget)
        .window_size((400.0, 800.0))
        .title("Jager");
    let launcher = AppLauncher::with_window(window).delegate(Delegate {});
    let event_sink = launcher.get_external_handle();
    tokio::task::spawn(async move { clipboard_watcher(event_sink).await });
    launcher.launch(initial_state);
}
