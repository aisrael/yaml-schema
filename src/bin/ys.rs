use clap::{Parser, Subcommand};
use yaml_schema::{Engine, YamlSchema};

use yaml_schema::version;

#[derive(Parser, Debug, Default)]
#[command(name = "ys")]
#[command(author = "Alistair Israel <aisrael@gmail.com>")]
#[command(version = clap::crate_version!())]
#[command(about = "A tool for validating YAML against a schema")]
#[command(arg_required_else_help = true)]
pub struct Opts {
    /// The command to run
    #[command(subcommand)]
    pub command: Option<Commands>,
    /// The schema to validate against
    #[arg(short = 'f', long = "schema")]
    pub schemas: Vec<String>,
    /// The YAML file to validate
    pub file: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Display the ys version")]
    Version,
}

fn main() {
    let opts = Opts::parse();
    if let Some(comand) = opts.command {
        match comand {
            Commands::Version => {
                println!("ys {}", version());
            }
        }
    } else {
        match command_validate(opts) {
            Ok(_) => {
                println!("Validation successful");
            }
            Err(e) => {
                eprintln!("Validation failed: {}", e);
            }
        }
    }
}

/// The `ys validate` command
fn command_validate(opts: Opts) -> Result<(), anyhow::Error> {
    // Currently, we only support a single schema file
    // TODO: Support multiple schema files
    let schema_file = std::fs::File::open(opts.schemas.first().unwrap())?;
    let schema: YamlSchema = serde_yaml::from_reader(schema_file)?;
    let engine = Engine::new(&schema);
    let yaml_file = std::fs::File::open(opts.file.unwrap())?;
    let yaml: serde_yaml::Value = serde_yaml::from_reader(yaml_file)?;
    match engine.evaluate(&yaml) {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
