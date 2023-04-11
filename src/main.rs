use chatgpt::{
    prelude::{ChatGPT, Conversation},
    types::{ChatMessage, ResponseChunk},
};
use clap::Parser;
use colored::{Color, ColoredString, Colorize};
use futures_util::StreamExt;
use reedline::{PromptEditMode, PromptHistorySearchStatus, PromptViMode, Reedline, Signal};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fs::{self, File},
    io::{self, Write},
    path::PathBuf,
};

// const AI_NAME: &str = "hiro";
// const DEFAULT_CONFIG_PATH: &str = "/home/xiuxiu/.config/gptui/config.json";
//
const COMMANDS: &str = "
commands:
    help - displays this message
    exit - exits the application";

type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

pub static PROMPT_INDICATOR: &str = ": ";
pub static MULTILINE_INDICATOR: &str = "::: ";
pub static VI_INSERT_PROMPT_INDICATOR: &str = "[i]: ";
pub static VI_NORMAL_PROMPT_INDICATOR: &str = "[n]: ";

struct Prompt(ColoredString);

impl Prompt {
    pub fn new(username: &str, foreground: Color) -> Self {
        Self(username.color(foreground))
    }
}

impl Default for Prompt {
    fn default() -> Self {
        Self::new(whoami::username().as_str(), Color::Cyan)
    }
}

impl reedline::Prompt for Prompt {
    fn render_prompt_left(&self) -> Cow<str> {
        Cow::Borrowed(&self.0)
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode) -> Cow<str> {
        match edit_mode {
            PromptEditMode::Default | PromptEditMode::Emacs => PROMPT_INDICATOR.into(),
            PromptEditMode::Vi(vi_mode) => match vi_mode {
                PromptViMode::Normal => VI_NORMAL_PROMPT_INDICATOR.into(),
                PromptViMode::Insert => VI_INSERT_PROMPT_INDICATOR.into(),
            },
            PromptEditMode::Custom(str) => format!("({str})").into(),
        }
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed(MULTILINE_INDICATOR)
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: reedline::PromptHistorySearch,
    ) -> std::borrow::Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };

        // NOTE: magic strings, given there is logic on how these compose I am not sure if it
        // is worth extracting in to static constant
        Cow::Owned(format!(
            "({}reverse-search: {}) ",
            prefix, history_search.term
        ))
    }

    fn get_prompt_color(&self) -> reedline::Color {
        reedline::Color::Cyan
    }

    fn get_indicator_color(&self) -> reedline::Color {
        reedline::Color::White
    }
}

#[tokio::main]
async fn main() -> DynResult<()> {
    let args = Args::parse();
    let config = match args.config() {
        Some(config) => config,
        None => Config::generate()?,
    };

    // let args = Args::parse();
    // let config = match args.config() {
    //     Some(config) => config,
    //     None => Config::generate()?,
    // };

    // run(config).await
    //
    example(config).await
}

async fn example(config: Config) -> DynResult<()> {
    let mut line_editor = Reedline::create();
    let prompt = Prompt::default();
    let client = ChatGPT::new(&config.api_key)?;
    let mut conversation = client.new_conversation();

    system_println(COMMANDS);
    query(
        &config,
        &mut conversation,
        &format!(
            "My name is {}, please refer to me as that as often as makes conversation sense.
             Your name is {}, introduce yourself.",
            whoami::username(),
            &config.ai_name(),
        ),
    )
    .await?;

    loop {
        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buffer)) => match buffer.as_str() {
                "help" => system_println(COMMANDS),
                "exit" => {
                    query(&config, &mut conversation, "Goodbye").await?;
                    break;
                }
                request => {
                    query(&config, &mut conversation, &request).await?;
                }
            },
            Ok(signal) if matches!(signal, Signal::CtrlC | Signal::CtrlD) => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

#[inline]
fn system_println(message: &str) {
    println!("{}: {message}", "system".red());
}

// fn example_code_parse() -> DynResult<()> {
//     let src = "fn main() {
//     println!(\"hello world\");

//     let syntax_set = SyntaxSet::load_defaults_newlines();
//     let theme_set = ThemeSet::load_defaults();
//     let mut lines = src.lines().peekable();
// }";

//     let syntax_set = SyntaxSet::load_defaults_newlines();
//     let theme_set = ThemeSet::load_defaults();
//     let mut lines = src.lines().peekable();

//     println!("{:#?}", theme_set.themes.keys());

//     // for line in lines {
//     //     for region in hi
//     // }
//     //
//     //

//     // syntax_set.find_syntax_plain_text();
//     // src.find_syntax_plain_text();
//     // syntax_set.find_syntax_by_first_line(src.lines().collect::<Vec<&str>>()[0])
//     //
//     let syntax = syntax_set.find_syntax_by_extension("rs").unwrap();
//     let mut theme = theme_set.themes["base16-eighties.dark"].clone();
//     theme.scopes.iter_mut().for_each(|scope| {
//         scope.style.background = Some(syntect::highlighting::Color {
//             r: 0x00,
//             g: 0x00,
//             b: 0x00,
//             a: 0x00,
//         })
//     });
//     // let mut theme = theme_set.themes["Solarized (dark)"].clone()
//     // theme.settings.background = Some(syntect::highlighting::Color {
//     //     r: 0x00,
//     //     g: 0x00,
//     //     b: 0x00,
//     //     a: 0xFF,
//     // });

