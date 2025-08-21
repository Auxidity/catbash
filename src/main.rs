use clap::{ArgMatches, CommandFactory, Parser};
use std::error::Error;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

//ToDo: Handle "" parsing in strings (raw stringify).
//E.g. echo "something" should behave weirdly.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(
        short = 'i',
        long,
        help = r#"Input argument to write to a file that will be catbashed. Accepts a single word and "multiple words" formats (e.g. ls and "ls -l" are both valid). Call without other flags to create a tmp file that gets deleted immediately after (why don't you just write to terminal directly if you don't want to store the command?)"#
    )]
    input: Option<String>,

    #[arg(
        short = 'o',
        long,
        help = "Output file that will be catbashed. NOTE: THE OUTPUT FILE WILL BE OVERWRITTEN ON EVERY INVOCATION"
    )]
    output: Option<String>,

    #[arg(
        short = 'c',
        long,
        help = "Used to signify the need to capture output of catbash"
    )]
    capture: bool,

    #[arg(
        short = 't',
        long,
        help = "Target file where to store captured value. THIS FILE WILL BE OVERWRITTEN ON EVERY INVOCATION"
    )]
    target: Option<String>,

    #[arg(
        short = 'a',
        long,
        help = "Post capture modification arguments. E.g if catbash runs ls, argument could be '| grep something', and only matches with something would be stored to target."
    )]
    arguments: Option<String>,

    #[arg(
        short = 'f',
        long,
        help = "Same as arguments flag, with the exception the arguments are read from a file"
    )]
    arguments_from_file: Option<String>,

    // Positional arguments (non-flag arguments)
    #[arg(help = "Files to process with default behavior. Only 1 is expected for now")]
    file: Vec<String>,
}

#[derive(Debug)]
enum AppMode {
    NoArgs,
    DefaultBehavior(String),
    DefinedFlags,
    Error(String),
}

