use clap::{crate_authors, crate_version, App, Arg};
use stackup_lint::{self, interface::Format};
use std::error::Error;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

type Result<T> = ::std::result::Result<T, Box<dyn Error>>;
const INPUT_EXTENSION_MESSAGE: &str =
    r#"Input file must be a graphql schema!".graphql" extension is missing"#;

fn main() {
    let app = app();
    let matches = app.get_matches();

    let format = matches
        .value_of("format")
        .map(Format::from)
        .unwrap_or_default();

    match matches.value_of("INPUT") {
        Some("-") => try_checking(try_read_stdin(), format),
        Some(path) => try_checking(try_read_contents(path), format),
        _ => (),
    }
}

fn try_checking(r: Result<String>, format: Format) {
    match r.map(|s| stackup_lint::check(&s)) {
        Ok(check_result) => match format {
            Format::TTY => println!("{}", check_result),
            Format::JSON => println!(
                "{}",
                check_result
                    .to_json()
                    .expect("failed to serialize comments")
            ),
        },
        Err(e) => eprintln!("{}", e),
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

fn try_read_stdin() -> Result<String> {
    let mut contents = String::new();

    io::stdin().read_to_string(&mut contents)?;
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
        .arg(
            Arg::with_name("format")
                .takes_value(true)
                .short("f")
                .long("format")
                .help("choose the specified format")
                .default_value("tty")
                .possible_values(&["tty", "json"]),
        )
}
