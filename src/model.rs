// --------------------
// types
// --------------------

use {
    crate::order::Order as ShellOrder,
    proc_macro2::{TokenStream as TokenStream2},
    syn::{Ident, Path, Type}
};

#[derive(Copy, Clone)]
pub enum ExpandMode {
    BuilderOnly,
    ShellOnly,
    Both,
}

#[derive(Default, Debug, Clone)]
pub struct BuilderCfg {
    pub required: bool, // `required` OR `not_in_default`
    pub into: bool,     // for required args / setters: take `impl Into<T>`
    pub skip_setter: bool,

    pub init_only: bool,

    // Explicit “kind” overrides (otherwise inferred)
    pub kind: Option<BuilderKind>,

    // Override init expression in new()
    pub default_expr: Option<TokenStream2>,
}

#[derive(Debug, Clone, Copy)]
pub enum BuilderKind {
    Flag,     // bool -> fn field(mut self)->Self { self.field=true; self }
    Opt,      // Option<T> -> fn field(mut self, v:T)->Self { self.field=Some(v); self }
    OptInto,  // Option<T> -> fn field(mut self, v: impl Into<T>)->Self { self.field=Some(v.into()); self }
    Set,      // T -> fn field(mut self, v:T)->Self { self.field=v; self }
    SetInto,  // T -> fn field(mut self, v: impl Into<T>)->Self { self.field=v.into(); self }
    VecInto,  // Vec<T> from impl Into<Vec<T>>
    VecIter,  // Vec<T> from IntoIterator<Item=T>
    Push,     // Vec<T> push single elem (needs explicit method name later; kept for future)
}

#[derive(Default, Debug, Clone)]
pub struct ShellCfg {
    pub cmd: Option<String>,
    pub trait_path: Option<Path>,

    pub require_order: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum PosMode {
    Clone,
    Display,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub ident: Ident,
    pub ty: Type,
    pub builder: BuilderCfg,
    pub shell: ShellFieldCfg,
    #[allow(unused)]
    pub order: usize,
}



#[derive(Default, Debug, Clone)]
pub struct ShellFieldCfg {
    // existing
    pub flag: Option<String>,
    pub opt_kv: Option<String>,
    pub positional: bool,
    pub positional_mode: Option<PosMode>,
    pub kv: Option<String>,
    pub prefix: Option<String>,
    pub eq: Option<String>,
    // NEW:
    // Option<T> => single arg: "{prefix}{value}"  e.g. "-s:/tmp/x"
    pub opt_prefix: Option<String>,

    // Option<T> => single arg: "{flag}={value}"  e.g. "--user=bob"
    pub opt_eq: Option<String>,

    // Positional => push (expr).to_string()
    pub arg_expr: Option<TokenStream2>,

    // Positional composite: base field joined with another field, optionally present
    // e.g. host + ":" + port if Some(port)
    pub arg_join_opt_with: Option<String>,
    pub arg_join_sep: Option<String>,

    // ordering
    pub order: Option<ShellOrder>,
    // ... your existing fields ...

    // NEW: Multi-value support
    // Tells the generator to loop over a Vec and repeat the flag/prefix
    pub multi_opt_kv: Option<String>,     // -e K1=V1 -e K2=V2
    pub multi_opt_prefix: Option<String>, // -I/path1 -I/path2
    
    // NEW: The "One Flag, Many Values" pattern
    // e.g. --tags val1 val2
    pub multi_arg_flag: Option<String>, 

    // NEW: The "Counter" pattern
    // e.g. value 3 => "-vvv"
    pub count_flag: Option<String>,

    // NEW: Boolean Negation
    // e.g. if false => "--no-check"
    pub flag_off: Option<String>,

    // NEW: enum mode + subcommand
    pub is_mode: bool,
    pub subcommand: bool,


    // CANT DO THIS TODAY I AM TOO SICK AND
    // I ALSO CANT FIND QUIET SPACE TO WORK
    // 
    // pubopt_join: Option<String>,
    // pubsep: Option<String>,
}
