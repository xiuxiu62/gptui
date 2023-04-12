mod config;
mod io;
mod prompt;

use chatgpt::{
    prelude::{ChatGPT, Conversation},
    types::{ChatMessage, ResponseChunk},
};
use clap::Parser;
use colored::Colorize;
use config::Config;
use futures_util::{Future, StreamExt, TryStreamExt};
use prompt::Prompt;
use reedline::{Reedline, Signal};
use std::path::PathBuf;

const COMMANDS: &str = "
commands:
    help - displays this message
    exit - exits the application";

pub type DynResult<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> DynResult<()> {
    let args = Args::parse();
    let config = match args.config() {
        Some(config) => config,
        None => Config::generate()?,
    };

    run(config).await
}

async fn run(config: Config) -> DynResult<()> {
    let mut line_editor = Reedline::create();
    let prompt = Prompt::default();
    let client = ChatGPT::new(config.api_key())?;
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

// async fn run(config: Config) -> DynResult<()> {
//     let user = whoami::username();
//     let client = ChatGPT::new(&config.api_key)?;
//     let mut conversation = client.new_conversation();

//     loop {
//         let mut request = "".to_owned();

//         print!("{}: ", user.cyan());
//         flush()?;
//         io::stdin().read_line(&mut request)?;

//         query(&config, &mut conversation, &request).await?;
//     }
// }

async fn query(config: &Config, conversation: &mut Conversation, request: &str) -> DynResult<()> {
    async fn write_chunk(
        mut delta_accumulator: Vec<ResponseChunk>,
        chunk: ResponseChunk,
    ) -> DynResult<ResponseChunk> {
        if let ResponseChunk::Content {
            delta,
            response_index: _,
        } = &chunk
        {
            print!("{delta}");
            let _ = io::flush();
        }

        delta_accumulator.push(chunk);

        delta_accumulator
    }

    async fn try_write_chunk(chunk: ResponseChunk) -> ResponseChunk {
        if let ResponseChunk::Content {
            delta,
            response_index: _,
        } = &chunk
        {
            print!("{delta}");
            io::flush();
        }

        chunk
    }

    print!("{}: ", config.ai_name().green());
    // let chunks = conversation
    //     .send_message_streaming(request)
    //     .await?
    //     .fold(vec![], write_chunk)
    //     .await;

    let chunks = conversation
        .send_message_streaming(request)
        .await?
        .map(try_write_chunk)
        // .buffered(8)
        .collect::<Vec<ResponseChunk>>();
    println!();
    // .collect();
    // .collect::<DynResult<Vec<ResponseChunk>>>();

    if let Some(message) = ChatMessage::from_response_chunks(chunks).first() {
        conversation.history.push(message.to_owned());
    }

    Ok(())
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
            .unwrap_or_else(config::default_path);

        path.exists().then(|| Config::try_from(path).ok()).flatten()
    }
}

#[inline]
fn system_println(message: &str) {
    println!("{}: {message}", "system".red());
}
