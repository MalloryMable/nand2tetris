use clap::{Parser, ValueHint};
use std::path::Path;
use walkdir::WalkDir;
use std::path::PathBuf;
use std::process;
use std::fs;
use std::collections::HashSet;

mod parser;
mod codewriter;

use parser::{Parser as VMParser};
use codewriter::CodeWriter;

#[derive(Parser)]
#[command(version, name = "Hack VM Translator")]
#[command(about = "Translates down .vm files into one ROM file (.hack) for the Hack machine")]
#[command(long_about = "Recursively scans directories for .vm files or accepts a single .vm file.\
                        Outputs a single .asm file.")]
struct Cli {
    #[arg(value_hint= ValueHint::FilePath, required = true, num_args = 1..)]
    input: Vec<PathBuf>,

    #[arg(short = 'o', long = "out")]
    #[arg(value_hint = ValueHint::FilePath)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let source_files = match input_file_filter(&cli.input) {
        Ok(files) => files,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    if source_files.is_empty() {
        eprintln!("Error: No .vm files found in the provided input.");
        process::exit(1);
    }

    // I could check for the first directory but the reward is so marginal
    let first_input = &cli.input[0];
    let output_path = resolve_output_path(first_input, cli.output);
    match process_files(&source_files, &output_path) {
        Ok(_) => {
            println!("Successfully compiled to {}.asm", output_path.display());
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

// Filters input for vm files
fn input_file_filter(input: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let mut seen_paths = HashSet::new(); // Track unique files

    input.iter()
        .map(|path| {
            if path.is_file() {
                if path.extension().and_then(|s| s.to_str()) == Some("vm") {
                    Ok(vec![path.clone()])
                } else {
                    Err(format!("Error: '{}' is not a .vm file", path.display()))
                }
            } else if path.is_dir() {
                let dir_files = WalkDir::new(path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_type().is_file() &&
                        e.path().extension().and_then(|s| s.to_str()) == Some("vm")
                    })
                    .map(|e| e.path().to_owned())
                    .collect();
                Ok(dir_files)
            } else {
                Err(format!("Error: Input '{}' does not exist", path.display()))
            }
        })
        .collect::<Result<Vec<Vec<PathBuf>>, String>>()
        .map(|nested| {
            nested.into_iter()
                .flatten()
                .filter(|path| {
                    // Canonicalize checks for existence and resolves "Main.vm" vs "./Main.vm"
                    match path.canonicalize() {
                        Ok(canon) => seen_paths.insert(canon),
                        Err(_) => false, // Skip invalid paths
                    }
                })
                .collect()
        })
}fn resolve_output_path(first_input: &Path, output_flag: Option<PathBuf>) -> PathBuf {
    match output_flag {
        Some(path) => {
            if path.is_dir() {
                let default_name = default_filename(first_input);
                path.join(default_name)
            } else {
                path
            }
        },
        None => {
            let filename = default_filename(first_input);
            Path::new(".").join(filename)
        }
    }
}

fn default_filename(input: &Path) -> PathBuf {
    if input.is_dir() {
        let stem = input.file_name().unwrap_or(input.as_os_str());
        Path::new(stem).to_path_buf()
    } else {
        PathBuf::from("a")
    }
}

fn process_files(source_files: &[PathBuf], output_file: &Path) -> Result<(), String> {
    // Create a .part file
    let temp_path = output_file.with_extension("part");
    let mut writer = CodeWriter::new(&temp_path).map_err(|e| e.to_string())?;
    writer.init().map_err(|e| format!("Init error: {}", e))?;

    for path in source_files {
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format!("Invalid filename: {}", path.display()))?;

        writer.set_file_name(file_stem);

        // NOTE: parser must be mutable to iterate
        let mut parser = VMParser::new(path).map_err(|e| format!("Error opening {}: {}", path.display(), e))?;

        // 2. The Clean Loop (No Result unpacking here)
        for cmd in &mut parser {
            writer.command(cmd).map_err(|e| format!("Write error: {}", e))?;
        }

        if let Some(err) = parser.get_error() {
             return Err(format!("Error in file {}: {}", path.display(), err));
        }
    }

    drop(writer); // Flush and close file handle
    let final_path = output_file.with_extension("asm");
    fs::rename(&temp_path, final_path).map_err(|e| e.to_string())?;

    Ok(())
}
