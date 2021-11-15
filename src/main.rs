mod api;
mod commands;
mod formats;
mod iotools;

use commands::Command;
use structopt::StructOpt;

fn main() -> anyhow::Result<()> {
    commands::Main::from_args().execute()
}
