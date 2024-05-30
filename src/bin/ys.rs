use clap::{Parser, Subcommand};
use yaml_rust2::{YamlEmitter, YamlLoader};
use yaml_schema::{YamlSchema, YamlSchemaError};

use yaml_schema::version;

#[derive(Parser, Debug)]
#[command(name = "ys")]
#[command(author = "Alistair Israel <aisrael@gmail.com>")]
#[command(version = clap::crate_version!())]
#[command(about = "A tool for validating YAML against a schema")]
pub struct Opts {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Display the ys version")]
    Version,
}

fn main() {
    let opts = Opts::parse();
    match opts.command {
        Commands::Version => {
            println!("ys {}", version());
        }
    }
    let docs = YamlLoader::load_from_str("[1, 2, 3]").unwrap();
    for doc in docs {
        println!("{:?}", &doc);
    }
}
