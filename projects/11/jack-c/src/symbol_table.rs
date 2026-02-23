use std::collections::{HashMap, HashSet};

// --- Mocking Context ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Primitive { Int, Char, Boolean, Void }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeDescriptor {
    Primitive(Primitive),
    Class(String),
}

//TODO: Change keyword Type to primative
// - split up registry and symbol tables
// - Remove Class level dump and leave symbol table declaration in new()
// - Fix the negative/subtraction bug + update range catching


/// Tracks the state of a specific method
#[derive(Debug, Clone)]
struct MethodInfo {
    /// generic Option is used because a 'request' (call) might not know the arg count yet.
    arg_count: usize,
    is_defined: bool,
}

impl MethodInfo {
    /// Created when we define a method (compile its declaration)
    fn defined(args: usize) -> Self {
        Self {
            arg_count: args,
            is_defined: true,
        }
    }

    /// Created when we see a call to a method, but haven't compiled it yet
    fn requested(args: usize) -> Self {
        Self {
            arg_count: args, // We don't know the authoritative arg count yet
            is_defined: false,
        }
    }
}

/// Tracks the state of a class and its member methods
#[derive(Debug, Clone)]
struct ClassInfo {
    is_defined: bool,
    methods: HashMap<String, MethodInfo>,
}

impl ClassInfo {
    fn new(is_defined: bool) -> Self {
        Self {
            is_defined,
            methods: HashMap::new(),
        }
    }
}

// --- Main Symbol Table ---

#[derive(Debug, Clone)]
struct Symbol {
    type_desc: TypeDescriptor,
    segment: Segment,
    index: usize,
}

pub struct SymbolTable {
    table: HashMap<String, Symbol>,
    class_registry: HashMap<String, ClassInfo>,
    static_count: usize,
    local_count: usize,
    arg_count: usize,
    this_count: usize,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
            class_registry: HashMap::new(),
            static_count: 0,
            local_count: 0,
            arg_count: 0,
            this_count: 0,
        }
    }

    // --- Standard Scope Management (Unchanged) ---

    pub fn define(&mut self, name: &str, type_desc: TypeDescriptor, segment: Segment) {
        let index = match segment {
            Segment::Static => { self.static_count += 1; self.static_count - 1 }
            Segment::Local => { self.local_count += 1;  self.local_count - 1 }
            Segment::Arg => { self.arg_count += 1;    self.arg_count - 1 }
            Segment::This => { self.this_count += 1;   self.this_count - 1 }
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
        self.this_count = 0;
    }

    pub fn flush_class(&mut self) {
        self.table.retain(|_, sym| sym.segment != Segment::Static && sym.segment != Segment::Local);
        self.static_count = 0;
        self.local_count = 0;
    }

    // TODO: Change to Result that throws an outof bounds exception
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.table.get(name).map(|s| s.index)
    }

    // TODO: Throw undeclared variable error
    pub fn kind_of(&self, name: &str) -> Option<Segment> {
        self.table.get(name).map(|s| s.segment)
    }

    // TODO: Have this create a class if name doesn't get a match
    pub fn type_of(&self, name: &str) -> Option<&TypeDescriptor> {
        self.table.get(name).map(|s| &s.type_desc)
    }

    /// Call when compiling `class MyClass { ... }`
    pub fn declare_class(&mut self, class_name: &str) {
        self.update_class_status(class_name, true);
    }

    /// Call when seeing `MyClass varName` or `MyClass.method()`
    pub fn request_class(&mut self, class_name: &str) {
        self.update_class_status(class_name, false);
    }

    /// Helper to ensure ClassInfo exists
    fn update_class_status(&mut self, class_name: &str, is_definition: bool) {
        self.class_registry
            .entry(class_name.to_string())
            .and_modify(|info| {
                if is_definition { info.is_defined = true; }
            })
            .or_insert_with(|| ClassInfo::new(is_definition));
    }

    /// Call when compiling `function void myMethod(args...)`
    /// This Defines the method.
    pub fn define_method(&mut self, class_name: &str, method_name: &str, arg_count: usize) {
        let class_info = self.class_registry.get_mut(class_name).unwrap();

        // Overwrite any previous "request" placeholder with the authoritative definition
        // TODO: ensure method is counted as declared in this case
        class_info.methods.insert(method_name.to_string(), MethodInfo::defined(arg_count));
    }

    /// Call when seeing `obj.myMethod()` or `Class.myMethod()`
    /// This Requests the method.
    pub fn request_method(&mut self, class_name: &str, method_name: &str) {
        // Requesting a method implies requesting the class
        self.request_class(class_name);

        let class_info = self.class_registry.get_mut(class_name).unwrap();

        // Only insert a "Requested" placeholder if it doesn't exist yet.
        // If it was already Defined or Requested, leave it alone.
        class_info.methods
            .entry(method_name.to_string())
            .or_insert_with(MethodInfo::requested);
        //TODO: remove requested and keep it to true/false since we can get the rest from context
        //NOTE: This structure implies a requested declaration can can give the wrong informatio
        //nand not get caught since we assume they're telling the truth and then only correct
        //when we se see the truth. Find a way of tracking Liar Requests
    }

    pub fn get_method_arg_count(&self, class_name: &str, method_name: &str) -> Option<usize> {
        self.class_registry.get(class_name)
            .and_then(|c| c.methods.get(method_name))
            .and_then(|m| m.arg_count)
    }


}
