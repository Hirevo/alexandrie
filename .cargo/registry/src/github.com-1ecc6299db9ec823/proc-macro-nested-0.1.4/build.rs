use std::env;
use std::fs::File;
use std::io::Write;
use std::iter;
use std::path::Path;

/*
#[doc(hidden)]
#[macro_export]
macro_rules! count {
    () => { proc_macro_call_0!() };
    (!) => { proc_macro_call_1!() };
    (!!) => { proc_macro_call_2!() };
    ...
}
*/

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("count.rs");
    let mut f = File::create(&dest_path).unwrap();

    let mut content = String::new();
    content += "
        #[doc(hidden)]
        #[macro_export]
        macro_rules! count {
    ";
    for i in 0..=64 {
        let bangs = iter::repeat("!").take(i).collect::<String>();
        content += &format!("({}) => {{ proc_macro_call_{}!() }};", bangs, i);
    }
    content += "
            ($(!)+) => {
                compile_error!(\"this macro does not support >64 nested macro invocations\")
            };
        }
    ";

    f.write_all(content.as_bytes()).unwrap();
}
