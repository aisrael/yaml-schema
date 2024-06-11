use cucumber::{gherkin::Step, given, then, World};
use log::debug;
use yaml_schema::YamlSchema;

#[derive(Debug, Default, World)]
pub struct BasicsWorld {
    yaml_schema: Option<YamlSchema>,
}

#[given(regex = "a YAML schema:")]
async fn a_yaml_schema(world: &mut BasicsWorld, step: &Step) {
    let schema = step.docstring().unwrap();
    debug!("schema: {:?}", schema);
    let yaml_schema: YamlSchema = serde_yaml::from_str(&schema).unwrap();
    world.yaml_schema = Some(yaml_schema);
}

#[then(regex = "it should accept:")]
async fn it_should_accept(world: &mut BasicsWorld, step: &Step) {
    let raw_input = step.docstring().unwrap();
    debug!("raw_input: {:?}", raw_input);
    let input: serde_yaml::Value = serde_yaml::from_str(&raw_input).unwrap();
    debug!("input: {:?}", input);
    let schema = world.yaml_schema.as_ref().unwrap();
    assert!(schema.accepts(&input));
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
