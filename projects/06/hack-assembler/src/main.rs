use clap::{Parser, ValueHint};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process;

mod parser;
mod symboltable;
mod code;

use parser::{Command, Parser as AsmParser};
use symboltable::SymbolTable;

#[derive(Parser)]
#[command(version, name = "Hack Assembler")]
#[command(about = "Compiles hack assembly(.asm) to binary (.hack)")]
struct Cli {
    #[arg(value_hint= ValueHint::FilePath)]
    #[arg(value_parser = validate_asm)] // Custom hook
    input: PathBuf,

    #[arg(value_hint= ValueHint::FilePath)]
    output: Option<PathBuf>,
}

fn validate_asm(s: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(s);
    if path.extension().and_then(|ext| ext.to_str()) == Some("asm") {
        Ok(path)
    } else {
        Err(format!("Input file '{}' must have an .asm extension.", path.display()))
    }
}


fn main() {
    let cli = Cli::parse();
    let output_path = match cli.output {
        Some(path) => {
            if path.is_dir() {
                let stem = cli.input.file_stem().expect("Input file must have a filename");
                path.join(stem).with_extension("hack")
            } else {
                if path.extension().and_then(|s| s.to_str()) != Some("hack") {
                     eprintln!("Error: Output file must have .hack extension");
                     process::exit(1);
                }
                path
            }
        },
        None => {
            let stem = cli.input.file_stem().expect("Input file must have a filename");
            cli.input.with_file_name(stem).with_extension("hack")
        }
    };

    let parser = match AsmParser::new(&cli.input) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error opening file '{}': {}", cli.input.display(), e);
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

