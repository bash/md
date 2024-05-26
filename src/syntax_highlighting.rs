use anstyle::{Reset, Style};
use bat::assets::HighlightingAssets;
use bat::config::Config as BatConfig;
use bat::controller::Controller as BatController;
use bat::input::Input;
use bat::WrappingMode;
use md_language_names::lookup_bat_language;
use std::borrow::Cow;

// TODO: can we detect if bat supports a given language
// so we can gracefully fall back to plain?
pub(crate) fn highlight(code: &str, options: &Options) -> String {
    try_highlight(code, options)
        .unwrap_or_else(|_error| format!("{}{code}{Reset}", Style::new().italic()))
}

fn try_highlight(code: &str, options: &Options) -> bat::error::Result<String> {
    let assets = HighlightingAssets::from_binary(); // TODO: re-use
    let config = bat_config(options, &assets);
    let controller = BatController::new(&config, &assets);
    let inputs = vec![Input::from_reader(Box::new(code.as_bytes()))];
    let mut output = String::new();
    controller
        .run_with_error_handler(inputs, Some(&mut output), |e, _w| _ = dbg!(e))
        .inspect_err(|e| _ = dbg!(e))
        .map(|_| output)
}

#[derive(Debug)]
pub(crate) struct Options<'a> {
    pub(crate) available_columns: usize,
    pub(crate) language: Option<Cow<'a, str>>,
}

// TODO: use theme appropriate for dark/light mode
// TODO: make theme configurable
fn bat_config<'a>(options: &'a Options, assets: &'a HighlightingAssets) -> BatConfig<'a> {
    let language = options
        .language
        .as_ref()
        .and_then(|l| lookup_bat_language(l, assets));
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
