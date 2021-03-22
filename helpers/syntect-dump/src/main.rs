use std::io::Write;
use std::{fs, io};

use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSetBuilder;

fn main() {
    println!();

    print!("loading syntaxes from directory... ");
    io::stdout().flush().expect("could not flush stdout");
    let syntaxes = {
        let mut builder = SyntaxSetBuilder::new();
        builder.add_plain_text_syntax();
        builder
            .add_from_folder("syntect/syntaxes", true)
            .expect("couldn't load syntaxes");
        builder.build()
    };
    println!("OK !");

    print!("loading syntaxes from directory... ");
    io::stdout().flush().expect("could not flush stdout");
    let themes = ThemeSet::load_from_folder("syntect/themes").expect("couldn't load themes");
    println!("OK !");

    let mut sorted_sytaxes: Vec<_> = syntaxes
        .syntaxes()
        .iter()
        .map(|it| (it.name.as_str(), it.file_extensions.as_slice()))
        .collect();
    sorted_sytaxes.sort_unstable_by_key(|it| it.0);

    let mut sorted_themes: Vec<_> = themes.themes.keys().collect();
    sorted_themes.sort_unstable();

    println!();
    println!("found syntaxes:");
    for (name, file_extensions) in sorted_sytaxes {
        println!("{:?} -> {:?}", name, file_extensions);
    }
    println!();

    println!("found themes:");
    for name in sorted_themes {
        println!("{:?}", name);
    }
    println!();

    fs::create_dir_all("syntect/dumps").expect("couldn't create dumps' folder");

    print!("generating syntaxes dump... ");
    io::stdout().flush().expect("could not flush stdout");
    dumps::dump_to_file(&syntaxes, "syntect/dumps/syntaxes.dump")
        .expect("couldn't generate syntaxes dump");
    println!("OK !");

    print!("generating themes dump... ");
    io::stdout().flush().expect("could not flush stdout");
    dumps::dump_to_file(&themes, "syntect/dumps/themes.dump")
        .expect("couldn't generate themes dump");
    println!("OK !");

    println!();
}
