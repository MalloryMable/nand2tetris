use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process;

mod parser;
mod symboltable;
mod code;

use parser::{Command, Parser};
use symboltable::SymbolTable;

fn main() {
    let (input_path, output_path) = match parse_config() {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        }
    };

    let parser = match Parser::new(&input_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", input_path.display(), e);
            process::exit(1);
        }
    };

    let mut symboltable = SymbolTable::new();
    let mut rom_addr = 0;

    let mut instructions = Vec::new();

    for cmd in parser{
        match cmd {
            Command::L(label) => {
                // Adds the label as a "side effect" of flitering it
                symboltable.add_entry(&label, rom_addr);
            },
            executable_cmd => {
                // Also a "side effect" tracks the line in the ROM final byte code
                rom_addr+= 1;
                instructions.push(executable_cmd);
            }
        }
    }
    // Notice here we finally "set off" the chain of iterations we just built we have to do this
    // before the next iteration

    let mut writer = File::create(&output_path).unwrap_or_else(|e| {
        eprintln!("Error create output file: {}", e);
        process::exit(1);
    });

    //after the first 16 reserved addresses
    let mut ram_address = 16;

    for  (i, command) in instructions.iter().enumerate() {

        // We add the new line every loop but the first one because
        // We concatinate hack files and assume there are no blank lines
        if i > 0 {
            writeln!(writer, "").unwrap();
        }

        match command {
            Command::C(dest, comp, jump) => {
                let comp_bin = code::comp(&comp);
                let dest_bin = code::dest(dest.as_deref());
                let jump_bin = code::jump(jump.as_deref());

                write!(writer, "111{}{}{}", comp_bin, dest_bin, jump_bin).unwrap();
            },
            Command::A(symbol) => {
                let address_value = if let Ok(val) = symbol.parse::<i32>() {
                    val
                } else {
                    if symboltable.contains(&symbol) {
                        symboltable.get_address(&symbol)
                    } else {
                        symboltable.add_entry(symbol, ram_address);
                        let val = ram_address;
                        ram_address += 1;
                        val
                    }
                };
                // write!(writer, "{:016b}", address_value).unwrap();
                write!(writer, "{:016b}", address_value).unwrap();
            },
            _ => unreachable!("L-Commands were filtered out in the first pass"),
        }
    }

    print!("Succesfully complied to: {}", output_path.display());
}

// This takes in a simple CLI argument
fn parse_config() -> Result<(PathBuf, PathBuf), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return Err("Error: No arguments passed. Usage: <input.asm> [dest]".to_string());
    }

    let input_path = Path::new(&args[1]);

    if input_path.extension().and_then(|s| s.to_str()) != Some("asm") {
        return Err(format!("Error: Input file '{}' must have an .asm extension.", input_path.display()))
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

