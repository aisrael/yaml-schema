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
    /// The schema to validate against
    #[arg(long = "fail-fast", default_value = "false")]
    pub fail_fast: bool,
    /// The YAML file to validate
    pub file: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Display the ys version")]
    Version,
}

fn main() {
    env_logger::init();
    let opts = Opts::parse();
    if let Some(comand) = opts.command {
        match comand {
            Commands::Version => {
                println!("ys {}", version());
            }
        }
    } else {
        match command_validate(opts) {
            Ok(return_code) => {
                std::process::exit(return_code);
            }
            Err(e) => {
                eprintln!("Validation failed: {}", e);
                std::process::exit(1);
            }
        }
    }
}

/// The `ys validate` command
fn command_validate(opts: Opts) -> Result<i32, anyhow::Error> {
    // Currently, we only support a single schema file
    // TODO: Support multiple schema files
    let schema_file = std::fs::File::open(opts.schemas.first().unwrap())?;
    let schema: YamlSchema = serde_yaml::from_reader(schema_file)?;
    let engine = Engine::new(&schema);
    let yaml_file = std::fs::File::open(opts.file.unwrap())?;
    let yaml: serde_yaml::Value = serde_yaml::from_reader(yaml_file)?;
    match engine.evaluate(&yaml, opts.fail_fast) {
        Ok(context) => {
            let errors = context.errors.borrow();
            if errors.is_empty() {
                println!("Validation successful");
                Ok(0)
            } else {
                let error_messages: Vec<String> = errors
                    .iter()
                    .map(|validation_error| {
                        format!("{}: {}", validation_error.path, validation_error.error)
                    })
                    .collect();
                println!("Validation encountered errors:");
                for error in error_messages {
                    println!("  {}", error);
                }
                Ok(1)
            }
        }
        Err(e) => Err(e.into()),
    }
}
