/*
 * arguments.rs -- Contains code related to command-line argument parsing.
 */
use clap::Parser;

#[derive(Parser, Debug)]
pub struct Args {
    // Whether or not to show the debug menu
    #[arg(short, long)]
    pub debug: bool,

    // The name of the file containing the program
    pub program: String,
}
