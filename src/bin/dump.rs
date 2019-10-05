use std::fs;

use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSetBuilder;

fn main() {
    let syntaxes = {
        let mut builder = SyntaxSetBuilder::new();
        builder.add_plain_text_syntax();
        builder
            .add_from_folder("syntect-syntaxes", true)
            .expect("couldn't load syntaxes");
        builder.build()
    };
    let themes = ThemeSet::load_from_folder("syntect-themes").expect("couldn't load themes");

    fs::create_dir_all("syntect-dumps").expect("couldn't create dumps' folder");

    dumps::dump_to_file(&syntaxes, "syntect-dumps/syntaxes.dump")
        .expect("couldn't emit syntaxes' dump");
    dumps::dump_to_file(&themes, "syntect-dumps/themes.dump").expect("couldn't emit themes' dump");
}
