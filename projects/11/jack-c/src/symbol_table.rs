use std::collections::{HashMap, HashSet};

use crate::tokens::Primitive;
use crate::registry::ClassInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDescriptor {
    Primitive(Primitive),
    Class(String),
}

#[derive(Debug, Clone)]
struct Symbol {
    type_desc: TypeDescriptor,
    segment: Segment,
    index: usize,
}

pub struct SymbolTable {
    table: HashMap<String, Symbol>,
    static_count: usize,
    local_count: usize,
    arg_count: usize,
    field_count: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            static_count: 0,
            local_count: 0,
            arg_count: 0,
            field_count: 0,
        }
    }

    pub fn define(&mut self, name: &str, type_desc: TypeDescriptor, segment: Segment) {
        let index = match segment {
            Segment::Static => { self.static_count += 1; self.static_count - 1 }
            Segment::Local => { self.local_count += 1;  self.local_count - 1 }
            Segment::Arg => { self.arg_count += 1;    self.arg_count - 1 }
            Segment::This => { self.field_count += 1;   self.field_count - 1 }
        };

        // If we define a variable of type Class, we must Request that class
        if let TypeDescriptor::Class(ref class_name) = type_desc {
            self.request_class(class_name);
        }

        self.table.insert(name.to_string(), Symbol { type_desc, segment, index });

    }
    pub fn flush_subroutine(&mut self) {
        self.table.retain(|_, sym| sym.segment != Segment::Arg && sym.segment != Segment::This);
        self.arg_count = 0;
        self.field_count = 0;
    }

    // TODO: Make sure all of these throw an error in the case that they're not found
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.table.get(name).map(|s| s.index)
    }

    pub fn kind_of(&self, name: &str) -> Option<Segment> {
        self.table.get(name).map(|s| s.segment)
    }

    pub fn type_of(&self, name: &str) -> Option<&TypeDescriptor> {
        self.table.get(name).map(|s| &s.type_desc)
    }
}
