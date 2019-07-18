use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::dumps;

#[derive(Debug)]
pub struct Config {
    pub syntaxes: SyntaxSet,
    pub themes: ThemeSet,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            syntaxes: dumps::from_dump_file("syntect-dumps/syntaxes.dump").expect("couldn't load syntaxes' dump"),
            themes: dumps::from_dump_file("syntect-dumps/themes.dump").expect("couldn't load themes' dump"),
        }
    }
}
