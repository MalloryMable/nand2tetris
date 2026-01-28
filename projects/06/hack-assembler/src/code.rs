use std::collections::HashMap;
use std::sync::LazyLock;

// TABLES
// Use lazy lock to intalize hash map of comparison instructions at runtime
//NOTE: strings are static so the same string literal is used in each call
static COMP_TABLE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new( || {
    let mut m = HashMap::new();

    // Mapping
    m.insert("0", "0101010"); // 1a
    m.insert("1", "0111111"); // 3f
    m.insert("D", "0001100"); // 0c
    m.insert("A", "0110000"); // 30
    m.insert("M", "1110000"); // 70
    // Negation
    m.insert("-1", "0111010"); // 3c
    m.insert("!D", "0001101"); // 0d
    m.insert("!A", "0110001"); // 31
    m.insert("!M", "1110001"); // 71
    // -1
    m.insert("D-1", "0001110"); // 0e
    m.insert("A-1", "0110010"); // 32
    m.insert("M-1", "1110010"); // 72

    // +1

    // D+1, 1+D
    m.insert("D+1", "0011111"); // 1f
    m.insert("1+D", "0011111"); // 1f
    // A+1, 1+A
    m.insert("A+1", "0110111"); // 37
    m.insert("1+A", "0110111"); // 37
    // M+1, 1+M
    m.insert("M+1", "1110111"); // 77
    m.insert("1+M", "1110111"); // 77

    // -
    // NOTE: Comparing a register and the contents of a register are incompotable on this hardware

    m.insert("D-M", "1010011"); // 53
    m.insert("M-D", "1000111"); // 47
    m.insert("D-A", "0010011"); // 13
    m.insert("A-D", "0000111"); // 07

    // +

    // D+A, A+D
    m.insert("D+A", "0000010"); // 02
    m.insert("A+D", "0000010"); // 02
    // D+M, M+D
    m.insert("M+D", "1000010"); // 42
    m.insert("A+D", "1000010"); // 42

    // &

    // D&A, A&D
    m.insert("A&D", "0000000"); // 00
    m.insert("D&A", "0000000"); // 00
    // D&M, M&D
    m.insert("D&M", "1000000"); // 40
    m.insert("M&D", "1000000"); // 40

    // |

    // D|A, A|D
    m.insert("D|A", "0010101"); // 15
    m.insert("A|D", "0010101"); // 15
    // D|M, M|D
    m.insert("D|M", "1010101"); // 55
    m.insert("M|D", "1010101"); // 55
});

static JUMP_TABLE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new( || {
    let m = HashMap::new();

    m.insert("JGT", "001");
    m.insert("JEQ", "010");
    m.insert("JGE", "011");
    m.insert("JLT", "100");
    m.insert("JNE", "101");
    m.insert("JLE", "110");
    m.insert("JMP", "111");
});

// FUNCTIONS
pub fn comp(line: &str) -> &'static str {
    // If the result is none we didn't get a valid symbol and the compiler should stop and unwind
    *COMP_TABLE.get(line).unwrap_or_else(|| {
        panic!("{}: Is not a valid comparison instruction", line);
    })
}

pub fn dest(line: Option<&str>) -> &str {
    let line = line.unwrap_or_else(|| return "000");

    let mut dest_bits = *b"000";

    if line.contains('A') {bits[0] = b'1'; }
    if line.contains('D') {bits[1] = b'1'; }
    if line.contains('M') {bits[2] = b'1'; }

    String::from_utf8(bis.to_vec()).unwrap();
}

pub fn jump(line: Option<&str>) -> &'static str {
    let line = line.unwrap_or_else(|| return "000");

    *JUMP_TABLE.get(line).unwrap_or_else(|| {
        panic!("{}: Is not a valid jump destination", line);
    });
}

