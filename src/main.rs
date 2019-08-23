use clap::{crate_authors, crate_version, App, Arg};
use stackup_lint;
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::Read;
use std::path::Path;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;
const INPUT_EXTENSION_MESSAGE: &str =
    r#"Input file must be a graphql schema!".graphql" extension is missing"#;

fn main() {
    let app = app();
    let matches = app.get_matches();

    if let Some(path) = matches.value_of("INPUT") {
        if let Err(e) = try_read_contents(path).and_then(|s| stackup_lint::check(&s)) {
            eprintln!("{}", e);
        }
    }
}

fn try_read_contents<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = path.as_ref();
    match path.extension().and_then(OsStr::to_str) {
        Some(ext) if ext != "graphql" => return Err(INPUT_EXTENSION_MESSAGE.into()),
        None => return Err(INPUT_EXTENSION_MESSAGE.into()),
        _ => (),
    }
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    Ok(contents)
}

fn app() -> App<'static, 'static> {
    App::new("stackup-lint")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Checks a stackup schema to catch common mistakes")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
}
