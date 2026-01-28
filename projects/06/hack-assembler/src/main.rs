use std::{env, os::unix::process, process::Output};

mod code;

fn main() {
    let (input_path, output_path) = match parse_config() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    // TODO 3 Pass Structure
}

// TODO: Break out into separte module?
fn parse_config() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Error: No arguments passed. Usage: <input.asm> [dest]".to_string())
    }

    let input_path = Path::new(&args[1]);

    if ( input_path.extension().and_then(|s| s.to_str()) != Some("asm")) {
        Err(format!("Error: Input file '{}' must have an .asm extension.", input_path.display()))
    }

    let output_path = if args.len() > 2 {
        let dest_arg = Path::new(&args[2]);

        if dest_arg.is_dir() {
            let stem = input_path.file_stem().ok_or("input file has no name(?)")?;
            dest_arg.join(stem).with_extension("hack")
        } else if dest_arg.extension().and_then(|s| s.to_str()) == Some("hack") {
            dest_arg.to_path_buf()
        } else {
            return Err(format!("Error: Destination '{}' is invalid. Usage: <input.asm> [dest]", dest_arg.display()))
        }
    } else {
        let stem = input_path.file_stem().ok_or("Input file has no name")?;
        Path::new(".").join(stem).with_extension("hack")
    };

    Ok((input_path.to_path_buf(), output_path))
}
