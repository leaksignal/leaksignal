use std::{
    env,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use jsonschema::JSONSchema;
use leakpolicy::{yaml_parser::ParseYaml, Policy};

/// attempts to get the path to the root folder of the leaksignal workspace.
/// this will panic if you run the binary outside of cargo, but its fine for testing
pub fn workspace_root_dir() -> PathBuf {
    let project_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&project_dir).parent().unwrap().parent().unwrap();
    assert!(
        (path.ends_with("leaksignal") || path.ends_with("leaksignal-public")) && path.exists(),
        "invalid workspace root `{}`. (try running tester through cargo)",
        path.display()
    );
    path.into()
}

fn main() {
    println!("starting");
    let p = workspace_root_dir();
    let schema = read_to_string(p.join("policy.schema.json")).unwrap();
    let schema = serde_json::from_str(&schema).unwrap();
    let schema = JSONSchema::compile(&schema).unwrap();
    let p = p.join("examples/policies");
    for entry in std::fs::read_dir(p).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|v| v == "yaml") {
            println!("checking {}", path.display());
            let policy = read_to_string(&path).unwrap();
            let policy = Policy::parse_str(&policy, true).unwrap();
            let policy = serde_json::to_value(policy).unwrap();
            let result = schema.validate(&policy);
            if let Err(e) = result {
                println!("{}\n\n\n", serde_json::to_string_pretty(&policy).unwrap());
                for e in e {
                    println!("{e}: {e:#?}\n");
                }
                panic!("failed to validate schema for {}", path.display());
            }
        }
    }
}
