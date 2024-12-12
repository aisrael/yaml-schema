use cucumber::{gherkin::Step, given, then, World};
use log::{debug, error};
use std::cell::RefCell;
use std::rc::Rc;
use yaml_schema::validation::ValidationError;
use yaml_schema::{Engine, RootSchema, YamlSchema};

#[derive(Debug, Default, World)]
pub struct BasicsWorld {
    root_schema: RootSchema,
    yaml_schema_error: Option<yaml_schema::Error>,
    errors: Option<Rc<RefCell<Vec<ValidationError>>>>,
}

#[given(regex = "a YAML schema:")]
async fn a_yaml_schema(world: &mut BasicsWorld, step: &Step) {
    let schema = step.docstring().unwrap();
    debug!("schema: {:?}", schema);
    match RootSchema::load_from_str(schema) {
        Ok(root_schema) => world.root_schema = root_schema,
        Err(e) => {
            error!("Error: {:?}", e);
            world.yaml_schema_error = Some(e);
        }
    }
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
    let schema = &world.root_schema.schema;
    assert!(accepts(schema, &input));
}

#[then(regex = "it should NOT accept:")]
async fn it_should_not_accept(world: &mut BasicsWorld, step: &Step) {
    let raw_input = step.docstring().unwrap();
    debug!("raw_input: {:?}", raw_input);
    let input: serde_yaml::Value = serde_yaml::from_str(raw_input).unwrap();
    debug!("input: {:?}", input);
    let schema = &world.root_schema.schema;
    debug!("schema: {:?}", schema);
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
        assert_eq!(expected_error_message, yaml_schema_error.to_string());
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
