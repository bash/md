use bat::assets::HighlightingAssets;

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

/// Resolves a markdown language specifier to a language
/// name that can be passed to `bat`.
pub fn lookup_bat_language<'a>(token: &'a str, assets: &'a HighlightingAssets) -> Option<&'a str> {
    lookup(token, assets)
        .or_else(|| without_modifier(token).and_then(|l| lookup(l, assets)))
        .or_else(|| lookup_by_extension(token, assets))
}

fn lookup<'a>(token: &'a str, assets: &'a HighlightingAssets) -> Option<&'a str> {
    lookup_in_syntax_set(token, assets).or_else(|| lookup_linguist_language(token))
}

fn lookup_linguist_language(token: &str) -> Option<&'static str> {
    LINGUIST_TO_BAT.get(&token.to_lowercase()).copied()
}

fn lookup_in_syntax_set<'a, 'b>(token: &'a str, assets: &'b HighlightingAssets) -> Option<&'b str> {
    let syntax_set = assets.get_syntax_set().ok()?;
    let syntax = syntax_set.find_syntax_by_token(token)?;
    Some(&syntax.name)
}

fn lookup_by_extension<'a, 'b>(token: &'a str, assets: &'b HighlightingAssets) -> Option<&'b str> {
    let (_, extension) = token.rsplit_once('.')?;
    let syntax_set = assets.get_syntax_set().ok()?;
    let syntax = syntax_set.find_syntax_by_extension(extension)?;
    Some(&syntax.name)
}

fn without_modifier(language: &str) -> Option<&str> {
    // Some languages can have modifiers
    // after a comma e.g. `rust,no_run`
    language
        .split_once(',')
        .map(|(language, _modifier)| language)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_bat_language() {
        let assets = HighlightingAssets::from_binary();
        for language in bat_language_names(&assets) {
            assert_eq!(Some(language), lookup_bat_language(&language, &assets));
            assert_eq!(
                Some(language),
                lookup_bat_language(&language.to_uppercase(), &assets)
            );
            assert_eq!(
                Some(language),
                lookup_bat_language(&language.to_lowercase(), &assets)
            );
        }
    }

    #[test]
    fn finds_language_by_extension() {
        let assets = HighlightingAssets::from_binary();
        assert_eq!(Some("TOML"), lookup_bat_language("Cargo.toml", &assets));
    }

    #[test]
    fn finds_language_with_modifier() {
        let assets = HighlightingAssets::from_binary();
        assert_eq!(Some("Rust"), lookup_bat_language("rust,no_run", &assets));
    }

    #[test]
    fn finds_linguist_language() {
        let assets = HighlightingAssets::from_binary();

        const ARBITRARILY_PICKED_EXAMPLE: &str = "regexp";

        // Let's ensure that our example makes sense by ensuring
        // that it would not be found by bat itself
        assert!(assets
            .get_syntax_set()
            .unwrap()
            .find_syntax_by_token(ARBITRARILY_PICKED_EXAMPLE)
            .is_none());

        assert_eq!(
            Some("Regular Expression"),
            lookup_bat_language(ARBITRARILY_PICKED_EXAMPLE, &assets)
        );
    }

    fn bat_language_names<'a>(assets: &'a HighlightingAssets) -> impl Iterator<Item = &'a str> {
        // * "ASP" is both an extension and a name so when we lookup
        //   using `find_syntax_by_token` we get the wrong one back
        // * "TeX" is both a language and an extension also.
        assets
            .get_syntax_set()
            .unwrap()
            .syntaxes()
            .iter()
            .map(|l| l.name.as_str())
            .filter(|l| *l != "ASP" && *l != "TeX")
    }
}
