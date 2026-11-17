//! Each layer of the crate uses only the layers before it.

use std::{
    fs,
    path::{
        Path,
        PathBuf,
    },
};

const LAYERS: [&str; 7] = [
    "tokens",
    "material",
    "part",
    "face",
    "machinery",
    "component",
    "api",
];

fn rank(name: &str) -> Option<usize> {
    LAYERS.iter().position(|layer| *layer == name)
}

fn identifier(text: &str) -> &str {
    let end = text
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(text.len());
    &text[..end]
}

/// The first segment of every `crate::` path in `text`, comments excluded,
/// including each entry of a `crate::{ .. }` group.
fn referenced(text: &str) -> Vec<String> {
    let code = text
        .lines()
        .map(|line| line.find("//").map_or(line, |at| &line[..at]))
        .collect::<Vec<_>>()
        .join("\n");

    let mut found = Vec::new();
    let mut rest = code.as_str();
    while let Some(at) = rest.find("crate::") {
        rest = &rest[at + "crate::".len()..];
        let Some(group) = rest.strip_prefix('{') else {
            found.push(identifier(rest).to_string());
            continue;
        };

        let mut depth = 1;
        let mut expecting = true;
        let mut index = 0;
        while depth > 0 && index < group.len() {
            let tail = &group[index..];
            let Some(c) = tail.chars().next() else {
                break;
            };
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                ',' if depth == 1 => expecting = true,
                c if c.is_whitespace() => {}
                _ if expecting && depth == 1 => {
                    let name = identifier(tail);
                    found.push(name.to_string());
                    expecting = false;
                    index += name.len().max(1);
                    continue;
                }
                _ => {}
            }
            index += c.len_utf8();
        }
    }
    found
}

/// The layers `text`, written in `layer`, reaches above its own.
fn violations(layer: &str, text: &str) -> Vec<String> {
    let Some(own) = rank(layer) else {
        return Vec::new();
    };
    referenced(text)
        .into_iter()
        .filter(|name| rank(name).is_some_and(|other| other > own))
        .collect()
}

fn sources(dir: &Path, out: &mut Vec<PathBuf>) -> anyhow::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            sources(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

#[test]
fn no_layer_uses_one_above_it() -> anyhow::Result<()> {
    let src = src();
    let mut files = Vec::new();
    sources(&src, &mut files)?;

    let mut broken = Vec::new();
    for file in files {
        let relative = file.strip_prefix(&src)?;
        let Some(layer) = relative
            .components()
            .next()
            .and_then(|c| c.as_os_str().to_str())
        else {
            continue;
        };
        for name in violations(layer, &fs::read_to_string(&file)?) {
            broken.push(format!("{} uses crate::{name}", relative.display()));
        }
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
    Ok(())
}

#[test]
fn every_source_directory_is_a_layer() -> anyhow::Result<()> {
    for entry in fs::read_dir(src())? {
        let path = entry?.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str());
        assert!(
            name.is_some_and(|name| rank(name).is_some()),
            "{} is not a layer",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn a_path_to_a_higher_layer_is_caught() {
    assert_eq!(
        violations("part", "use crate::component::tree::Mote;"),
        ["component"]
    );
    assert_eq!(
        violations(
            "material",
            "use crate::{\n    tokens::palette::Palette,\n    part::{\n        draw,\n        mesh::{self, X},\n    },\n    wired::Y,\n};",
        ),
        ["part"]
    );
    assert_eq!(
        violations("part", "let x = crate::part::draw::hidden();"),
        Vec::<String>::new()
    );
    assert_eq!(
        violations("part", "// see crate::api::Vui"),
        Vec::<String>::new()
    );
}
