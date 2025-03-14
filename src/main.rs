use chrono::Local;
use clap::{Args, Parser, Subcommand};
use env_logger::Builder;
use log::LevelFilter;
use log::*;
use rusty_radamsa;
use std::io::Write;

// a comment

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "Rusty Radamsa is a general purpose fuzzer. It modifies given sample data
in ways, which might expose errors in programs intended to process
the data. For more information, read the fine manual page, or visit
https://gitlab.com/akihe/radamsa.

Rusty Radamsa was written by Amanda Rousseau (malwareunicorn), based on Radams by Aki Helin, initially at OUSPG."
)]
struct Cli {
    /// random seed (u64, default random)
    #[arg(short, long)]
    seed: Option<u64>,
    /// how many outputs to generate (u64)
    #[arg(short = 'n', long)]
    count: Option<usize>,
    /// hash algorithm for uniqueness checks (default sha256)
    #[arg(short = 'H', long, default_value_t = String::from("default"))]
    hash: String,
    /// which mutation patterns to use (use list command to see all hashes)
    #[arg(short, long, default_value_t = String::from("default"))]
    patterns: String,
    /// which mutations to use (use list command to see all mutations)
    #[arg(short, long, default_value_t = String::from("default"))]
    mutators: String,
    /// which data generators to use (use list command to see all generators)
    #[arg(short, long, default_value_t = String::from("default"))]
    generators: String,
    /// output pattern
    #[arg(short, long, num_args(1..3))]
    output: Option<Vec<String>>,
    /// maximum number of checksums in uniqueness filter (0 disables)
    #[arg(short = 'C', long, default_value_t = 10000)]
    checksums: usize,
    /// sleep for n milliseconds between outputs
    #[arg(short, long, default_value_t = 0)]
    delay: usize,
    // TODO: metadata
    // TODO: recursive
    /// take only first n bytes of each output (mainly intended for UDP).]
    /// if truncate is zero, no truncation happens.
    #[arg(short = 'T', long, default_value_t = 0)]
    truncate: usize,
    /// start from given testcase
    #[arg(short = 'S', long, default_value_t = 0)]
    seek: usize,
    /// show progress during generation
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
    #[command(subcommand)]
    command: Option<Commands>,
    /// file or directory as generator input.
    /// example: "./input/* test.bin"
    #[arg(value_name = "FILE", num_args(0..))]
    file: Option<Vec<String>>,
}

#[derive(Subcommand)]
enum Commands {
    /// list mutations, patterns and generators
    List(ListArgs),
}

#[derive(Args, Debug)]
struct ListArgs {
    /// List all mutations, generators, patterns, hashes
    #[arg(short, long, default_value_t = false)]
    all: bool,
    /// List mutators
    #[arg(short, long, default_value_t = false)]
    mutators: bool,
    /// List generators
    #[arg(short, long, default_value_t = false)]
    generators: bool,
    /// List patterns
    #[arg(short, long, default_value_t = false)]
    patterns: bool,
    /// List hash types
    #[arg(short = 'H', long, default_value_t = false)]
    hashes: bool,
    /// List output options
    #[arg(short, long, default_value_t = false)]
    outputs: bool,
}

fn match_lists(cmd: &Commands) {
    match &cmd {
        Commands::List(list) => {
            if list.mutators || list.all {
                println!("MUTATIONS:\n----------");
                println!("  DEFAULT: {}", rusty_radamsa::mutations::DEFAULT_MUTATIONS);
                let mutations = rusty_radamsa::mutations::init_mutations();
                mutations.iter().for_each(|(_, x)| {
                    println!("    {0: <6} {1: <10}", x.id(), x.info());
                });
                println!("---");
            }
            if list.generators || list.all {
                println!("GENERATORS:\n----------");
                println!(
                    "  DEFAULT: {}",
                    rusty_radamsa::generators::DEFAULT_GENERATORS
                );
                let mutations = rusty_radamsa::generators::init_generators();
                mutations.iter().for_each(|x| {
                    println!("    {0: <6} {1: <10}", x.gen_type.id(), x.gen_type.info())
                });
                println!("---");
            }
            if list.patterns || list.all {
                println!("PATTERNS:\n----------");
                println!("  DEFAULT: {}", rusty_radamsa::patterns::DEFAULT_PATTERNS);
                let mutations = rusty_radamsa::patterns::init_patterns();
                mutations.iter().for_each(|x| {
                    println!(
                        "    {0: <6} {1: <10}",
                        x.pattern_type.id(),
                        x.pattern_type.info()
                    )
                });
                println!("---");
            }
            if list.hashes || list.all {
                println!("HASHES:\n----------");
                println!("  DEFAULT: sha256");
                let mutations = rusty_radamsa::digest::init_digests();
                mutations
                    .iter()
                    .for_each(|x| println!("    {0: <6} {1: <10}", x.id, x.desc));
                println!("---");
            }
            if list.outputs || list.all {
                println!("OUTPUTS:\n----------");
                println!("  DEFAULT: -");
                let mutations = rusty_radamsa::output::init_outputs();
                mutations
                    .iter()
                    .for_each(|x| println!("    {0: <10} {1: <10}", x.id, x.desc));
                println!("---");
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    if let Some(ref cmd) = cli.command {
        match_lists(cmd);
        return;
    }

    let mut radamsa = match cli.seed {
        Some(s) => rusty_radamsa::Radamsa::new_with_seed(s),
        None => rusty_radamsa::Radamsa::new(),
    };
    radamsa.init();
    radamsa.verbose = cli.verbose;
    if cli.verbose {
        radamsa.verbose = cli.verbose;
        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Debug)
            .init();
    } else {
        Builder::new()
            .format(|buf, record| {
                writeln!(
                    buf,
                    "{} [{}] - {}",
                    Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.args()
                )
            })
            .filter(None, LevelFilter::Error)
            .init();
    }
    if let Some(n) = cli.count {
        radamsa.count = n;
    }

    radamsa.set_checksum(&cli.hash).expect("bad input");
    radamsa.set_patterns(&cli.patterns).expect("bad input");
    radamsa.set_mutators(&cli.mutators).expect("bad input");
    radamsa.set_generators(&cli.generators).expect("bad input");
    if let Some(o) = cli.output {
        debug!("{:#?}", o);
        let outputs: Vec<&str> = o.iter().map(|s| &**s).collect();
        radamsa.set_output(outputs).expect("bad input");
    } else {
        debug!("o is empty");
        radamsa.set_output(vec!["default"]).expect("bad input");
    }
    let all_paths = match cli.file {
        Some(files) => rusty_radamsa::shared::get_files(files).ok(),
        None => None,
    };

    radamsa.checksum_max(cli.checksums);
    radamsa.delay = cli.delay;
    radamsa.truncate(cli.truncate);
    radamsa.offset = cli.seek;
    debug!("Seed {}", radamsa.seed);
    let len = radamsa.fuzz(None, all_paths, None).unwrap_or(0);
    debug!("TOTAL LEN = {}", len);
}
