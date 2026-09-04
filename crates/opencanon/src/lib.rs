mod commands;
mod envelope;
mod error;
mod map_error;
mod now;
mod stdin;

use std::ffi::OsString;
use std::io::IsTerminal;
use std::str::FromStr;

use canon_core::ops::ListFilter;
use canon_core::Status;
use canon_store::Store;
use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand};
use serde_json::Value;

use crate::error::CliError;

const HELP_TEXT: &str = "\
opencanon --version
opencanon init
opencanon add                                          # stdin: JSON array of {slug, title, body, tags?, freshness?}
opencanon get <id>
opencanon list [--status draft|active|deprecated] [--all]
opencanon edit                                         # stdin: JSON array of {id, title?, tags?, body?, freshness?}
opencanon delete <id>
opencanon active <id>
opencanon query <keyword>... [--status draft|active|deprecated] [--all]
opencanon freshness [id...]                            # omit = all active
opencanon compose                                      # stdin: JSON object {slug, title, atoms, body}
opencanon help";

#[derive(Parser)]
#[command(
    name = "opencanon",
    disable_help_subcommand = true,
    disable_version_flag = true,
    color = clap::ColorChoice::Never
)]
struct Cli {
    /// Print the binary version and exit.
    #[arg(long = "version")]
    version: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add,
    Init,
    Get {
        id: String,
    },
    List {
        #[arg(long, value_parser = parse_status)]
        status: Option<Status>,
        #[arg(long, conflicts_with = "status")]
        all: bool,
    },
    Edit,
    Delete {
        id: String,
    },
    Active {
        id: String,
    },
    Query {
        #[arg(required = true, num_args = 1.., value_name = "KEYWORD")]
        keywords: Vec<String>,
        #[arg(long, value_parser = parse_status)]
        status: Option<Status>,
        #[arg(long, conflicts_with = "status")]
        all: bool,
    },
    Freshness {
        #[arg(num_args = 0.., value_name = "ID")]
        ids: Vec<String>,
    },
    Compose,
    Help,
}

fn list_filter(status: Option<Status>, all: bool) -> ListFilter {
    if all {
        ListFilter::All
    } else if let Some(status) = status {
        ListFilter::Status(status)
    } else {
        ListFilter::Active
    }
}

fn parse_status(s: &str) -> Result<Status, String> {
    Status::from_str(s)
}

pub fn run<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err) => {
            let _ = err.print();
            return match err.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => 0,
                _ => 2,
            };
        }
    };

    if cli.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        return 0;
    }

    let Some(command) = cli.command else {
        let mut cmd = Cli::command();
        let err = cmd.error(ErrorKind::MissingSubcommand, "a subcommand is required");
        let _ = err.print();
        return 2;
    };

    if matches!(command, Commands::Help) {
        println!("{HELP_TEXT}");
        return 0;
    }

    let command_name = command_name(&command);
    if matches!(command, Commands::Init) && !std::io::stdin().is_terminal() {
        eprintln!("opencanon init requires an interactive terminal");
        return 2;
    }
    match dispatch(command) {
        Ok(data) => {
            envelope::write_ok(command_name, data);
            0
        }
        Err(err) => {
            envelope::write_err(command_name, err);
            1
        }
    }
}

fn command_name(command: &Commands) -> &'static str {
    match command {
        Commands::Add => "add",
        Commands::Init => "init",
        Commands::Get { .. } => "get",
        Commands::List { .. } => "list",
        Commands::Edit => "edit",
        Commands::Delete { .. } => "delete",
        Commands::Active { .. } => "active",
        Commands::Query { .. } => "query",
        Commands::Freshness { .. } => "freshness",
        Commands::Compose => "compose",
        Commands::Help => "help",
    }
}

fn dispatch(command: Commands) -> Result<Value, CliError> {
    let cwd = std::env::current_dir().map_err(|e| CliError::Io {
        message: e.to_string(),
    })?;
    let store = Store::open(cwd);
    match command {
        Commands::Add => commands::add(&store),
        Commands::Init => commands::init(&store),
        Commands::Get { id } => commands::get(&store, &id),
        Commands::List { status, all } => commands::list(&store, list_filter(status, all)),
        Commands::Edit => commands::edit(&store),
        Commands::Delete { id } => commands::delete(&store, &id),
        Commands::Active { id } => {
            let now = now::resolve_now()?;
            commands::active(&store, now, &id)
        }
        Commands::Query {
            keywords,
            status,
            all,
        } => commands::query(&store, &keywords, list_filter(status, all)),
        Commands::Freshness { ids } => commands::freshness(&store, &ids),
        Commands::Compose => commands::compose(&store),
        Commands::Help => unreachable!("help is handled before dispatch"),
    }
}