//     let mut highlighter = HighlightLines::new(syntax, &theme);
//     let output = src
//         .lines()
//         .into_iter()
//         .map(|line| {
//             println!("{line}");

//             highlighter
//                 .highlight_line(line, &syntax_set)
//                 .unwrap()
//                 .into_iter()
//                 .collect::<Vec<(Style, &str)>>()
//         })
//         .collect::<Vec<Vec<(Style, &str)>>>();
//     // };
//     //

//     for regions in output {
//         let parsed = syntect::util::as_24_bit_terminal_escaped(&regions, true);
//         println!("{parsed}");
//     }

//     Ok(())
// }

async fn run(config: Config) -> DynResult<()> {
    let user = whoami::username();
    let client = ChatGPT::new(&config.api_key)?;
    // let client = ChatGPT::new(API_KEY)?;
    let mut conversation = client.new_conversation();

    // query(
    //     &config,
    //     &mut conversation,
    //     &format!(
    //         "My name is {user}, please refer to me as that as often as makes conversation sense.
    //          Your name is {}, introduce yourself.",
    //         config.ai_name(),
    //     ),
    // )
    // .await?;

    loop {
        let mut request = "".to_owned();

        print!("{}: ", user.cyan());
        flush()?;
        io::stdin().read_line(&mut request)?;

        query(&config, &mut conversation, &request).await?;
    }
}

async fn query(config: &Config, conversation: &mut Conversation, request: &str) -> DynResult<()> {
    async fn write_chunk(
        mut delta_accumulator: Vec<ResponseChunk>,
        chunk: ResponseChunk,
    ) -> Vec<ResponseChunk> {
        if let ResponseChunk::Content {
            delta,
            response_index: _,
        } = &chunk
        {
            print!("{delta}");
            flush().unwrap();
        }
        delta_accumulator.push(chunk);

        delta_accumulator
    }

    print!("{}: ", config.ai_name().green());
    let chunks = conversation
        .send_message_streaming(request)
        .await?
        .fold(vec![], write_chunk)
        .await;
    println!();

    // TODO: remove unwrap.  Simply don't push the history if it hasn't been parsed properly
    let message = ChatMessage::from_response_chunks(chunks)
        .first()
        .unwrap()
        .to_owned();
    conversation.history.push(message);

    Ok(())
}

fn flush() -> io::Result<()> {
    io::stdout().lock().flush()
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

impl Args {
    pub fn config(&self) -> Option<Config> {
        let path = self
            .config
            .as_ref()
            .cloned()
            .unwrap_or_else(default_config_path);

        path.exists().then(|| Config::try_from(path).ok()).flatten()
    }
}

// #[derive(Debug, Clone, Copy, Deserialize)]
// pub enum Color {
//     Black,
//     Red,
//     Green,
//     Yellow,
//     Blue,
//     Magenta,
//     Cyan,
//     White,
//     BrightBlack,
//     BrightRed,
//     BrightGreen,
//     BrightYellow,
//     BrightBlue,
//     BrightMagenta,
//     BrightCyan,
//     BrightWhite,
//     TrueColor { r: u8, g: u8, b: u8 },
// }

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    api_key: String,
    conversation_file: Option<PathBuf>,
    ai_name: Option<String>,
    // ai_color: Option<Color>,
}

impl Config {
    pub fn generate() -> DynResult<Self> {
        // Prompt for api key
        let mut api_key = "".to_owned();
        print!("{}: ", "Enter your api key".purple());
        flush().unwrap();
        io::stdin().read_line(&mut api_key).unwrap();

        // Create config
        let config = Self {
            api_key: api_key.trim_end().to_owned(),
            conversation_file: None,
            ai_name: None,
        };

        // Open file or create if it doesn't exist
        let path = default_config_path();
        let mut file = match File::create(&path).ok() {
            Some(file) => file,
            None => {
                fs::create_dir_all(path.parent().unwrap())?;
                File::create(path)?
            }
        };

        // Serialize and write config
        let contents = serde_json::to_string_pretty(&config)?;
        write!(file, "{contents}\n")?;

        Ok(config)
    }

    pub fn ai_name(&self) -> &str {
        self.ai_name
            .as_ref()
            .map(|name| name.as_str())
            .unwrap_or("gpt")
    }
}

impl TryFrom<PathBuf> for Config {
    type Error = io::Error;

    fn try_from(path: PathBuf) -> io::Result<Self> {
        let data = fs::read_to_string(path)?;
        let inner = serde_json::from_str(&data)?;

        Ok(inner)
    }
}

fn default_config_path() -> PathBuf {
    PathBuf::from(format!(
        "/home/{}/.config/gptui/config.json",
        whoami::username()
    ))
}
