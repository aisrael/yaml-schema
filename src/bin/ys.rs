use clap::{Parser, Subcommand};
use eyre::{Context, Result};
use yaml_schema::deser::Deser;
use yaml_schema::{deser, YamlSchema};
use yaml_schema::{version, Engine};

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
    /// Specify this flag to exit (1) as soon as any error is encountered
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

/// The main entrypoint function of the ys executable
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
fn command_validate(opts: Opts) -> Result<i32> {
    // Currently, we only support a single schema file
    // TODO: Support multiple schema files
    if opts.schemas.is_empty() {
        return Err(eyre::eyre!("No schema file(s) specified"));
    }
    let schema_filename = opts.schemas.first().unwrap();
    let schema_file = std::fs::File::open(schema_filename)
        .wrap_err_with(|| format!("Failed to open schema file: {}", schema_filename))?;
    let deserialized_representation: deser::YamlSchema = serde_yaml::from_reader(schema_file)
        .wrap_err_with(|| format!("Failed to read YAML schema file: {}", schema_filename))?;
    let schema: YamlSchema = deserialized_representation
        .deserialize()
        .wrap_err_with(|| {
            format!(
                "Failed to deserialize schema file to a YamlSchema model: {}",
                schema_filename
            )
        })?;

    if opts.file.is_none() {
        return Err(eyre::eyre!("No YAML file specified"));
    }

    let engine = Engine::new(&schema);
    let yaml_filename = opts.file.as_ref().unwrap();
    let yaml_file = std::fs::File::open(yaml_filename)
        .wrap_err_with(|| format!("Failed to open YAML file: {}", yaml_filename))?;
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