fn execute_arguments(
    captured_value: &str,
    arguments: &str,
    write_to_stdout: bool,
) -> Result<String, Box<dyn Error>> {
    let cleaned_capture = captured_value.trim_end();
    let cmd = format!(r#"echo "{cleaned_capture}" {arguments}"#);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to catbash");

    if !output.status.success() {
        eprintln!("DEBUG: Command failed: {cmd}");
        eprintln!("DEBUG: stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("DEBUG: stdout: {}", String::from_utf8_lossy(&output.stdout));
        return Err("Command exited with non-zero status".into());
    }

    if write_to_stdout {
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.to_string())
}

fn catbash(path: &str) -> Result<(), Box<dyn Error>> {
    let cmd = format!(r#"cat "{path}" | bash"#);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .status()
        .expect("Failed to catbash");

    if !output.success() {
        return Err("Command exited with non-zero status".into());
    }

    Ok(())
}

//There is no winning on this particular function. If you call capture catbash with ls, it works
//better with args flags (other shell commands) when we use the following. If we wanted to emulate
//what regular catbash does (in terms of formatting the output with \t instead of \n in the case of
//ls, which is handled by ls internally), the execute_arguments() becomes brittle immediately.
fn capture_catbash(path: &str, write_to_stdout: bool) -> Result<String, Box<dyn Error>> {
    let cmd = format!(r#"cat "{path}" | bash"#);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .expect("Failed to catbash");

    if !output.status.success() {
        return Err("Command exited with non-zero status".into());
    }

    if write_to_stdout {
        io::stdout().write_all(&output.stdout)?;
        io::stderr().write_all(&output.stderr)?;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout.to_string())
}

fn has_flags_from_matches(matches: &ArgMatches) -> bool {
    //Regardless of if files is recognized as a non-flag argument or not, which seems to vary on the day, this should work.
    matches.args_present() && !matches.contains_id("files")
}

fn validate_matches(matches: &ArgMatches, args_given: &Args) -> Result<(), Box<dyn Error>> {
    if args_given.capture && !matches.contains_id("output") {
        return Err("Capture flag requires an output file to catbash".into());
    };

    if matches.contains_id("target") && (!matches.contains_id("output") || !args_given.capture) {
        return Err(
            "Target flag requires an output file to catbash and capture flag to be present".into(),
        );
    };

    if matches.contains_id("arguments") && (!matches.contains_id("output") || !args_given.capture) {
        return Err(
            "Arguments flag requires an output file to catbash and capture flag to be present"
                .into(),
        );
    }

    if matches.contains_id("arguments_from_file")
        && (!matches.contains_id("output") || !args_given.capture)
    {
        return Err(
            "Arguments from file flag requires an output file to catbash and capture flag to be present"
                .into(),
        );
    }

    if matches.contains_id("arguments") && matches.contains_id("arguments_from_file") {
        return Err("Cannot provide arguments from -a and -f flags simultaneously".into());
    }

    Ok(())
}

fn determine_app_mode(args: &Args) -> Result<AppMode, Box<dyn Error>> {
    let cmd = Args::command();
    let matches = cmd.try_get_matches()?;
    validate_matches(&matches, args)?;

    let has_flags = has_flags_from_matches(&matches);
    let files: Vec<String> = matches
        .get_many::<String>("files")
        .unwrap_or_default()
        .cloned()
        .collect();

    //Debug
    // println!("{matches:?}");
    // println!("{has_flags:?}");

    Ok(match (has_flags, files.len()) {
        (false, 0) => AppMode::NoArgs,
        (false, 1) => AppMode::DefaultBehavior(args.file[0].clone()),
        (false, n) if n > 1 => AppMode::Error(format!(
            "Too many files for default behavior: {:?}. Please use flags or provide only one file.",
            args.file
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
    catbash(filename)?;
    Ok(())
}

//This function is a clusterfuck. I cant think of anything more clear than this though to achieve
//the same functionality.
fn handle_defined_flags(args: &Args) -> Result<(), Box<dyn Error>> {
    match (
        &args.input,
        &args.output,
        &args.capture,
        &args.target,
        &args.arguments,
        &args.arguments_from_file,
    ) {
        (Some(input), None, false, None, None, None) => {
            println!(
                "Avoid using -i flag alone, the application might not be able to create or remove the temporary file created to execute"
            );

            let temp_path = "temp.txt";
            write_to_file(input, temp_path)?;
            catbash(temp_path)?;
            delete_file(temp_path)?;
        }
        (None, Some(output), false, None, None, None) => {
            catbash(output)?;
        }
        (Some(input), Some(output), false, None, None, None) => {
            write_to_file(input, output)?;
            catbash(output)?;
        }
        (Some(input), Some(output), true, None, None, None) => {
            write_to_file(input, output)?;
            let _captured_value = capture_catbash(output, true)?;
        }
        (Some(input), Some(output), true, Some(target), None, None) => {
            write_to_file(input, output)?;
            let captured_value = capture_catbash(output, false)?;
            write_to_file(&captured_value, target)?;
        }
        (None, Some(output), true, None, Some(arguments), None) => {
            let captured_value = capture_catbash(output, false)?;
            let _modified_value = execute_arguments(&captured_value, arguments, true)?;
        }
        (None, Some(output), true, Some(target), Some(arguments), None) => {
            let captured_value = capture_catbash(output, false)?;
            let modified_value = execute_arguments(&captured_value, arguments, false)?;
            write_to_file(&modified_value, target)?;
        }
        (Some(input), Some(output), true, None, Some(arguments), None) => {
            write_to_file(input, output)?;
            let captured_value = capture_catbash(output, false)?;
            let _modified_value = execute_arguments(&captured_value, arguments, true)?;
        }
        (Some(input), Some(output), true, Some(target), Some(arguments), None) => {
            write_to_file(input, output)?;
            let captured_value = capture_catbash(output, false)?;
            let modified_value = execute_arguments(&captured_value, arguments, false)?;
            write_to_file(&modified_value, target)?;
        }
        (None, Some(output), true, None, None, Some(arguments)) => {
            let captured_value = capture_catbash(output, false)?;
            let actual_arguments = read_from_file(arguments)?;
            let _modified_value = execute_arguments(&captured_value, &actual_arguments, true)?;
        }
        (None, Some(output), true, Some(target), None, Some(arguments)) => {
            let captured_value = capture_catbash(output, false)?;
            let actual_arguments = read_from_file(arguments)?;
            let modified_value = execute_arguments(&captured_value, &actual_arguments, false)?;
            write_to_file(&modified_value, target)?;
        }
        (Some(input), Some(output), true, None, None, Some(arguments)) => {
            write_to_file(input, output)?;
            let captured_value = capture_catbash(output, false)?;
            let actual_arguments = read_from_file(arguments)?;
            let _modified_value = execute_arguments(&captured_value, &actual_arguments, true)?;
        }
        (Some(input), Some(output), true, Some(target), None, Some(arguments)) => {
            write_to_file(input, output)?;
            let captured_value = capture_catbash(output, false)?;
            let actual_arguments = read_from_file(arguments)?;
            let modified_value = execute_arguments(&captured_value, &actual_arguments, false)?;
            write_to_file(&modified_value, target)?;
        }
        _ => todo!(),
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

    let mut file = OpenOptions::new()
        .append(false)
        .write(true)
        .truncate(true)
        .open(dest)?;
    file.write_all(content.as_bytes())?;

    Ok(())
}

fn read_from_file(path: &str) -> Result<String, Box<dyn Error>> {
    if !Path::new(path).exists() {
        return Err("File not found in given path".into());
    }

    let content = fs::read(path)?;

    let output = String::from_utf8_lossy(&content);

    Ok(output.to_string())
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
