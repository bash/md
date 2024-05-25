use anstyle::{Reset, Style};
use bat::assets::HighlightingAssets;
use bat::config::Config as BatConfig;
use bat::controller::Controller as BatController;
use bat::input::Input;
use bat::WrappingMode;
use std::borrow::Cow;

pub(crate) fn highlight(code: &str, options: &Options) -> String {
    try_highlight(code, options)
        .unwrap_or_else(|_error| format!("{}{code}{Reset}", Style::new().italic()))
}

fn try_highlight(code: &str, options: &Options) -> bat::error::Result<String> {
    let assets = HighlightingAssets::from_binary(); // TODO: re-use
    let config = bat_config(options);
    let controller = BatController::new(&config, &assets);
    let inputs = vec![Input::from_reader(Box::new(code.as_bytes()))];
    let mut output = String::new();
    controller.run(inputs, Some(&mut output)).map(|_| output)
}

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub(crate) available_columns: usize,
    pub(crate) language: Option<Cow<'a, str>>,
}

fn bat_config<'a>(options: &'a Options) -> BatConfig<'a> {
    let language = options.language.as_ref().map(|l| parse_language(&l));
    BatConfig {
        language,
        term_width: options.available_columns,
        colored_output: true,
        true_color: true,
        wrapping_mode: WrappingMode::Character,
        // theme: "ansi".to_owned(),
        ..Default::default()
    }
}

fn parse_language(language: &str) -> &str {
    let language = remove_modifier(language);
    language
}

fn remove_modifier(language: &str) -> &str {
    // Languages can have a modifier separated by a comma
    // e.g. `rust,no_run`
    let comma = language.find(',');
    match comma {
        Some(comma) => &language[..comma],
        None => language,
    }
}
