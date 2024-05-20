use yaml_rust2::{YamlLoader, YamlEmitter};

use yaml_schema::{YamlSchema, YamlSchemaError};


fn main() {
    let docs = YamlLoader::load_from_str("[1, 2, 3]").unwrap();
    for doc in docs {
        println!("{:?}", &doc);
    }
}
