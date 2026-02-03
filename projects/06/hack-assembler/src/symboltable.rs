use std::collections::HashMap;

// Defined as module constants so they can be shared.
// We store the static string references here.
const R_REGS: [&str; 16] = [
    "R0", "R1", "R2", "R3", "R4", "R5", "R6", "R7",
    "R8", "R9", "R10", "R11", "R12", "R13", "R14", "R15",
];

const PREDEFINED: [(&str, i32); 7] = [
    ("SCREEN", 16384),
    ("KBD", 24576),
    ("SP", 0),
    ("LCL", 1),
    ("ARG", 2),
    ("THIS", 3),
    ("THAT", 4),
];

pub struct SymbolTable {
    table: HashMap<String, i32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = HashMap::new();

        for (i, &name) in R_REGS.iter().enumerate() {
            table.insert(name.to_string(), i as i32);
        }

        for &(name, address) in &PREDEFINED {
            table.insert(name.to_string(), address);
        }

        SymbolTable { table }
    }

    pub fn add_entry(&mut self, symbol: &String, address: i32) {
        self.table.insert(symbol.to_string(), address);
    }

    pub fn contains(&self, symbol: &str) -> bool {
        let symbol = self.resolve_predefined(symbol).unwrap_or(symbol);
        self.table.contains_key(symbol)
    }

    pub fn get_address(&self, symbol: &str) -> i32 {
        // Returns the static ref if found, otherwise borrows the input 'symbol'
        let symbol = self.resolve_predefined(symbol).unwrap_or(symbol);
        *self.table.get(symbol).expect("Symbol not found in table")
    }

    // Checks incoming symbols so that we can me only predefined symbols case insensitive
    fn resolve_predefined(&self, symbol: &str) -> Option<&'static str> {
        for &(name, _) in &PREDEFINED {
            if symbol.eq_ignore_ascii_case(name) {
                return Some(name);
            }
        }

        if symbol.len() > 1 && symbol.len() <= 3 {
             let bytes = symbol.as_bytes();
             if bytes[0].to_ascii_uppercase() == b'R' {
                 // Try to parse the number part
                 if let Ok(idx) = symbol[1..].parse::<usize>() {
                     // Safe array access using the constant
                     if idx < R_REGS.len() {
                         return Some(R_REGS[idx]);
                     }
                 }
             }
        }

        None
    }
}
