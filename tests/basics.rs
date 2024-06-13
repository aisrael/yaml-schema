use cucumber::{gherkin::Step, given, then, World};
use log::debug;
use yaml_schema::{Engine, YamlSchema};

#[derive(Debug, Default, World)]
pub struct BasicsWorld {
    yaml_schema: YamlSchema,
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
    assert!(!accepts(schema, &input));
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
