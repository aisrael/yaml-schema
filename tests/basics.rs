use log::{debug, error};
use std::cell::RefCell;
use std::rc::Rc;

use cucumber::{gherkin::Step, given, then, World};
use yaml_schema::engine::ValidationError;
use yaml_schema::{Engine, YamlSchema};

#[derive(Debug, Default, World)]
pub struct BasicsWorld {
    yaml_schema: YamlSchema,
    yaml_schema_error: Option<yaml_schema::YamlSchemaError>,
    errors: Option<Rc<RefCell<Vec<ValidationError>>>>,
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
    match engine.evaluate(value, true) {
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
    match engine.evaluate(&input, true) {
        Ok(context) => {
            assert!(
                context.has_errors(),
                "Validation succeeded when it was expected to fail!"
            );
            world.errors = Some(context.errors.clone());
        }
        Err(e) => {
            error!("Error: {:?}", e);
            world.yaml_schema_error = Some(e);
        }
    }
}

#[then(expr = "the error message should be {string}")]
fn the_error_message_should_be(world: &mut BasicsWorld, expected_error_message: String) {
    let errors = world.errors.as_ref().unwrap().borrow();
    if !errors.is_empty() {
        let first_error = errors.first().unwrap();
        let actual_error_message = format!(".{}: {}", first_error.path, first_error.error);
        assert_eq!(actual_error_message, expected_error_message);
    } else {
        panic!("Expected an error message, but there was no error!");
    }
}

#[then(expr = "it should fail with {string}")]
async fn it_should_fail_with(world: &mut BasicsWorld, expected_error_message: String) {
    if let Some(yaml_schema_error) = world.yaml_schema_error.as_ref() {
        match yaml_schema_error {
            yaml_schema::YamlSchemaError::GenericError(actual_error_message) => {
                debug!("expected_error_message: {:?}", expected_error_message);
                debug!("actual_error_message: {:?}", actual_error_message);
                assert_eq!(expected_error_message, *actual_error_message)
            }
            yaml_schema::YamlSchemaError::NotYetImplemented => {
                assert_eq!(expected_error_message, "a NotYetImplemented error");
            }
            _ => panic!("Unexpected error: {:?}", yaml_schema_error),
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
    BasicsWorld::run("features/validation/arrays.feature").await;
    BasicsWorld::run("features/validation/booleans.feature").await;
    BasicsWorld::run("features/validation/const.feature").await;
    BasicsWorld::run("features/validation/enums.feature").await;
    BasicsWorld::run("features/validation/nulls.feature").await;
    BasicsWorld::run("features/validation/numbers.feature").await;
    BasicsWorld::run("features/validation/objects.feature").await;
    BasicsWorld::run("features/validation/strings.feature").await;
    BasicsWorld::run("features/composition.feature").await;
}
