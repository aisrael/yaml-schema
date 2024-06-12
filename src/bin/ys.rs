use clap::{Parser, Subcommand};
use yaml_schema::{Engine, YamlSchema};

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
    let yaml: serde_yaml::Value = serde_yaml::from_str(r#""hello""#).unwrap();
    let schema = YamlSchema::new();
    let binding = Some(schema);
    let engine = Engine::new(&binding);

    engine.evaluate(&yaml).unwrap();
}
