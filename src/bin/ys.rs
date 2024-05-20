use yaml_rust2::{YamlEmitter, YamlLoader};

use yaml_schema::{YamlSchema, YamlSchemaError};

fn main() {
    let docs = YamlLoader::load_from_str("[1, 2, 3]").unwrap();
    for doc in docs {
        println!("{:?}", &doc);
    }
}
