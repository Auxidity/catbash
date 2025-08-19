use clap::{ArgMatches, CommandFactory, Parser};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'i',
        long,
        help = r#"Input argument to write to a file that will be catbashed. Accepts a single word and "multiple words" formats (e.g. ls and "ls -l" are both valid). Call without other flags to create a tmp file that gets deleted immediately after (why don't you just write to terminal directly if you don't want to store the command?)"#
    )]
    input: Option<String>,

    #[arg(short = 'o', long, help = "Output file that will be catbashed")]
    output: Option<String>,

    // Positional arguments (non-flag arguments)
    #[arg(help = "Files to process with default behavior. Only 1 is expected for now")]
    files: Vec<String>,
}

#[derive(Debug)]
enum AppMode {
    NoArgs,
    DefaultBehavior(String),
    DefinedFlags,
    Error(String),
}

fn catbash(path: &str) {
    let cmd = format!(r#"cat "{path}" | bash"#);

    let status = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
        .expect("Failed to catbash");

    if !status.success() {
        eprintln!("Command exited with non-zero status");
    }
}

fn has_flags_from_matches(matches: &ArgMatches) -> bool {
    matches.args_present()
}

fn determine_app_mode(args: &Args) -> Result<AppMode, Box<dyn Error>> {
    let cmd = Args::command();
    let matches = cmd.try_get_matches()?;

    let has_flags = has_flags_from_matches(&matches);
    let files: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .cloned()
        .collect();

    Ok(match (has_flags, files.len()) {
        (false, 0) => AppMode::NoArgs,
        (false, 1) => AppMode::DefaultBehavior(args.files[0].clone()),
        (false, n) if n > 1 => AppMode::Error(format!(
            "Too many files for default behavior: {:?}. Please use flags or provide only one file.",
            args.files
        )),
        (true, 0) => AppMode::DefinedFlags,
        (true, _) => AppMode::Error(
            "Cannot mix positional files with flags. Use either flags OR a single file."
                .to_string(),
        ),
        (_, _) => AppMode::Error("Undefined behavior".to_string()),
    })
}

fn handle_no_args() -> Result<(), Box<dyn Error>> {
    println!("No arguments given, nothing to do.");
    std::process::exit(0);
}

fn handle_default_behavior(filename: &str) -> Result<(), Box<dyn Error>> {
    catbash(filename);
    Ok(())
}

fn handle_defined_flags(args: &Args) -> Result<(), Box<dyn Error>> {
    if let (Some(input), None) = (&args.input, &args.output) {
        println!(
            "Avoid using -i flag alone, the application might not be able to remove the temporary file created to execute"
        );

        let temp_path = "temp.txt";
        write_to_file(input, temp_path)?;
        catbash(temp_path);
        delete_file(temp_path)?;
    }

    if let (None, Some(output)) = (&args.input, &args.output) {
        catbash(output);
    }

    if let (Some(input), Some(output)) = (&args.input, &args.output) {
        write_to_file(input, output)?;
        catbash(output);
    }

    Ok(())
}

fn handle_error(error_msg: &str) -> Result<(), Box<dyn Error>> {
    eprintln!("Error: {error_msg}");
    std::process::exit(1);
}

fn write_to_file(content: &str, dest: &str) -> io::Result<()> {
    if !Path::new(dest).exists() {
        fs::write(dest, "")?; // create the file if it doesn't exist
    }

    let mut file = OpenOptions::new().append(false).write(true).open(dest)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn delete_file(path: &str) -> io::Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mode = determine_app_mode(&args)?;

    match mode {
        AppMode::NoArgs => handle_no_args(),
        AppMode::DefaultBehavior(filename) => handle_default_behavior(&filename),
        AppMode::DefinedFlags => handle_defined_flags(&args),
        AppMode::Error(msg) => handle_error(&msg),
    }
}
