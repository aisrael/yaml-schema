use cucumber::{gherkin::Step, given, then, World};
use log::debug;
use yaml_schema::{Engine, YamlSchema};

#[derive(Debug, Default, World)]
pub struct BasicsWorld {
    yaml_schema: YamlSchema,
    yaml_schema_error: Option<yaml_schema::YamlSchemaError>,
}

#[given(regex = "a YAML schema:")]
async fn a_yaml_schema(world: &mut BasicsWorld, step: &Step) {
    let schema = step.docstring().unwrap();
    debug!("schema: {:?}", schema);
    let yaml_schema: YamlSchema = serde_yaml::from_str(schema).unwrap();
    world.yaml_schema = yaml_schema;
}

fn accepts(schema: &YamlSchema, value: &serde_yaml::Value) -> bool {
    let engine = Engine::new(schema);
    match engine.evaluate(value) {
        Ok(_) => true,
        Err(e) => {
            debug!("Error: {:?}", e);
            false
        }
    }
}

#[then(regex = "it should accept:")]
async fn it_should_accept(world: &mut BasicsWorld, step: &Step) {
    let raw_input = step.docstring().unwrap();
    debug!("raw_input: {:?}", raw_input);
    let input: serde_yaml::Value = serde_yaml::from_str(raw_input).unwrap();
    debug!("input: {:?}", input);
    let schema = &world.yaml_schema;
    assert!(accepts(schema, &input));
}

#[then(regex = "it should NOT accept:")]
async fn it_should_not_accept(world: &mut BasicsWorld, step: &Step) {
    let raw_input = step.docstring().unwrap();
    debug!("raw_input: {:?}", raw_input);
    let input: serde_yaml::Value = serde_yaml::from_str(raw_input).unwrap();
    debug!("input: {:?}", input);
    let schema = &world.yaml_schema;
    let engine = Engine::new(schema);
    match engine.evaluate(&input) {
        Ok(_) => panic!("Validation succeeded when it was expected to fail!"),
        Err(e) => {
            debug!("Error: {:?}", e);
            world.yaml_schema_error = Some(e);
        }
    }
}

#[then(regex = "the error should be a (GenericError)")]
async fn the_error_should_be_a(world: &mut BasicsWorld, error: String) {
    let actual_error = world.yaml_schema_error.as_ref().unwrap();
    match error.as_str() {
        "GenericError" => assert!(matches!(
            actual_error,
            yaml_schema::YamlSchemaError::GenericError(_)
        )),
        "NotYetImplemented" => assert!(matches!(
            actual_error,
            yaml_schema::YamlSchemaError::NotYetImplemented
        )),
        "NotYetImplemented error" => assert!(matches!(
            actual_error,
            yaml_schema::YamlSchemaError::NotYetImplemented
        )),
        _ => panic!("Unexpected error: {:?}", error),
    };
}

#[then(expr = "the error message should be {string}")]
fn the_error_message_should_be(world: &mut BasicsWorld, expected_error_message: String) {
    if let Some(error) = world.yaml_schema_error.as_ref() {
        match error {
            yaml_schema::YamlSchemaError::GenericError(actual_error_message) => {
                assert_eq!(expected_error_message, *actual_error_message)
            }
            yaml_schema::YamlSchemaError::NotYetImplemented => {
                panic!("NotYetImplemented error does not have an error message!")
            }
            _ => panic!("Unexpected error: {:?}", error),
        }
    } else {
        panic!("Expected an error message, but there was no error!");
    }
}

#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .format_target(false)
        .format_timestamp_secs()
        .target(env_logger::Target::Stdout)
        .init();

    BasicsWorld::run("features/basics.feature").await;
}
