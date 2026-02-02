pub fn comp(mnemonic: &str) -> &'static str {
    match mnemonic {
        // 0 / 1 / -1
        "0"   => "0101010", // 2a
        "1"   => "0111111", // 3f
        "-1"  => "0111010", // 3a

        // D register
        "D"   => "0001100", // 0c
        "!D"  => "0001101", // 0d
        "-D"  => "0001111", // 0f
        "D-1" => "0001110", // 0e
        "D+1" | "1+D" => "0011111", // 1f

        // A register
        "A"   => "0110000", // 30
        "!A"  => "0110001", // 31
        "-A"  => "0110011", // 33
        "A-1" => "0110010", // 33
        "A+1" | "1+A" => "0110111", // 37

        // M register
        "M"   => "1110000", // 70
        "!M"  => "1110001", // 71
        "-M"  => "1110011", // 73
        "M-1" => "1110010", // 72
        "M+1" | "1+M" => "1110111", // 77

        // Operations
        "D+A" | "A+D" => "0000010", // 02
        "D-A" => "0010011", // 13
        "A-D" => "0000111", // 07
        "D&A" | "A&D" => "0000000", // 00
        "D|A" | "A|D" => "0010101", // 15

        "D+M" | "M+D" => "1000010", // 42
        "D-M" => "1010011", // 53
        "M-D" => "1000111", // 47
        "D&M" | "M&D" => "1000000", // 40
        "D|M" | "M|D" => "1010101", // 55

        _ => panic!("Invalid comp instruction: {}", mnemonic),
    }
}

pub fn jump(mnemonic: Option<&str>) -> &'static str {
    match mnemonic {
        None => "000", // 0
        Some("JGT") => "001", // 1
        Some("JEQ") => "010", // 2
        Some("JGE") => "011", // 3
        Some("JLT") => "100", // 4
        Some("JNE") => "101", // 5
        Some("JLE") => "110", // 6
        Some("JMP") => "111", // 7
        Some(s) => panic!("Invalid jump instruction: {}", s),
    }
}

pub fn dest(line: Option<&str>) -> &str {
    let line = match line {
        Some(s) => s,
        None => return "000".to_string(),
    };

    // TODO: Mask design pattern
    let mut dest_bits = *b"000";

    if line.contains('A') {dest_bits[0] = b'1'; }
    if line.contains('D') {dest_bits[1] = b'1'; }
    if line.contains('M') {dest_bits[2] = b'1'; }

    String::from_utf8(dest_bis.to_vec()).unwrap();
}

