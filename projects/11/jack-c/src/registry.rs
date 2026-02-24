
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

