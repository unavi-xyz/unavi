use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let protocol_dir = manifest_dir.join("../../protocol");

    // Tell cargo to rerun if schemas or protocols change.
    println!(
        "cargo:rerun-if-changed={}",
        protocol_dir.join("dwn/schemas").display()
    );
    println!(
        "cargo:rerun-if-changed={}",
        protocol_dir.join("dwn/protocols").display()
    );

    // Generate types from schemas.
    generate_schema_types(&protocol_dir, &out_dir);

    // Generate constants from protocols and schemas.
    generate_constants(&protocol_dir, &out_dir);
}

fn generate_schema_types(protocol_dir: &Path, out_dir: &Path) {
    let schema_dir = protocol_dir.join("dwn/schemas");
    let mut schemas = Vec::new();

    // Collect all schemas with titles derived from filenames.
    for entry in fs::read_dir(&schema_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let schema_content = fs::read_to_string(&path).unwrap();
            let mut schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();

            // Add title from filename if not present.
            if schema.get("title").is_none() {
                let title = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .split('-')
                    .map(|s| {
                        let mut chars = s.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => {
                                first.to_uppercase().collect::<String>() + chars.as_str()
                            }
                        }
                    })
                    .collect::<String>();
                schema
                    .as_object_mut()
                    .unwrap()
                    .insert("title".to_string(), serde_json::Value::String(title));
            }

            schemas.push(schema);
        }
    }

    // Generate the types.
    let mut type_space = typify::TypeSpace::default();
    for schema in schemas {
        let schema: schemars::schema::Schema = serde_json::from_value(schema).unwrap();
        type_space.add_type(&schema).unwrap();
    }
    let contents = prettyplease::unparse(&syn::parse2(type_space.to_stream()).unwrap());

    fs::write(out_dir.join("types.rs"), contents).unwrap();
}

fn generate_constants(protocol_dir: &PathBuf, out_dir: &Path) {
    let schema_dir = protocol_dir.join("dwn/schemas");
    let protocol_files_dir = protocol_dir.join("dwn/protocols");

    let mut schema_urls = Vec::new();
    let mut protocol_urls = Vec::new();
    let mut protocol_defs = Vec::new();

    // Collect schema URLs.
    for entry in fs::read_dir(&schema_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let schema_content = fs::read_to_string(&path).unwrap();
            let schema: serde_json::Value = serde_json::from_str(&schema_content).unwrap();

            if let Some(id) = schema.get("$id").and_then(|v| v.as_str()) {
                let name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_uppercase()
                    .replace('-', "_");
                schema_urls.push((name, id.to_string()));
            }
        }
    }

    // Collect protocol URLs and definitions.
    for entry in fs::read_dir(&protocol_files_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let protocol_content = fs::read_to_string(&path).unwrap();
            let protocol: serde_json::Value = serde_json::from_str(&protocol_content).unwrap();

            if let Some(url) = protocol.get("protocol").and_then(|v| v.as_str()) {
                let name = path
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_uppercase()
                    .replace('-', "_");

                let relative_path = path
                    .strip_prefix(protocol_dir)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/");

                protocol_urls.push((name.clone(), url.to_string()));
                protocol_defs.push((name, relative_path));
            }
        }
    }

    // Generate schemas.rs.
    let mut schemas_rs = String::from("// Schema URLs.\n\n");
    for (name, url) in &schema_urls {
        schemas_rs.push_str(&format!("pub const {}_SCHEMA: &str = \"{}\";\n", name, url));
    }
    fs::write(out_dir.join("schemas.rs"), schemas_rs).unwrap();

    // Generate protocols.rs.
    let mut protocols_rs = String::from("// Protocol URLs and embedded definitions.\n\n");
    for (name, url) in &protocol_urls {
        protocols_rs.push_str(&format!(
            "pub const {}_PROTOCOL: &str = \"{}\";\n",
            name, url
        ));
    }
    protocols_rs.push('\n');
    for (name, path) in &protocol_defs {
        protocols_rs.push_str(&format!(
            "pub const {}_DEFINITION: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../protocol/{}\"));\n",
            name, path
        ));
    }
    fs::write(out_dir.join("protocols.rs"), protocols_rs).unwrap();
}
