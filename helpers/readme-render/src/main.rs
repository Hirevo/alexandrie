use std::fs;
use std::env;

use alexandrie_rendering::config::{SyntectConfig, SyntectState};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Config {
    /// The syntax-highlighting configuration.
    pub syntect: SyntectConfig,
}

fn main() {
    let readme_path = env::args().skip(1).next().expect("could not find a command-line argument");

    let contents = fs::read("alexandrie.toml").expect("could not open configuration file `alexandrie.toml`");
    let config: Config = toml::from_slice(contents.as_slice()).expect("could not parse configuration file");

    let state = SyntectState::from(config.syntect);

    let contents = fs::read_to_string(readme_path).expect("could not open README file");
    let rendered = alexandrie_rendering::render_readme(&state, &contents);

    fs::write("output.html", &rendered).expect("could not write rendered HTML output");
}
