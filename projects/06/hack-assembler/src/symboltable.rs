use std::collections::HashMap;

pub struct SymbolTable {
    table: HashMap<String, i32>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut table = HashMap::new();

        for i in 0..=15 {
            table.insert(format!("R{}", i), i);
        }

        // Initialize other predefined symbols
        // We use .to_string() because HashMap keys need to own the data
        table.insert("SCREEN".to_string(), 16384);
        table.insert("KBD".to_string(), 24576);
        table.insert("SP".to_string(), 0);
        table.insert("LCL".to_string(), 1);
        table.insert("ARG".to_string(), 2);
        table.insert("THIS".to_string(), 3);
        table.insert("THAT".to_string(), 4);

        SymbolTable { table }
    }

    pub fn add_entry(&mut self, symbol: String, address: i32) {
        self.table.insert(symbol, address);
    }

    /// Accepts &str so we can check without creating a new String object.
    pub fn contains(&self, symbol: &str) -> bool {
        self.table.contains_key(symbol)
    }

    /// Panics if the symbol is not found
    pub fn get_address(&self, symbol: &str) -> i32 {
        // .get returns Option<&i32>, so we unwrap and dereference (*) it to get i32
        *self.table.get(symbol).expect("Symbol not found in table")
    }
}
