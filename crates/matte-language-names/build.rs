use bat::assets::HighlightingAssets;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write as _};
use std::path::Path;
use std::{env, iter};
use syntect::parsing::{Scope, SyntaxReference, SyntaxSet};

fn main() {
    println!("cargo::rerun-if-changed=languages.json");

    let languages: HashMap<String, LinguistLanguage> =
        serde_json::from_reader(File::open("languages.json").unwrap()).unwrap();

    let assets = HighlightingAssets::from_binary();
    let syntax_set = assets.get_syntax_set().unwrap();

    let names = build_name_map(languages.iter(), &syntax_set);

    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
    let mut file = BufWriter::new(File::create(&path).unwrap());
    write!(
        &mut file,
        "static LINGUIST_TO_BAT: phf::Map<&'static str, &'static str> = {}",
        to_phf_map(&names).build()
    )
    .unwrap();
    write!(&mut file, ";\n").unwrap();
}

fn build_name_map<'a>(
    linguist_languages: impl Iterator<Item = (&'a String, &'a LinguistLanguage)>,
    syntax_set: &'a SyntaxSet,
) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for (id, language) in linguist_languages {
        if let Some(syntax) = linguist_language_to_bat(id, language, &syntax_set) {
            for name in names(&id, &language) {
                let name = name.to_lowercase();
                let syntax_for_name = syntax_set.find_syntax_by_token(&name);
                match syntax_for_name {
                    Some(_) => {} // Not mapping needed, already covered by bat
                    None => {
                        if map.contains_key(&name) {
                            panic!("Duplicate language name {name}");
                        }
                        map.insert(name, syntax.name.clone());
                    }
                };
            }
        }
    }

    map
}

fn to_phf_map<'a>(input: &'a HashMap<String, String>) -> phf_codegen::Map<&'a str> {
    let mut phf = phf_codegen::Map::new();
    for (key, value) in input {
        phf.entry(key.as_str(), &format!("\"{}\"", value.escape_default()));
    }
    phf
}

fn linguist_language_to_bat<'a>(
    language_id: &str,
    language: &LinguistLanguage,
    syntax_set: &'a SyntaxSet,
) -> Option<&'a SyntaxReference> {
    // bat's JSON syntax supports comments
    if language_id == "JSON with Comments" {
        return Some(
            syntax_set
                .find_syntax_by_extension("json")
                .expect("bat to have a JSON syntax"),
        );
    }

    // Make sure that JavaScript maps to the correct JavaScript language in bat.
    if language_id == "JavaScript" {
        return Some(
            syntax_set
                .find_syntax_by_extension("js")
                .expect("bat to have a JavaScript syntax"),
        );
    }

    syntax_set
        .find_syntax_by_scope(Scope::new(&language.tm_scope).unwrap())
        .or_else(|| syntax_set.find_syntax_by_token(language_id))
}

fn names<'a>(id: &'a str, language: &'a LinguistLanguage) -> impl Iterator<Item = &'a str> {
    // I don't know how accurate this is, but in my testing
    // I've found that GitHub doesn't support looking up identifiers with spaces.
    iter::once(id)
        .filter(|id| !id.chars().any(char::is_whitespace))
        .chain(language.aliases.iter().map(|a| a.as_str()))
}

#[derive(Debug, Deserialize)]
struct LinguistLanguage {
    tm_scope: String,
    #[serde(default)]
    aliases: Vec<String>,
}
