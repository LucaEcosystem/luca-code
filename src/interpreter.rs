use std::{
    collections::{HashMap, VecDeque},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    ast::{BinaryOp, Block, CompareOp, ContainsMode, Expr, LengthArg, Param, Program, RaiseArg, Statement, UnaryOp},
    error::LucaError,
    lexer, parser,
    types::{values_equal, CollectionKind, Dec, DictEntry, Type, Value},
};

#[derive(Debug, Clone)]
struct Module {
    #[allow(dead_code)]
    name: String,
    globals: HashMap<String, Binding>,
    funcs: HashMap<String, Rc<Function>>,
    errors: HashMap<String, CustomError>,
}

#[derive(Debug, Clone)]
struct CustomError {
    /// Declared fields in definition order: implicit `code`/`message` first,
    /// then any extra body bindings sorted by name (deterministic display).
    fields: Vec<(String, Value)>,
    #[allow(dead_code)]
    line: usize,
    #[allow(dead_code)]
    column: usize,
}

impl CustomError {
    fn has(&self, field: &str) -> bool {
        self.fields.iter().any(|(name, _)| name == field)
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Function {
    name: String,
    params: Vec<Param>,
    return_type: Option<Type>,
    body: Block,
    closure_scopes: Vec<HashMap<String, Binding>>,
    closure_funcs: Vec<HashMap<String, Rc<Function>>>,
    line: usize,
    column: usize,
}

#[derive(Debug, Clone)]
enum BindingType {
    Scalar(Type),
    Collection(CollectionKind, Option<Type>),
}

#[derive(Debug, Clone)]
struct Binding { value: Value, binding_type: BindingType, mutable: bool }

impl Binding {
    #[allow(dead_code)]
    fn declared_type_for_messages(&self) -> String {
        match &self.binding_type {
            BindingType::Scalar(t) => t.to_string(),
            BindingType::Collection(kind, elem) => match elem {
                Some(t) => format!("{kind}({t})"),
                None => kind.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
enum ExecSignal {
    Continue,
    Return(Value, usize, usize),
    Stop(usize, usize),
    Skip(usize, usize),
}

pub struct Interpreter {
    scopes: Vec<HashMap<String, Binding>>,
    func_scopes: Vec<HashMap<String, Rc<Function>>>,
    error_scopes: Vec<HashMap<String, CustomError>>,
    output: Vec<String>,
    call_depth: usize,
    recursion_limit: usize,
    loop_depth: usize,
    iterating: Vec<String>,
    modules: HashMap<String, Module>,
    loading: Vec<String>,
    search_paths: Vec<PathBuf>,
    input_lines: Option<VecDeque<String>>,
    cli_args: Vec<String>,
    loaded_builtins: std::collections::HashSet<String>,
}

impl Default for Interpreter {
    fn default() -> Self { Self::new() }
}

impl Interpreter {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            scopes: vec![HashMap::new()],
            func_scopes: vec![HashMap::new()],
            error_scopes: vec![HashMap::new()],
            output: Vec::new(),
            call_depth: 0,
            recursion_limit: 64,
            loop_depth: 0,
            iterating: Vec::new(),
            modules: HashMap::new(),
            loading: Vec::new(),
            search_paths: vec![cwd],
            input_lines: None,
            cli_args: Vec::new(),
            loaded_builtins: std::collections::HashSet::new(),
        }
    }

    pub fn with_search_paths(search_paths: Vec<PathBuf>) -> Self {
        let mut interp = Self::new();
        interp.search_paths = search_paths;
        interp
    }

    /// Supply scripted input lines for `ask` (used by tests; the CLI reads stdin).
    pub fn with_input(mut self, lines: Vec<String>) -> Self {
        self.input_lines = Some(lines.into());
        self
    }

    /// Supply command-line arguments observed via `System.args()`.
    pub fn with_cli_args(mut self, args: Vec<String>) -> Self {
        self.cli_args = args;
        self
    }

    fn read_input_line(&mut self, line: usize, column: usize) -> Result<String, LucaError> {
        if let Some(queue) = self.input_lines.as_mut() {
            queue.pop_front().ok_or_else(|| LucaError::new("No more input available", line, column))
        } else {
            use std::io::BufRead;
            let mut buf = String::new();
            match std::io::stdin().lock().read_line(&mut buf) {
                Ok(0) => Err(LucaError::new("No more input available", line, column)),
                Ok(_) => {
                    while buf.ends_with('\n') || buf.ends_with('\r') {
                        buf.pop();
                    }
                    Ok(buf)
                }
                Err(e) => Err(LucaError::new(format!("Failed to read input: {e}"), line, column)),
            }
        }
    }

    fn warn(&self, message: &str, line: usize, column: usize) {
        warn_at(line, column, message);
    }

    fn check_not_iterating(&self, name: &str, line: usize, column: usize) -> Result<(), LucaError> {
        if self.iterating.iter().any(|n| n == name) {
            return Err(LucaError::new(format!("Cannot mutate collection '{name}' being iterated"), line, column));
        }
        Ok(())
    }

    fn check_element_matches(&self, value: &Value, expected: Option<Type>, line: usize, column: usize) -> Result<(), LucaError> {
        match expected {
            None => Ok(()),
            Some(Type::Uni) => Ok(()),
            Some(t) => {
                if value.is_none() {
                    return Ok(());
                }
                // For nested collections, value_type is Uni placeholder; compare via kind?
                // If expected is scalar but value is collection, error with kind name.
                if value.is_collection() {
                    return Err(LucaError::new(
                        format!("Expected {t}, found {}", type_name_for_value(value)),
                        line,
                        column,
                    ));
                }
                if value.value_type() == t {
                    Ok(())
                } else {
                    Err(LucaError::new(
                        format!("Expected {t}, found {}", type_name_for_value(value)),
                        line,
                        column,
                    ))
                }
            }
        }
    }

    fn current_scope(&self) -> &HashMap<String, Binding> {
        self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut HashMap<String, Binding> {
        self.scopes.last_mut().unwrap()
    }

    fn find_binding(&self, name: &str) -> Option<&Binding> {
        self.scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    fn find_binding_mut(&mut self, name: &str) -> Option<&mut Binding> {
        self.scopes.iter_mut().rev().find_map(|scope| scope.get_mut(name))
    }

    fn find_binding_scope_mut(&mut self, name: &str) -> Option<&mut HashMap<String, Binding>> {
        self.scopes.iter_mut().rev().find(|scope| scope.contains_key(name))
    }

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.func_scopes.push(HashMap::new());
        self.error_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.func_scopes.pop();
        self.error_scopes.pop();
    }

    fn current_error_scope(&self) -> &HashMap<String, CustomError> {
        self.error_scopes.last().unwrap()
    }

    fn current_error_scope_mut(&mut self) -> &mut HashMap<String, CustomError> {
        self.error_scopes.last_mut().unwrap()
    }

    fn current_func_scope(&self) -> &HashMap<String, Rc<Function>> {
        self.func_scopes.last().unwrap()
    }

    fn current_func_scope_mut(&mut self) -> &mut HashMap<String, Rc<Function>> {
        self.func_scopes.last_mut().unwrap()
    }

    fn find_function(&self, name: &str) -> Option<Rc<Function>> {
        for scope in self.func_scopes.iter().rev() {
            if let Some(f) = scope.get(name) {
                return Some(Rc::clone(f));
            }
        }
        // Also check variable scopes for closure captured functions? Functions are only in func_scopes, but closure captured them
        None
    }

    #[allow(dead_code)]
    fn find_function_with_closure(&self, name: &str) -> Option<Rc<Function>> {
        self.find_function(name)
    }
    pub fn run(&mut self, source: &str) -> Result<(), LucaError> {
        let tokens = lexer::lex(source)?;
        let program = parser::parse(tokens)?;
        match self.execute(&program) {
            Ok(()) => {
                for line in &self.output {
                    println!("{line}");
                }
                Ok(())
            }
            // `System.exit` terminates the process; flush pending output first.
            // Note this ends the test runner too, so tests never invoke it
            // with valid input.
            Err(error) if error.is_exit() => {
                for line in &self.output {
                    println!("{line}");
                }
                std::process::exit(error.exit_code.unwrap_or(1));
            }
            Err(error) => Err(error),
        }
    }
    pub fn run_collect(mut self, source: &str) -> Result<Vec<String>, LucaError> {
        let tokens = lexer::lex(source)?;
        let program = parser::parse(tokens)?;
        match self.execute(&program) {
            Ok(()) => Ok(self.output),
            Err(error) if error.is_exit() => {
                for line in &self.output {
                    println!("{line}");
                }
                std::process::exit(error.exit_code.unwrap_or(1));
            }
            Err(error) => Err(error),
        }
    }
    fn execute(&mut self, program: &Program) -> Result<(), LucaError> {
        match self.execute_block_statements(&program.statements)? {
            ExecSignal::Continue => Ok(()),
            ExecSignal::Return(_, line, column) => Err(LucaError::new("return outside function", line, column)),
            ExecSignal::Stop(line, column) => Err(LucaError::new("stop outside loop", line, column)),
            ExecSignal::Skip(line, column) => Err(LucaError::new("skip outside loop", line, column)),
        }
    }

    fn warn_unreachable_after(&self, statements: &[Statement], index: usize) {
        // Locked reference line 109: an unconditional `return`/`stop` making
        // following code unreachable reports a Warning, not an Error.
        // (`skip` is intentionally excluded by the reference.)
        if let Some(next) = statements.get(index + 1) {
            let (line, column) = statement_location(next);
            self.warn("Unreachable code", line, column);
        }
    }

    fn execute_block_statements(&mut self, statements: &[Statement]) -> Result<ExecSignal, LucaError> {
        for (index, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Declare { name, declared_type, mutable, value, line, column } => {
                    self.exec_declare(name, declared_type, *mutable, value, *line, *column)?;
                }
                Statement::DeclareCollection { kind, name, element_type, value, line, column } => {
                    self.exec_declare_collection(*kind, name, element_type, value, *line, *column)?;
                }
                Statement::Assign { name, asserted_type, value, line, column } => {
                    self.exec_assign(name, asserted_type, value, *line, *column)?;
                }
                Statement::Print { value, .. } => {
                    let val = self.evaluate(value)?;
                    self.output.push(val.display());
                }
                Statement::Forget { name, is_const, asserted_type, line, column } => {
                    self.exec_forget(name, *is_const, asserted_type, *line, *column)?;
                }
                Statement::Clear { name, line, column } => {
                    self.exec_clear(name, *line, *column)?;
                }
                Statement::Change { name, new_type, value, line, column } => {
                    self.exec_change(name, *new_type, value, *line, *column)?;
                }
                Statement::ChangeIndex { name, index, value, line, column } => {
                    self.execute_change_index(name, index, value, *line, *column)?;
                }
                Statement::CollectionAdd { name, args, line, column } => {
                    self.execute_collection_add(name, args, *line, *column)?;
                }
                Statement::CollectionRemove { name, arg, line, column } => {
                    self.execute_collection_remove(name, arg, *line, *column)?;
                }
                Statement::If { branches, else_branch, line, column } => {
                    match self.execute_if(branches, else_branch.as_ref(), *line, *column)? {
                        ExecSignal::Continue => {},
                        other => return Ok(other),
                    }
                }
                Statement::ForTimes { count, body, line, column } => {
                    match self.execute_for_times(count, body, *line, *column)? {
                        ExecSignal::Continue => {},
                        ExecSignal::Return(v, l, c) => return Ok(ExecSignal::Return(v, l, c)),
                        ExecSignal::Stop(_, _) => {}, // stop already handled inside
                        ExecSignal::Skip(_, _) => unreachable!("skip should be handled inside loop"),
                    }
                }
                Statement::ForEach { item, collection, body, line, column } => {
                    match self.execute_for_each(item, collection, body, *line, *column)? {
                        ExecSignal::Continue => {},
                        ExecSignal::Return(v, l, c) => return Ok(ExecSignal::Return(v, l, c)),
                        ExecSignal::Stop(_, _) => {},
                        ExecSignal::Skip(_, _) => unreachable!(),
                    }
                }
                Statement::While { condition, body, line, column } => {
                    match self.execute_while(condition, body, *line, *column)? {
                        ExecSignal::Continue => {},
                        ExecSignal::Return(v, l, c) => return Ok(ExecSignal::Return(v, l, c)),
                        ExecSignal::Stop(_, _) => {},
                        ExecSignal::Skip(_, _) => unreachable!(),
                    }
                }
                Statement::Stop { line, column } => {
                    if self.loop_depth == 0 {
                        return Err(LucaError::new("stop outside loop", *line, *column));
                    }
                    self.warn_unreachable_after(statements, index);
                    return Ok(ExecSignal::Stop(*line, *column));
                }
                Statement::Skip { line, column } => {
                    if self.loop_depth == 0 {
                        return Err(LucaError::new("skip outside loop", *line, *column));
                    }
                    return Ok(ExecSignal::Skip(*line, *column));
                }
                Statement::FuncDef { name, params, return_type, body, line, column } => {
                    self.exec_funcdef(name, params, return_type, body, *line, *column)?;
                }
                Statement::Return { value, line, column } => {
                    if self.call_depth == 0 {
                        return Err(LucaError::new("return outside function", *line, *column));
                    }
                    let ret_val = if let Some(expr) = value {
                        self.evaluate(expr)?
                    } else {
                        Value::None
                    };
                    self.warn_unreachable_after(statements, index);
                    return Ok(ExecSignal::Return(ret_val, *line, *column));
                }
                Statement::Call { name, args, line, column } => {
                    // Evaluate call as statement, discard return value
                    let _ = self.evaluate_call(name, args, *line, *column)?;
                }
                Statement::MemberCall { object, member, args, line, column } => {
                    // A direct `System.exit(...)` statement terminates the process,
                    // so any following code in this block is unreachable. (Exit in
                    // expression position terminates the same way via dispatch.)
                    if member == "exit" {
                        if let Expr::Variable { name, .. } = object.as_ref() {
                            if name == "System" {
                                self.warn_unreachable_after(statements, index);
                            }
                        }
                    }
                    let _ = self.evaluate_member_call(object, member, args, *line, *column)?;
                }
                Statement::Import { name, line, column } => {
                    self.import_module(name, *line, *column)?;
                }
                Statement::Try { try_block, capture_block, finally_block, line: _, column: _ } => {
                    match self.execute_try(try_block, capture_block.as_ref(), finally_block.as_ref())? {
                        ExecSignal::Continue => {}
                        other => return Ok(other),
                    }
                }
                Statement::ErrorDef { name, body, line, column } => {
                    match self.execute_error_def(name, body, *line, *column)? {
                        ExecSignal::Continue => {}
                        other => return Ok(other),
                    }
                }
                Statement::Raise { name, overrides, line, column } => {
                    self.warn_unreachable_after(statements, index);
                    return Err(self.exec_raise(name, overrides, *line, *column));
                }
            }
        }
        Ok(ExecSignal::Continue)
    }

    // Statement helpers below keep `execute_block_statements` small so deep
    // recursion (functions calling functions) stays within default thread
    // stacks even in unoptimized builds. They implement exactly what the
    // inlined match arms did before; no behavior change.
    fn exec_declare(
        &mut self,
        name: &str,
        declared_type: &Option<Type>,
        mutable: bool,
        value: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        if self.current_scope().contains_key(name)
            || self.current_func_scope().contains_key(name)
            || self.current_error_scope().contains_key(name)
            || self.modules.contains_key(name)
            || self.loaded_builtins.contains(name)
        {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        if self.find_function(name).is_some() && self.current_func_scope().contains_key(name) {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        let evaluated = self.evaluate(value)?;
        let value = self.apply_ask_conversion(value, evaluated, *declared_type)?;
        if value.is_collection() {
            if let Some(t) = declared_type {
                return Err(LucaError::new(
                    format!("Expected {t}, found {}", type_name_for_value(&value)),
                    line,
                    column,
                ));
            }
                        let kind = value.collection_kind().unwrap();
                        let elem = match &value {
                            Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                                infer_element_type(&items.borrow())
                            }
                            Value::Dict(_) => None,
                            _ => None,
                        };
                        self.current_scope_mut().insert(
                            name.to_owned(),
                            Binding { value, binding_type: BindingType::Collection(kind, elem), mutable },
                        );
        } else {
            let inferred = if let Some(t) = declared_type {
                self.require_type(&value, *t, line, column)?;
                *t
            } else {
                if value.is_none() {
                    return Err(LucaError::new("Cannot infer type from 'none'; explicit type required", line, column));
                }
                value.value_type()
            };
            self.current_scope_mut().insert(
                name.to_owned(),
                Binding { value, binding_type: BindingType::Scalar(inferred), mutable },
            );
        }
        Ok(())
    }

    fn exec_declare_collection(
        &mut self,
        kind: CollectionKind,
        name: &str,
        element_type: &Option<Type>,
        value: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        if self.current_scope().contains_key(name)
            || self.current_func_scope().contains_key(name)
            || self.current_error_scope().contains_key(name)
            || self.modules.contains_key(name)
            || self.loaded_builtins.contains(name)
        {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        let mut value = self.evaluate(value)?;
        value = convert_empty_set_dict(value, kind);
        let actual_kind = value.collection_kind().ok_or_else(|| {
            LucaError::new(format!("Expected {kind}, found {}", type_name_for_value(&value)), line, column)
        })?;
        if actual_kind != kind {
            return Err(LucaError::new(
                format!("Expected {kind}, found {actual_kind}"),
                line,
                column,
            ));
        }
        match (&value, kind) {
                        (Value::List(items), CollectionKind::List)
                        | (Value::Tuple(items), CollectionKind::Tuple)
                        | (Value::Set(items), CollectionKind::Set) => {
                            let items = items.borrow().clone();
                            if let Some(t) = element_type {
                    for item in &items {
                        self.check_element_matches(item, Some(*t), line, column)?;
                    }
                }
                // Deduplicate sets with warning
                let value = if kind == CollectionKind::Set {
                    let mut unique: Vec<Value> = Vec::new();
                    for item in items {
                        let mut dup = false;
                        for u in &unique {
                            if let Some(true) = values_equal(&item, u) {
                                dup = true;
                                break;
                            }
                        }
                        if dup {
                            self.warn("Duplicate set value", line, column);
                        } else {
                            unique.push(item);
                        }
                    }
                                Value::new_set(unique)
                            } else {
                                value
                            };
                let stored_elem = *element_type;
                self.current_scope_mut().insert(
                    name.to_owned(),
                    Binding { value, binding_type: BindingType::Collection(kind, stored_elem), mutable: true },
                );
            }
            (Value::Dict(_), CollectionKind::Dict) => {
                // Per-value annotations already checked during literal evaluation
                self.current_scope_mut().insert(
                    name.to_owned(),
                    Binding { value, binding_type: BindingType::Collection(kind, None), mutable: true },
                );
            }
            _ => {
                return Err(LucaError::new(
                    format!("Expected {kind}, found {actual_kind}"),
                    line,
                    column,
                ));
            }
        }
        Ok(())
    }

    fn exec_assign(
        &mut self,
        name: &str,
        asserted_type: &Option<Type>,
        value: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        let binding_kind = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            if !b.mutable {
                return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
            }
            b.binding_type.clone()
        };
        match binding_kind {
            BindingType::Scalar(t) => {
                if asserted_type.is_some_and(|ty| ty != t) {
                    return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), line, column));
                }
                let evaluated = self.evaluate(value)?;
                let value = self.apply_ask_conversion(value, evaluated, Some(t))?;
                self.require_type(&value, t, line, column)?;
                let target = self.find_binding_mut(name).unwrap();
                target.value = value;
            }
            BindingType::Collection(kind, elem) => {
                if let Some(a) = asserted_type {
                    match elem {
                        Some(e) if *a != e => {
                            return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), line, column));
                        }
                        _ => {}
                    }
                }
                let effective = (*asserted_type).or(elem);
                let mut value = self.evaluate(value)?;
                value = convert_empty_set_dict(value, kind);
                let actual = value.collection_kind().ok_or_else(|| {
                    LucaError::new(format!("Expected {kind}, found {}", type_name_for_value(&value)), line, column)
                })?;
                if actual != kind {
                    return Err(LucaError::new(format!("Expected {kind}, found {actual}"), line, column));
                }
                            match &value {
                                Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                                    for item in items.borrow().iter() {
                                        self.check_element_matches(item, effective, line, column)?;
                                    }
                                }
                                Value::Dict(_) => {}
                                _ => unreachable!(),
                            }
                            let target = self.find_binding_mut(name).unwrap();
                            target.value = value;
            }
        }
        Ok(())
    }

    fn exec_forget(
        &mut self,
        name: &str,
        is_const: bool,
        asserted_type: &Option<Type>,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        let (is_binding_const, binding_kind) = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            (!b.mutable, b.binding_type.clone())
        };
        if is_const != is_binding_const {
            let kind = if is_const { "const" } else { "var" };
            return Err(LucaError::new(format!("'{}' is not a {kind}", name), line, column));
        }
        if let Some(t) = asserted_type {
            match binding_kind {
                BindingType::Scalar(decl) if *t != decl => {
                    return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), line, column));
                }
                BindingType::Collection(_, Some(elem)) if *t != elem => {
                    return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), line, column));
                }
                _ => {}
            }
        }
        // Remove from the scope where it is found
        let scope = self.find_binding_scope_mut(name).unwrap();
        scope.remove(name);
        Ok(())
    }

    fn exec_clear(&mut self, name: &str, line: usize, column: usize) -> Result<(), LucaError> {
        let binding = self.find_binding_mut(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
        if !binding.mutable {
            return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
        }
        // Clear empties shared storage in place so every alias observes it.
        match &mut binding.value {
            Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.borrow_mut().clear(),
            Value::Dict(entries) => entries.borrow_mut().clear(),
            _ => {
                binding.value = Value::None;
            }
        }
        Ok(())
    }

    fn exec_change(
        &mut self,
        name: &str,
        new_type: Type,
        value: &Expr,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        let binding_kind = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            if !b.mutable {
                return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
            }
            b.binding_type.clone()
        };
        match binding_kind {
            BindingType::Scalar(_) => {
                let evaluated = self.evaluate(value)?;
                let value = self.apply_ask_conversion(value, evaluated, Some(new_type))?;
                self.require_type(&value, new_type, line, column)?;
                let b = self.find_binding_mut(name).unwrap();
                b.binding_type = BindingType::Scalar(new_type);
                b.value = value;
            }
            BindingType::Collection(kind, _) => {
                let mut value = self.evaluate(value)?;
                value = convert_empty_set_dict(value, kind);
                let actual = value.collection_kind().ok_or_else(|| {
                    LucaError::new(format!("Expected {kind}, found {}", type_name_for_value(&value)), line, column)
                })?;
                if actual != kind {
                    return Err(LucaError::new(format!("Expected {kind}, found {actual}"), line, column));
                }
                            match &value {
                                Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                                    for item in items.borrow().iter() {
                                        self.check_element_matches(item, Some(new_type), line, column)?;
                                    }
                        let b = self.find_binding_mut(name).unwrap();
                        b.binding_type = BindingType::Collection(kind, Some(new_type));
                        b.value = value;
                    }
                    Value::Dict(_) => {
                        let b = self.find_binding_mut(name).unwrap();
                        b.value = value;
                    }
                    _ => unreachable!(),
                }
            }
        }
        Ok(())
    }

    fn exec_funcdef(
        &mut self,
        name: &str,
        params: &[Param],
        return_type: &Option<Type>,
        body: &Block,
        line: usize,
        column: usize,
    ) -> Result<(), LucaError> {
        if self.current_scope().contains_key(name)
            || self.current_func_scope().contains_key(name)
            || self.current_error_scope().contains_key(name)
            || self.modules.contains_key(name)
            || self.loaded_builtins.contains(name)
        {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        // Check duplicate param names
        let mut seen = std::collections::HashSet::new();
        for p in params {
            if !seen.insert(&p.name) {
                return Err(LucaError::new(format!("Duplicate parameter '{}'", p.name), p.line, p.column));
            }
            if let Some(t) = p.ty {
                // Validate type is known (already parsed)
                let _ = t;
            }
        }
        let func = Rc::new(Function {
            name: name.to_owned(),
            params: params.to_vec(),
            return_type: *return_type,
            body: body.clone(),
            closure_scopes: self.scopes.clone(),
            closure_funcs: self.func_scopes.clone(),
            line,
            column,
        });
        self.current_func_scope_mut().insert(name.to_owned(), func);
        Ok(())
    }

    fn run_finally(&mut self, finally_block: Option<&Block>) -> Result<ExecSignal, LucaError> {
        match finally_block {
            None => Ok(ExecSignal::Continue),
            Some(block) => {
                self.push_scope();
                let result = self.execute_block_statements(&block.statements);
                self.pop_scope();
                result
            }
        }
    }

    fn execute_try(
        &mut self,
        try_block: &Block,
        capture_block: Option<&Block>,
        finally_block: Option<&Block>,
    ) -> Result<ExecSignal, LucaError> {
        self.push_scope();
        let try_result = self.execute_block_statements(&try_block.statements);
        self.pop_scope();
        match try_result {
            Ok(signal) => match self.run_finally(finally_block) {
                Ok(ExecSignal::Continue) => Ok(signal),
                Ok(other) => Ok(other),
                Err(e) => Err(e),
            },
            // Process termination is never captured: `finally` still runs,
            // then termination continues.
            Err(error) if error.is_exit() => match self.run_finally(finally_block) {
                Ok(ExecSignal::Continue) => Err(error),
                Ok(other) => Ok(other),
                Err(e) => Err(e),
            },
            Err(error) => match capture_block {
                Some(cap) => {
                    self.push_scope();
                    // Bind the caught error for the duration of the capture block.
                    let caught = match error.error_value.clone() {
                        Some(value) => value,
                        None => Value::new_dict(vec![
                            DictEntry { key: "code".to_owned(), value: Value::None },
                            DictEntry { key: "message".to_owned(), value: Value::Str(error.message.clone()) },
                            DictEntry { key: "line".to_owned(), value: Value::Int(error.line as i64) },
                            DictEntry { key: "column".to_owned(), value: Value::Int(error.column as i64) },
                        ]),
                    };
                    self.current_scope_mut().insert(
                        "error".to_owned(),
                        Binding { value: caught, binding_type: BindingType::Collection(CollectionKind::Dict, None), mutable: true },
                    );
                    let cap_result = self.execute_block_statements(&cap.statements);
                    self.pop_scope();
                    match cap_result {
                        Ok(ExecSignal::Continue) => match self.run_finally(finally_block) {
                            Ok(ExecSignal::Continue) => Ok(ExecSignal::Continue),
                            Ok(other) => Ok(other),
                            Err(e) => Err(e),
                        },
                        Ok(other) => match self.run_finally(finally_block) {
                            Ok(ExecSignal::Continue) => Ok(other),
                            Ok(final_other) => Ok(final_other),
                            Err(e) => Err(e),
                        },
                        Err(capture_error) => match self.run_finally(finally_block) {
                            Ok(ExecSignal::Continue) => Err(capture_error),
                            Ok(other) => Ok(other),
                            Err(e) => Err(e),
                        },
                    }
                }
                None => match self.run_finally(finally_block) {
                    Ok(ExecSignal::Continue) => Err(error),
                    Ok(other) => Ok(other),
                    Err(e) => Err(e),
                },
            },
        }
    }

    fn find_custom_error(&self, name: &str) -> Option<&CustomError> {
        self.error_scopes.iter().rev().find_map(|scope| scope.get(name))
    }

    /// Builds the raised error value. Always returns an error (raising never
    /// produces a value); callers propagate it like any runtime error.
    fn exec_raise(&mut self, name: &str, overrides: &[RaiseArg], line: usize, column: usize) -> LucaError {
        let template = match self.find_custom_error(name).cloned() {
            Some(template) => template,
            None => {
                return if self.find_binding(name).is_some()
                    || self.find_function(name).is_some()
                    || self.modules.contains_key(name)
                    || self.loaded_builtins.contains(name)
                {
                    LucaError::new(format!("'{name}' is not an error definition"), line, column)
                } else {
                    LucaError::new(format!("'{name}' is not declared"), line, column)
                };
            }
        };
        // Evaluate overrides in the caller's scope; repeats warn (first wins).
        let mut given: Vec<(String, Value, usize, usize)> = Vec::new();
        for arg in overrides {
            let value = match self.evaluate(&arg.value) {
                Ok(value) => value,
                Err(error) => return error,
            };
            if given.iter().any(|(field, _, _, _)| field == &arg.field) {
                self.warn(format!("Duplicate field '{}'", arg.field).as_str(), arg.line, arg.column);
            } else {
                given.push((arg.field.clone(), value, arg.line, arg.column));
            }
        }
        // Start from declared fields, applying overrides with type checks.
        let mut fields: Vec<(String, Value)> = Vec::with_capacity(template.fields.len());
        for (field_name, field_value) in &template.fields {
            match given.iter().find(|(name, _, _, _)| name == field_name) {
                Some((_, override_value, _, _)) => {
                    if !field_value.is_none() && !override_value.is_none() {
                        if field_value.is_collection() || override_value.is_collection() {
                            let old_kind = field_value.collection_kind();
                            let new_kind = override_value.collection_kind();
                            if old_kind != new_kind {
                                return LucaError::new(
                                    format!(
                                        "Cannot change type of field '{field_name}'; expected {}, found {}",
                                        type_name_for_value(field_value),
                                        type_name_for_value(override_value)
                                    ),
                                    line,
                                    column,
                                );
                            }
                        } else if field_value.value_type() != override_value.value_type() {
                            return LucaError::new(
                                format!(
                                    "Expected {}, found {} for field '{field_name}'",
                                    field_value.value_type(),
                                    type_name_for_value(override_value)
                                ),
                                line,
                                column,
                            );
                        }
                    }
                    fields.push((field_name.clone(), override_value.clone()));
                }
                None => fields.push((field_name.clone(), field_value.clone())),
            }
        }
        for (field_name, _, field_line, field_column) in &given {
            if !template.has(field_name) {
                return LucaError::new(
                    format!("'{name}' has no field '{field_name}'"),
                    *field_line,
                    *field_column,
                );
            }
        }
        let message = match fields.iter().find(|(field_name, _)| field_name == "message") {
            Some((_, Value::Str(text))) => text.clone(),
            Some((_, other)) => other.display(),
            None => String::new(),
        };
        let mut entries: Vec<DictEntry> = fields
            .into_iter()
            .filter(|(field_name, _)| field_name != "line" && field_name != "column")
            .map(|(field_name, field_value)| DictEntry { key: field_name, value: field_value })
            .collect();
        entries.push(DictEntry { key: "line".to_owned(), value: Value::Int(line as i64) });
        entries.push(DictEntry { key: "column".to_owned(), value: Value::Int(column as i64) });
        let raised = Value::new_dict(entries);
        LucaError::raised(message, raised, line, column)
    }

    fn execute_error_def(&mut self, name: &str, body: &Block, line: usize, column: usize) -> Result<ExecSignal, LucaError> {
        if self.current_scope().contains_key(name)
            || self.current_func_scope().contains_key(name)
            || self.current_error_scope().contains_key(name)
            || self.modules.contains_key(name)
            || self.loaded_builtins.contains(name)
        {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        self.push_scope();
        self.current_scope_mut().insert(
            "code".to_owned(),
            Binding { value: Value::None, binding_type: BindingType::Scalar(Type::Uni), mutable: true },
        );
        self.current_scope_mut().insert(
            "message".to_owned(),
            Binding { value: Value::None, binding_type: BindingType::Scalar(Type::Uni), mutable: true },
        );
        let result = self.execute_block_statements(&body.statements);
        match result {
            Ok(ExecSignal::Continue) => {
                let scope = self.current_scope();
                let code = scope.get("code").map(|b| b.value.clone()).unwrap_or(Value::None);
                let message = scope.get("message").map(|b| b.value.clone()).unwrap_or(Value::None);
                let mut extras: Vec<(String, Value)> = scope
                    .iter()
                    .filter(|(binding_name, _)| {
                        *binding_name != "code" && *binding_name != "message" && *binding_name != "line" && *binding_name != "column"
                    })
                    .map(|(binding_name, binding)| (binding_name.clone(), binding.value.clone()))
                    .collect();
                extras.sort_by(|a, b| a.0.cmp(&b.0));
                let mut fields = vec![("code".to_owned(), code), ("message".to_owned(), message)];
                fields.extend(extras);
                self.pop_scope();
                self.current_error_scope_mut().insert(
                    name.to_owned(),
                    CustomError { fields, line, column },
                );
                Ok(ExecSignal::Continue)
            }
            Ok(other) => {
                // Return/Stop/Skip signals propagate naturally (top-level `execute`
                // converts stray ones to outside-function/loop errors).
                self.pop_scope();
                Ok(other)
            }
            Err(e) => {
                self.pop_scope();
                Err(e)
            }
        }
    }

    fn resolve_module_file(&self, name: &str, line: usize, column: usize) -> Result<PathBuf, LucaError> {
        let file_name = format!("{name}.lucc");
        for dir in &self.search_paths {
            let candidate = dir.join(&file_name);
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
        Err(LucaError::new(format!("missing module '{name}'"), line, column))
    }

    fn import_module(&mut self, name: &str, line: usize, column: usize) -> Result<(), LucaError> {
        if self.modules.contains_key(name) || self.loaded_builtins.contains(name) {
            return Ok(());
        }
        if self.loading.iter().any(|n| n == name) {
            return Err(LucaError::new(format!("Circular import '{name}'"), line, column));
        }
        if self.current_scope().contains_key(name)
            || self.current_func_scope().contains_key(name)
            || self.current_error_scope().contains_key(name)
            || self.find_function(name).is_some()
        {
            return Err(LucaError::new(format!("'{name}' is already declared"), line, column));
        }
        // Built-in standard modules take precedence over same-named files.
        if crate::stdlib::is_builtin(name) {
            self.loaded_builtins.insert(name.to_owned());
            return Ok(());
        }
        let path = self.resolve_module_file(name, line, column)?;
        let source = std::fs::read_to_string(&path)
            .map_err(|e| LucaError::new(format!("Could not read module '{name}': {e}"), line, column))?;
        let tokens = lexer::lex(&source)
            .map_err(|e| LucaError::new(format!("In module '{name}': {}", e.message), e.line, e.column))?;
        let program = parser::parse(tokens)
            .map_err(|e| LucaError::new(format!("In module '{name}': {}", e.message), e.line, e.column))?;
        self.loading.push(name.to_owned());
        let saved_scopes = std::mem::replace(&mut self.scopes, vec![HashMap::new()]);
        let saved_funcs = std::mem::replace(&mut self.func_scopes, vec![HashMap::new()]);
        let saved_errors = std::mem::replace(&mut self.error_scopes, vec![HashMap::new()]);
        let saved_call = self.call_depth;
        let saved_loop = self.loop_depth;
        let saved_iterating = std::mem::take(&mut self.iterating);
        self.call_depth = 0;
        self.loop_depth = 0;
        let result = self.execute(&program);
        let module_scopes = std::mem::replace(&mut self.scopes, saved_scopes);
        let module_funcs = std::mem::replace(&mut self.func_scopes, saved_funcs);
        let module_errors = std::mem::replace(&mut self.error_scopes, saved_errors);
        self.call_depth = saved_call;
        self.loop_depth = saved_loop;
        self.iterating = saved_iterating;
        match result {
            Ok(()) => {
                let globals = module_scopes.into_iter().next().unwrap_or_default();
                let funcs = module_funcs.into_iter().next().unwrap_or_default();
                let errors = module_errors.into_iter().next().unwrap_or_default();
                self.modules.insert(
                    name.to_owned(),
                    Module { name: name.to_owned(), globals, funcs, errors },
                );
                self.loading.pop();
                Ok(())
            }
            Err(e) => {
                self.loading.pop();
                // Process termination is not an initialization error: it keeps
                // its identity (and exit code) instead of gaining the prefix.
                if e.is_exit() {
                    return Err(e);
                }
                let mut wrapped = LucaError::new(format!("In module '{name}': {}", e.message), e.line, e.column);
                wrapped.error_value = e.error_value;
                Err(wrapped)
            }
        }
    }

    fn module_member_value(&self, module_name: &str, member: &str, line: usize, column: usize) -> Result<Value, LucaError> {
        let module = self
            .modules
            .get(module_name)
            .ok_or_else(|| LucaError::new(format!("Module '{module_name}' is not imported"), line, column))?;
        if let Some(binding) = module.globals.get(member) {
            Ok(binding.value.clone())
        } else if module.funcs.contains_key(member) {
            Err(LucaError::new(
                format!("'{member}' is a function; use '{module_name}.{member}(...)' to call it"),
                line,
                column,
            ))
        } else if module.errors.contains_key(member) {
            Err(LucaError::new(
                format!("'{member}' is an error definition, not a value"),
                line,
                column,
            ))
        } else {
            Err(LucaError::new(
                format!("Module '{module_name}' has no member '{member}'"),
                line,
                column,
            ))
        }
    }

    fn evaluate_member(&mut self, base: &Expr, member: &str, line: usize, column: usize) -> Result<Value, LucaError> {
        if let Expr::Variable { name: base_name, .. } = base {
            if self.loaded_builtins.contains(base_name) {
                if let Some(value) = crate::stdlib::builtin_member_value(base_name, member) {
                    return Ok(value);
                }
                if crate::stdlib::is_native_function(base_name, member) {
                    return Err(LucaError::new(
                        format!("'{member}' is a function; use '{base_name}.{member}(...)' to call it"),
                        line,
                        column,
                    ));
                }
                return Err(LucaError::new(
                    format!("Module '{base_name}' has no member '{member}'"),
                    line,
                    column,
                ));
            }
            if self.modules.contains_key(base_name) {
                return self.module_member_value(base_name, member, line, column);
            }
            if self.find_binding(base_name).is_some() {
                return Err(LucaError::new(format!("Unknown method '{member}'"), line, column));
            }
            return Err(LucaError::new(format!("Module '{base_name}' is not imported"), line, column));
        }
        Err(LucaError::new(format!("Unknown method '{member}'"), line, column))
    }

    fn evaluate_member_call(&mut self, base: &Expr, member: &str, args: &[Expr], line: usize, column: usize) -> Result<Value, LucaError> {
        if let Expr::Variable { name: base_name, .. } = base {
            if self.loaded_builtins.contains(base_name) {
                return self.call_native_function(base_name, member, args, line, column);
            }
            if self.modules.contains_key(base_name) {
                return self.call_module_function(base_name, member, args, line, column);
            }
            if self.find_binding(base_name).is_some() {
                return Err(LucaError::new(format!("Unknown method '{member}'"), line, column));
            }
            return Err(LucaError::new(format!("Module '{base_name}' is not imported"), line, column));
        }
        Err(LucaError::new(format!("Unknown method '{member}'"), line, column))
    }

    fn call_native_function(
        &mut self,
        module_name: &str,
        member: &str,
        args: &[Expr],
        line: usize,
        column: usize,
    ) -> Result<Value, LucaError> {
        if !crate::stdlib::is_native_function(module_name, member) {
            return Err(LucaError::new(
                format!("Module '{module_name}' has no member '{member}'"),
                line,
                column,
            ));
        }
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            values.push(self.evaluate(arg)?);
        }
        if values.iter().any(|v| v.is_none()) {
            return Err(LucaError::new(
                format!("Cannot use 'none' as an argument to '{module_name}.{member}'"),
                line,
                column,
            ));
        }
        crate::stdlib::call(module_name, member, values, &self.cli_args, line, column)
    }

    fn call_module_function(&mut self, module_name: &str, func_name: &str, args: &[Expr], line: usize, column: usize) -> Result<Value, LucaError> {
        let module = self
            .modules
            .get(module_name)
            .ok_or_else(|| LucaError::new(format!("Module '{module_name}' is not imported"), line, column))?
            .clone();
        let func = module
            .funcs
            .get(func_name)
            .ok_or_else(|| {
                if module.globals.contains_key(func_name) {
                    LucaError::new(
                        format!("'{func_name}' is not a function in module '{module_name}'"),
                        line,
                        column,
                    )
                } else if module.errors.contains_key(func_name) {
                    LucaError::new(
                        format!("'{func_name}' is an error definition, not a function"),
                        line,
                        column,
                    )
                } else {
                    LucaError::new(
                        format!("Module '{module_name}' has no member '{func_name}'"),
                        line,
                        column,
                    )
                }
            })?
            .clone();
        if self.call_depth >= self.recursion_limit {
            return Err(LucaError::new("Recursion limit exceeded", line, column));
        }
        if args.len() > func.params.len() {
            return Err(LucaError::new(
                format!("Expected {} arguments, found {}", func.params.len(), args.len()),
                line,
                column,
            ));
        }
        let mut evaluated_args: Vec<Value> = Vec::new();
        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }
        let mut final_args: Vec<Value> = Vec::new();
        for (i, param) in func.params.iter().enumerate() {
            if i < evaluated_args.len() {
                let val = evaluated_args[i].clone();
                let val = self.apply_ask_conversion(&args[i], val, param.ty)?;
                if let Some(ty) = param.ty {
                    self.require_type(&val, ty, param.line, param.column)?;
                }
                final_args.push(val);
            } else if let Some(default_expr) = &param.default {
                let val = self.evaluate(default_expr)?;
                let val = self.apply_ask_conversion(default_expr, val, param.ty)?;
                if let Some(ty) = param.ty {
                    self.require_type(&val, ty, param.line, param.column)?;
                }
                final_args.push(val);
            } else {
                return Err(LucaError::new(format!("Missing argument for '{}'", param.name), line, column));
            }
        }
        let saved_scopes = std::mem::replace(&mut self.scopes, vec![module.globals.clone()]);
        let saved_funcs = std::mem::replace(&mut self.func_scopes, vec![module.funcs.clone()]);
        let saved_errors = std::mem::replace(&mut self.error_scopes, vec![module.errors.clone()]);
        let saved_loop = self.loop_depth;
        let saved_iterating = std::mem::take(&mut self.iterating);
        self.loop_depth = 0;
        self.call_depth += 1;
        self.push_scope();
        if let Err(e) = self.bind_call_params(&func.params, final_args) {
            self.pop_scope();
            self.call_depth -= 1;
            self.scopes = saved_scopes;
            self.func_scopes = saved_funcs;
            self.error_scopes = saved_errors;
            self.loop_depth = saved_loop;
            self.iterating = saved_iterating;
            return Err(in_module_error(module_name, e));
        }
        let result = self.execute_function_body(&func);
        self.pop_scope();
        self.call_depth -= 1;
        let updated_globals = self.scopes.first().cloned().unwrap_or_default();
        self.scopes = saved_scopes;
        self.func_scopes = saved_funcs;
        self.error_scopes = saved_errors;
        self.loop_depth = saved_loop;
        self.iterating = saved_iterating;
        if let Some(module) = self.modules.get_mut(module_name) {
            module.globals = updated_globals;
        }
        result.map_err(|e| in_module_error(module_name, e))
    }

    fn execute_if(&mut self, branches: &[(Expr, crate::ast::Block)], else_branch: Option<&crate::ast::Block>, line: usize, column: usize) -> Result<ExecSignal, LucaError> {
        for (cond, block) in branches {
            let val = self.evaluate(cond)?;
            if val.is_none() {
                return Err(LucaError::new("Condition must be bool; found none", line, column));
            }
            let cond_bool = match val {
                Value::Bool(b) => b,
                _ => return Err(LucaError::new(format!("Condition must be bool; found {}", val.display()), line, column)),
            };
            if cond_bool {
                self.push_scope();
                let res = self.execute_block_statements(&block.statements);
                self.pop_scope();
                return res;
            }
        }
        if let Some(block) = else_branch {
            self.push_scope();
            let res = self.execute_block_statements(&block.statements);
            self.pop_scope();
            return res;
        }
        Ok(ExecSignal::Continue)
    }

    fn execute_for_times(&mut self, count_expr: &Expr, body: &Block, line: usize, column: usize) -> Result<ExecSignal, LucaError> {
        let count_val = self.evaluate(count_expr)?;
        if count_val.is_none() {
            return Err(LucaError::new("for count must be int; found none", line, column));
        }
        let count = match count_val {
            Value::Int(n) => n,
            _ => return Err(LucaError::new(format!("for count must be int; found {}", count_val.display()), line, column)),
        };
        if count < 0 {
            return Err(LucaError::new("for count must be non-negative", line, column));
        }
        self.loop_depth += 1;
        for _ in 0..count {
            self.push_scope();
            let res = self.execute_block_statements(&body.statements);
            self.pop_scope();
            match res {
                Ok(ExecSignal::Continue) => {},
                Ok(ExecSignal::Return(v, l, c)) => {
                    self.loop_depth -= 1;
                    return Ok(ExecSignal::Return(v, l, c));
                }
                Ok(ExecSignal::Stop(_, _)) => break,
                Ok(ExecSignal::Skip(_, _)) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }
        self.loop_depth -= 1;
        Ok(ExecSignal::Continue)
    }

    fn execute_while(&mut self, condition: &Expr, body: &Block, line: usize, column: usize) -> Result<ExecSignal, LucaError> {
        self.loop_depth += 1;
        loop {
            let val = self.evaluate(condition)?;
            if val.is_none() {
                self.loop_depth -= 1;
                return Err(LucaError::new("Condition must be bool; found none", line, column));
            }
            let cond_bool = match val {
                Value::Bool(b) => b,
                _ => {
                    self.loop_depth -= 1;
                    return Err(LucaError::new(format!("Condition must be bool; found {}", val.display()), line, column));
                }
            };
            if !cond_bool {
                break;
            }
            self.push_scope();
            let res = self.execute_block_statements(&body.statements);
            self.pop_scope();
            match res {
                Ok(ExecSignal::Continue) => {},
                Ok(ExecSignal::Return(v, l, c)) => {
                    self.loop_depth -= 1;
                    return Ok(ExecSignal::Return(v, l, c));
                }
                Ok(ExecSignal::Stop(_, _)) => break,
                Ok(ExecSignal::Skip(_, _)) => continue,
                Err(e) => {
                    self.loop_depth -= 1;
                    return Err(e);
                }
            }
        }
        self.loop_depth -= 1;
        Ok(ExecSignal::Continue)
    }

    fn execute_for_each(&mut self, item: &str, collection_expr: &Expr, body: &Block, line: usize, column: usize) -> Result<ExecSignal, LucaError> {
        let coll_val = self.evaluate(collection_expr)?;
        if coll_val.is_none() {
            return Err(LucaError::new("Cannot iterate 'none'", line, column));
        }
        // Collect items to iterate
        enum Items {
            List(Vec<Value>),
            Keys(Vec<String>),
        }
        let items = match coll_val {
            Value::List(v) => Items::List(v.borrow().clone()),
            Value::Tuple(v) => Items::List(v.borrow().clone()),
            Value::Set(v) => Items::List(v.borrow().clone()),
            Value::Dict(entries) => Items::Keys(entries.borrow().iter().map(|e| e.key.clone()).collect()),
            _ => {
                return Err(LucaError::new(
                    format!("for ... from requires a collection; found {}", type_name_for_value(&coll_val)),
                    line,
                    column,
                ));
            }
        };
        // Track iterated collection if it's a simple variable for mutation check
        let coll_var_name: Option<String> = match collection_expr {
            Expr::Variable { name, .. } => Some(name.clone()),
            _ => None,
        };
        if let Some(ref n) = coll_var_name {
            self.iterating.push(n.clone());
        }
        self.loop_depth += 1;
        let result = (|| -> Result<ExecSignal, LucaError> {
            match items {
                Items::List(vals) => {
                    for v in vals {
                        self.push_scope();
                        // Insert item as Uni/permissive to allow hetero
                        let binding_type = if v.is_collection() {
                            let kind = v.collection_kind().unwrap();
                            BindingType::Collection(kind, None)
                        } else {
                            BindingType::Scalar(Type::Uni)
                        };
                        self.current_scope_mut().insert(
                            item.to_owned(),
                            Binding { value: v, binding_type, mutable: true },
                        );
                        let res = self.execute_block_statements(&body.statements);
                        self.pop_scope();
                        match res {
                            Ok(ExecSignal::Continue) => {},
                            Ok(ExecSignal::Return(v, l, c)) => return Ok(ExecSignal::Return(v, l, c)),
                            Ok(ExecSignal::Stop(_, _)) => break,
                            Ok(ExecSignal::Skip(_, _)) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(ExecSignal::Continue)
                }
                Items::Keys(keys) => {
                    for k in keys {
                        self.push_scope();
                        self.current_scope_mut().insert(
                            item.to_owned(),
                            Binding { value: Value::Str(k), binding_type: BindingType::Scalar(Type::Str), mutable: true },
                        );
                        let res = self.execute_block_statements(&body.statements);
                        self.pop_scope();
                        match res {
                            Ok(ExecSignal::Continue) => {},
                            Ok(ExecSignal::Return(v, l, c)) => return Ok(ExecSignal::Return(v, l, c)),
                            Ok(ExecSignal::Stop(_, _)) => break,
                            Ok(ExecSignal::Skip(_, _)) => continue,
                            Err(e) => return Err(e),
                        }
                    }
                    Ok(ExecSignal::Continue)
                }
            }
        })();
        self.loop_depth -= 1;
        if coll_var_name.is_some() {
            self.iterating.pop();
        }
        result
    }

    fn execute_change_index(&mut self, name: &str, index_expr: &Expr, value_expr: &Expr, line: usize, column: usize) -> Result<(), LucaError> {
        self.check_not_iterating(name, line, column)?;
        let (kind, elem) = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            if !b.mutable {
                return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
            }
            match &b.binding_type {
                BindingType::Scalar(_) => {
                    return Err(LucaError::new(format!("'{name}' is not a collection"), line, column));
                }
                BindingType::Collection(k, e) => (*k, *e),
            }
        };
        let index_val = self.evaluate(index_expr)?;
        let new_val = self.evaluate(value_expr)?;
        match kind {
            CollectionKind::List | CollectionKind::Tuple => {
                if index_val.is_none() {
                    return Err(LucaError::new("Index must be int; found none", line, column));
                }
                let idx = match index_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(LucaError::new(
                            format!("Index must be int; found {}", type_name_for_value(&index_val)),
                            line,
                            column,
                        ));
                    }
                };
                if idx < 1 {
                    return Err(LucaError::new("Index out of range", line, column));
                }
                let len = {
                    let binding = self.find_binding(name).unwrap();
                    match &binding.value {
                        Value::List(items) | Value::Tuple(items) => items.borrow().len(),
                        _ => unreachable!(),
                    }
                };
                let pos = (idx - 1) as usize;
                if pos >= len {
                    return Err(LucaError::new("Index out of range", line, column));
                }
                self.check_element_matches(&new_val, elem, line, column)?;
                let binding = self.find_binding_mut(name).unwrap();
                match &mut binding.value {
                    Value::List(items) | Value::Tuple(items) => {
                        items.borrow_mut()[pos] = new_val;
                    }
                    _ => unreachable!(),
                }
                Ok(())
            }
            CollectionKind::Set => Err(LucaError::new("Cannot change set item by index; sets are unordered", line, column)),
            CollectionKind::Dict => {
                if index_val.is_none() {
                    return Err(LucaError::new("Dictionary key must be str; found none", line, column));
                }
                let key = match index_val {
                    Value::Str(s) => s,
                    _ => {
                        return Err(LucaError::new(
                            format!("Dictionary key must be str; found {}", type_name_for_value(&index_val)),
                            line,
                            column,
                        ));
                    }
                };
                // Find position and old type info first (immutable), then update mutably
                let pos = {
                    let b = self.find_binding(name).unwrap();
                    match &b.value {
                        Value::Dict(entries) => entries.borrow().iter().position(|e| e.key == key),
                        _ => unreachable!(),
                    }
                };
                match pos {
                    Some(i) => {
                        // Get old value info for type preservation check
                        enum OldInfo {
                            Collection(CollectionKind),
                            Scalar(Type),
                            None,
                        }
                        let old_info = {
                            let b = self.find_binding(name).unwrap();
                            match &b.value {
                                Value::Dict(entries) => {
                                    let entries = entries.borrow();
                                    let old = &entries[i].value;
                                    if old.is_none() {
                                        OldInfo::None
                                    } else if old.is_collection() {
                                        OldInfo::Collection(old.collection_kind().unwrap())
                                    } else {
                                        OldInfo::Scalar(old.value_type())
                                    }
                                }
                                _ => unreachable!(),
                            }
                        };
                        match old_info {
                            OldInfo::None => {},
                            OldInfo::Collection(old_kind) => {
                                let new_kind = new_val.collection_kind();
                                if !new_val.is_none() && new_kind != Some(old_kind) {
                                    return Err(LucaError::new(
                                        format!(
                                            "Cannot change type of '{key}'; expected {}, found {}",
                                            old_kind,
                                            type_name_for_value(&new_val)
                                        ),
                                        line,
                                        column,
                                    ));
                                }
                            }
                            OldInfo::Scalar(t) => {
                                if !new_val.is_none() {
                                    self.require_type(&new_val, t, line, column)?;
                                }
                            }
                        }
                        let binding = self.find_binding_mut(name).unwrap();
                        match &mut binding.value {
                            Value::Dict(entries) => {
                                entries.borrow_mut()[i].value = new_val;
                            }
                            _ => unreachable!(),
                        }
                        Ok(())
                    }
                    None => Err(LucaError::new(
                        format!("Key '{key}' does not exist; use .add to add new entries"),
                        line,
                        column,
                    )),
                }
            }
        }
    }

    fn execute_collection_add(&mut self, name: &str, args: &[Expr], line: usize, column: usize) -> Result<(), LucaError> {
        self.check_not_iterating(name, line, column)?;
        let (kind, elem) = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            if !b.mutable {
                return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
            }
            match &b.binding_type {
                BindingType::Scalar(_) => {
                    return Err(LucaError::new(format!("'{name}' is not a collection"), line, column));
                }
                BindingType::Collection(k, e) => (*k, *e),
            }
        };
        match kind {
            CollectionKind::List | CollectionKind::Tuple | CollectionKind::Set => {
                if args.len() != 1 {
                    return Err(LucaError::new(format!("Expected 1 argument, found {}", args.len()), line, column));
                }
                let val = self.evaluate(&args[0])?;
                self.check_element_matches(&val, elem, line, column)?;
                if kind == CollectionKind::Set {
                    // Check duplicate
                    let exists = {
                        let b = self.find_binding(name).unwrap();
                        match &b.value {
                            Value::Set(items) => items.borrow().iter().any(|x| values_equal(&val, x) == Some(true)),
                            _ => false,
                        }
                    };
                    if exists {
                        self.warn("Set value already exists", line, column);
                        return Ok(());
                    }
                }
                let binding = self.find_binding_mut(name).unwrap();
                match &mut binding.value {
                    Value::List(items) | Value::Tuple(items) | Value::Set(items) => items.borrow_mut().push(val),
                    _ => unreachable!(),
                }
                Ok(())
            }
            CollectionKind::Dict => {
                if args.len() != 2 {
                    return Err(LucaError::new(format!("Expected 2 arguments, found {}", args.len()), line, column));
                }
                let key_val = self.evaluate(&args[0])?;
                let val = self.evaluate(&args[1])?;
                if key_val.is_none() {
                    return Err(LucaError::new("Dictionary key must be str; found none", line, column));
                }
                let key = match key_val {
                    Value::Str(s) => s,
                    _ => {
                        return Err(LucaError::new(
                            format!("Dictionary key must be str; found {}", type_name_for_value(&key_val)),
                            line,
                            column,
                        ));
                    }
                };
                let exists = {
                    let b = self.find_binding(name).unwrap();
                    match &b.value {
                        Value::Dict(entries) => entries.borrow().iter().any(|e| e.key == key),
                        _ => false,
                    }
                };
                if exists {
                    self.warn(format!("Duplicate dictionary key '{key}'").as_str(), line, column);
                    return Ok(());
                }
                let binding = self.find_binding_mut(name).unwrap();
                match &mut binding.value {
                    Value::Dict(entries) => entries.borrow_mut().push(DictEntry { key, value: val }),
                    _ => unreachable!(),
                }
                Ok(())
            }
        }
    }

    fn execute_collection_remove(&mut self, name: &str, arg: &Expr, line: usize, column: usize) -> Result<(), LucaError> {
        self.check_not_iterating(name, line, column)?;
        let kind = {
            let b = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            match &b.binding_type {
                BindingType::Scalar(_) => {
                    return Err(LucaError::new(format!("'{name}' is not a collection"), line, column));
                }
                BindingType::Collection(k, _) => *k,
            }
        };
        // Check mutable
        {
            let b = self.find_binding(name).unwrap();
            if !b.mutable {
                return Err(LucaError::new(format!("Cannot change constant '{name}'"), line, column));
            }
        }
        let arg_val = self.evaluate(arg)?;
        match kind {
            CollectionKind::List | CollectionKind::Tuple | CollectionKind::Set => {
                let pos = {
                    let binding = self.find_binding(name).unwrap();
                    let items = match &binding.value {
                        Value::List(items) | Value::Tuple(items) | Value::Set(items) => items,
                        _ => unreachable!(),
                    };
                    let items = items.borrow();
                    let mut pos: Option<usize> = None;
                    for (i, x) in items.iter().enumerate() {
                        if values_equal(x, &arg_val) == Some(true) {
                            pos = Some(i);
                            break;
                        }
                    }
                    pos
                };
                match pos {
                    Some(i) => {
                        let binding = self.find_binding_mut(name).unwrap();
                        match &mut binding.value {
                            Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                                items.borrow_mut().remove(i);
                            }
                            _ => unreachable!(),
                        }
                        Ok(())
                    }
                    None => {
                        self.warn("Value not found; no change", line, column);
                        Ok(())
                    }
                }
            }
            CollectionKind::Dict => {
                if arg_val.is_none() {
                    return Err(LucaError::new("Dictionary key must be str; found none", line, column));
                }
                let key = match arg_val {
                    Value::Str(s) => s,
                    _ => {
                        return Err(LucaError::new(
                            format!("Dictionary key must be str; found {}", type_name_for_value(&arg_val)),
                            line,
                            column,
                        ));
                    }
                };
                let pos = {
                    let binding = self.find_binding(name).unwrap();
                    match &binding.value {
                        Value::Dict(entries) => entries.borrow().iter().position(|e| e.key == key),
                        _ => unreachable!(),
                    }
                };
                match pos {
                    Some(i) => {
                        let binding = self.find_binding_mut(name).unwrap();
                        match &mut binding.value {
                            Value::Dict(entries) => {
                                entries.borrow_mut().remove(i);
                            }
                            _ => unreachable!(),
                        }
                        Ok(())
                    }
                    None => {
                        self.warn(format!("Key '{key}' not found; no change").as_str(), line, column);
                        Ok(())
                    }
                }
            }
        }
    }
    fn evaluate(&mut self, expr: &Expr) -> Result<Value, LucaError> {
        match expr {
            Expr::Literal(value) => Ok(value.clone()),
            Expr::StringTemplate { value: template, line, column } => {
                Ok(Value::Str(self.interpolate(template, *line, *column)?))
            }
            Expr::Variable { name, asserted_type, line, column } => {
                let binding = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), *line, *column))?;
                match &binding.binding_type {
                    BindingType::Scalar(t) => {
                        if asserted_type.is_some_and(|ty| ty != *t) {
                            return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), *line, *column));
                        }
                    }
                    BindingType::Collection(_, elem) => {
                        if let Some(a) = asserted_type {
                            match elem {
                                Some(e) if a != e => {
                                    return Err(LucaError::new(format!("Type annotation for '{name}' does not match its declaration"), *line, *column));
                                }
                                _ => {}
                            }
                        }
                    }
                }
                // Collections are reference values (shared storage), except reads
                // through a `const` binding return a deep copy so constant
                // contents can never change through an alias.
                if binding.value.is_collection() && !binding.mutable {
                    Ok(binding.value.deep_copy_collection())
                } else {
                    Ok(binding.value.clone())
                }
            }
            Expr::Group(inner) => self.evaluate(inner),
            Expr::Unary { op: UnaryOp::Negate, right, line, column } => {
                let val = self.evaluate(right)?;
                if val.is_none() {
                    return Err(LucaError::new("Cannot use 'none' in arithmetic", *line, *column));
                }
                match val {
                    Value::Int(value) => value.checked_neg().map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", *line, *column)),
                    Value::Dec(value) => value.neg().map(Value::Dec).ok_or_else(|| LucaError::new("Decimal result is out of range", *line, *column)),
                    _ => Err(LucaError::new("Unary '-' requires an int or dec", *line, *column)),
                }
            }
            Expr::Unary { op: UnaryOp::Not, right, line, column } => {
                let val = self.evaluate(right)?;
                if val.is_none() {
                    return Err(LucaError::new("Cannot use 'none' in logical operation", *line, *column));
                }
                match val {
                    Value::Bool(b) => Ok(Value::Bool(!b)),
                    _ => Err(LucaError::new("'not' requires a bool", *line, *column)),
                }
            }
            Expr::Binary { left, op, right, line, column } => {
                match op {
                    BinaryOp::And => {
                        let left_val = self.evaluate(left)?;
                        if left_val.is_none() {
                            return Err(LucaError::new("Cannot use 'none' in logical operation", *line, *column));
                        }
                        let left_bool = match left_val {
                            Value::Bool(b) => b,
                            _ => return Err(LucaError::new("'and' requires bool operands", *line, *column)),
                        };
                        if !left_bool {
                            return Ok(Value::Bool(false));
                        }
                        let right_val = self.evaluate(right)?;
                        if right_val.is_none() {
                            return Err(LucaError::new("Cannot use 'none' in logical operation", *line, *column));
                        }
                        match right_val {
                            Value::Bool(b) => Ok(Value::Bool(b)),
                            _ => Err(LucaError::new("'and' requires bool operands", *line, *column)),
                        }
                    }
                    BinaryOp::Or => {
                        let left_val = self.evaluate(left)?;
                        if left_val.is_none() {
                            return Err(LucaError::new("Cannot use 'none' in logical operation", *line, *column));
                        }
                        let left_bool = match left_val {
                            Value::Bool(b) => b,
                            _ => return Err(LucaError::new("'or' requires bool operands", *line, *column)),
                        };
                        if left_bool {
                            return Ok(Value::Bool(true));
                        }
                        let right_val = self.evaluate(right)?;
                        if right_val.is_none() {
                            return Err(LucaError::new("Cannot use 'none' in logical operation", *line, *column));
                        }
                        match right_val {
                            Value::Bool(b) => Ok(Value::Bool(b)),
                            _ => Err(LucaError::new("'or' requires bool operands", *line, *column)),
                        }
                    }
                    _ => {
                        let left_val = self.evaluate(left)?;
                        let right_val = self.evaluate(right)?;
                        self.binary(left_val, *op, right_val, *line, *column)
                    }
                }
            }
            Expr::Comparison { left, op, right, line, column } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                self.compare(left_val, *op, right_val, *line, *column).map(Value::Bool)
            }
            Expr::Call { name, args, line, column } => self.evaluate_call(name, args, *line, *column),
            Expr::Ask { prompt, line, column } => {
                let prompt_val = self.evaluate(prompt)?;
                match prompt_val {
                    Value::Str(text) => {
                        // The prompt is shown as its own output line (documented).
                        self.output.push(text.clone());
                        let input = self.read_input_line(*line, *column)?;
                        Ok(Value::Str(input))
                    }
                    _ => Err(LucaError::new(
                        format!("Expected str, found {}", type_name_for_value(&prompt_val)),
                        *line,
                        *column,
                    )),
                }
            }
            Expr::Member { base, member, line, column } => self.evaluate_member(base, member, *line, *column),
            Expr::MemberCall { base, member, args, line, column } => {
                self.evaluate_member_call(base, member, args, *line, *column)
            }
            Expr::List { items, line: _, column: _ } => self.eval_list(items),
            Expr::Tuple { items, line: _, column: _ } => self.eval_tuple(items),
            Expr::Set { items, line, column } => self.eval_set(items, *line, *column),
            Expr::Dict { entries, line: _, column: _ } => self.eval_dict(entries),
            Expr::Index { base, index, line, column } => self.evaluate_index(base, index, *line, *column),
            Expr::Contains { object, mode, value, line, column } => self.evaluate_contains(object, *mode, value, *line, *column),
            Expr::Length { object, arg, line, column } => self.evaluate_length(object, *arg, *line, *column),
        }
    }

    // Literal builders below keep `evaluate` small so deep call chains stay
    // within default thread stacks even in unoptimized builds.
    fn eval_list(&mut self, items: &[Expr]) -> Result<Value, LucaError> {
        let mut vals = Vec::with_capacity(items.len());
        for item in items {
            vals.push(self.evaluate(item)?);
        }
        Ok(Value::new_list(vals))
    }

    fn eval_tuple(&mut self, items: &[Expr]) -> Result<Value, LucaError> {
        let mut vals = Vec::with_capacity(items.len());
        for item in items {
            vals.push(self.evaluate(item)?);
        }
        Ok(Value::new_tuple(vals))
    }

    fn eval_set(&mut self, items: &[Expr], line: usize, column: usize) -> Result<Value, LucaError> {
        let mut vals = Vec::with_capacity(items.len());
        for item in items {
            vals.push(self.evaluate(item)?);
        }
        // Deduplicate with warning (sets are unique)
        let mut unique: Vec<Value> = Vec::new();
        for v in vals {
            let mut dup = false;
            for u in &unique {
                if values_equal(&v, u) == Some(true) {
                    dup = true;
                    break;
                }
            }
            if dup {
                        self.warn("Duplicate set value", line, column);
                    } else {
                        unique.push(v);
                    }
                }
                Ok(Value::new_set(unique))
    }

    fn eval_dict(&mut self, entries: &[crate::ast::DictEntryExpr]) -> Result<Value, LucaError> {
        let mut out: Vec<DictEntry> = Vec::with_capacity(entries.len());
        for entry in entries {
            let val = self.evaluate(&entry.value)?;
            if let Some(t) = entry.value_type {
                self.require_type(&val, t, entry.line, entry.column)?;
            }
            if out.iter().any(|e| e.key == entry.key) {
                self.warn(format!("Duplicate dictionary key '{}'", entry.key).as_str(), entry.line, entry.column);
            } else {
                out.push(DictEntry { key: entry.key.clone(), value: val });
            }
        }
        Ok(Value::new_dict(out))
    }

    fn evaluate_index(&mut self, base: &Expr, index: &Expr, line: usize, column: usize) -> Result<Value, LucaError> {
        let base_val = self.evaluate(base)?;
        let index_val = self.evaluate(index)?;
        if base_val.is_none() {
            return Err(LucaError::new("Cannot index 'none'", line, column));
        }
        match base_val {
            Value::List(items) | Value::Tuple(items) => {
                if index_val.is_none() {
                    return Err(LucaError::new("Index must be int; found none", line, column));
                }
                let idx = match index_val {
                    Value::Int(n) => n,
                    _ => {
                        return Err(LucaError::new(
                            format!("Index must be int; found {}", type_name_for_value(&index_val)),
                            line,
                            column,
                        ));
                    }
                };
                let items = items.borrow();
                if idx < 1 || (idx as usize) > items.len() {
                    self.warn("Index out of range", line, column);
                    return Ok(Value::None);
                }
                Ok(items[(idx - 1) as usize].clone())
            }
            Value::Dict(entries) => {
                if index_val.is_none() {
                    return Err(LucaError::new("Dictionary key must be str; found none", line, column));
                }
                let key = match index_val {
                    Value::Str(s) => s,
                    _ => {
                        return Err(LucaError::new(
                            format!("Dictionary key must be str; found {}", type_name_for_value(&index_val)),
                            line,
                            column,
                        ));
                    }
                };
                let entries = entries.borrow();
                match entries.iter().find(|e| e.key == key) {
                    Some(e) => Ok(e.value.clone()),
                    None => {
                        self.warn(format!("Key '{key}' not found").as_str(), line, column);
                        Ok(Value::None)
                    }
                }
            }
            Value::Set(_) => Err(LucaError::new("Cannot index set; sets are unordered", line, column)),
            _ => Err(LucaError::new(
                format!("Cannot index {}", type_name_for_value(&base_val)),
                line,
                column,
            )),
        }
    }

    fn evaluate_contains(&mut self, object: &Expr, mode: ContainsMode, value: &Expr, line: usize, column: usize) -> Result<Value, LucaError> {
        use ContainsMode as M;
        let obj = self.evaluate(object)?;
        if obj.is_none() {
            return Err(LucaError::new("Cannot use 'none' in contains", line, column));
        }
        match mode {
            M::Single => {
                let val = self.evaluate(value)?;
                match obj {
                    Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                        for item in items.borrow().iter() {
                            if values_equal(item, &val) == Some(true) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    Value::Dict(_) => Err(LucaError::new(
                        "Use contains(key = ...) or contains(value = ...) for dictionaries",
                        line,
                        column,
                    )),
                    _ => Err(LucaError::new(
                        format!("contains not supported for {}", type_name_for_value(&obj)),
                        line,
                        column,
                    )),
                }
            }
            M::Key => {
                let val = self.evaluate(value)?;
                if val.is_none() {
                    return Err(LucaError::new("Dictionary key must be str; found none", line, column));
                }
                let key = match val {
                    Value::Str(s) => s,
                    _ => {
                        return Err(LucaError::new(
                            format!("Dictionary key must be str; found {}", type_name_for_value(&val)),
                            line,
                            column,
                        ));
                    }
                };
                match obj {
                    Value::Dict(entries) => Ok(Value::Bool(entries.borrow().iter().any(|e| e.key == key))),
                    _ => Err(LucaError::new(
                        format!("contains(key = ...) only for dictionaries; found {}", type_name_for_value(&obj)),
                        line,
                        column,
                    )),
                }
            }
            M::Value => {
                let val = self.evaluate(value)?;
                match obj {
                    Value::Dict(entries) => {
                        for e in entries.borrow().iter() {
                            if values_equal(&e.value, &val) == Some(true) {
                                return Ok(Value::Bool(true));
                            }
                        }
                        Ok(Value::Bool(false))
                    }
                    _ => Err(LucaError::new(
                        format!("contains(value = ...) only for dictionaries; found {}", type_name_for_value(&obj)),
                        line,
                        column,
                    )),
                }
            }
        }
    }

    fn evaluate_length(&mut self, object: &Expr, arg: Option<LengthArg>, line: usize, column: usize) -> Result<Value, LucaError> {
        let obj = self.evaluate(object)?;
        match arg {
            None => match obj {
                Value::List(items) | Value::Tuple(items) | Value::Set(items) => Ok(Value::Int(items.borrow().len() as i64)),
                Value::Dict(entries) => Ok(Value::Int(entries.borrow().len() as i64)),
                Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
                Value::Int(n) => {
                    let digits = if n == 0 {
                        1
                    } else {
                        n.abs().to_string().len() as i64
                    };
                    Ok(Value::Int(digits))
                }
                Value::Dec(d) => {
                    // Digit count excluding decimal point and minus sign
                    let s = d.display();
                    let count = s.chars().filter(|c| c.is_ascii_digit()).count() as i64;
                    Ok(Value::Int(count))
                }
                Value::Bool(_) => Err(LucaError::new("length not supported for bool", line, column)),
                Value::None => Err(LucaError::new("Cannot use 'none' in length", line, column)),
            },
            Some(LengthArg::Decimals) => match obj {
                Value::Dec(d) => Ok(Value::Int(d.scale as i64)),
                Value::None => Err(LucaError::new("Cannot use 'none' in length", line, column)),
                _ => Err(LucaError::new(
                    format!("length(decimals) only for dec; found {}", type_name_for_value(&obj)),
                    line,
                    column,
                )),
            },
            Some(LengthArg::Items) => match obj {
                Value::List(items) | Value::Tuple(items) | Value::Set(items) => Ok(Value::Int(items.borrow().len() as i64)),
                Value::Dict(entries) => Ok(Value::Int(entries.borrow().len() as i64)),
                Value::None => Err(LucaError::new("Cannot use 'none' in length", line, column)),
                _ => Err(LucaError::new(
                    format!("length(items) only for collections; found {}", type_name_for_value(&obj)),
                    line,
                    column,
                )),
            },
        }
    }

    fn evaluate_call(&mut self, name: &str, args: &[Expr], line: usize, column: usize) -> Result<Value, LucaError> {
        // `find_function` returns an owned `Rc`, so no borrow of `self` is held.
        // (Previously this deep-cloned the whole `Function` per call, which
        // wasted stack on every recursion level; the `Rc` is sufficient.)
        let func = self.find_function(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
        // Check recursion limit
        if self.call_depth >= self.recursion_limit {
            return Err(LucaError::new("Recursion limit exceeded", line, column));
        }
        // Check argument count
        let params = &func.params;
        if args.len() > params.len() {
            return Err(LucaError::new(format!("Expected {} arguments, found {}", params.len(), args.len()), line, column));
        }
        // Evaluate provided args
        let mut evaluated_args: Vec<Value> = Vec::new();
        for arg in args {
            evaluated_args.push(self.evaluate(arg)?);
        }
        // Fill missing args with defaults
        let mut final_args: Vec<Value> = Vec::new();
        for (i, param) in params.iter().enumerate() {
            if i < evaluated_args.len() {
                let val = evaluated_args[i].clone();
                let val = self.apply_ask_conversion(&args[i], val, param.ty)?;
                if let Some(ty) = param.ty {
                    self.require_type(&val, ty, param.line, param.column)?;
                }
                final_args.push(val);
            } else if let Some(default_expr) = &param.default {
                let val = self.evaluate(default_expr)?;
                let val = self.apply_ask_conversion(default_expr, val, param.ty)?;
                if let Some(ty) = param.ty {
                    self.require_type(&val, ty, param.line, param.column)?;
                }
                final_args.push(val);
            } else {
                return Err(LucaError::new(format!("Missing argument for '{}'", param.name), line, column));
            }
        }
        // Type check provided args against param types (already done for each)
        // Now call the function
        self.call_depth += 1;
        // Save current scopes length to restore after (for closure we could use captured, but for now use current)
        // For closure support, we would use func.closure_scopes, but for 0.3 we use current scopes plus new frame
        // Push new scope for function locals
        self.push_scope();
        if let Err(e) = self.bind_call_params(params, final_args) {
            self.pop_scope();
            self.call_depth -= 1;
            return Err(e);
        }
        // Also need to make function itself available for recursion: insert it into current scope?
        // The function name should be visible inside its own body for recursion
        // It is already in func_scopes, but we need to ensure that the function is visible inside its own body
        // Since we are already inside a call, the function is in the outer scope's func_scopes, which is still visible via find_function
        // No need to insert again
        let result = self.execute_function_body(&func);
        self.pop_scope();
        self.call_depth -= 1;
        result
    }

    // Shared parameter binder for local and module calls. Inserts bindings
    // into the already-pushed current scope; on error the caller pops.
    fn bind_call_params(&mut self, params: &[Param], final_args: Vec<Value>) -> Result<(), LucaError> {
        for (param, val) in params.iter().zip(final_args) {
            if val.is_collection() {
                if let Some(t) = param.ty {
                    // Param has explicit scalar type but arg is collection
                    return Err(LucaError::new(
                        format!("Expected {t}, found {}", type_name_for_value(&val)),
                        param.line,
                        param.column,
                    ));
                }
                let kind = val.collection_kind().unwrap();
                let elem = match &val {
                    Value::List(items) | Value::Tuple(items) | Value::Set(items) => {
                        infer_element_type(&items.borrow())
                    }
                    Value::Dict(_) => None,
                    _ => None,
                };
                self.current_scope_mut().insert(
                    param.name.clone(),
                    Binding { value: val, binding_type: BindingType::Collection(kind, elem), mutable: true },
                );
            } else {
                let decl_ty = if let Some(t) = param.ty {
                    t
                } else {
                    if val.is_none() {
                        return Err(LucaError::new(format!("Cannot infer type for parameter '{}' from 'none'", param.name), param.line, param.column));
                    }
                    val.value_type()
                };
                self.require_type(&val, decl_ty, param.line, param.column)?;
                self.current_scope_mut().insert(
                    param.name.clone(),
                    Binding { value: val, binding_type: BindingType::Scalar(decl_ty), mutable: true },
                );
            }
        }
        Ok(())
    }

    fn execute_function_body(&mut self, func: &Function) -> Result<Value, LucaError> {
        // Execute body statements, handling return
        match self.execute_block_statements(&func.body.statements) {
            Ok(ExecSignal::Continue) => {
                // Reached end without return, return none
                let ret = Value::None;
                if let Some(ty) = func.return_type {
                    self.require_type(&ret, ty, func.line, func.column)?;
                }
                Ok(ret)
            }
            Ok(ExecSignal::Return(val, line, column)) => {
                if let Some(ty) = func.return_type {
                    self.require_type(&val, ty, line, column)?;
                }
                Ok(val)
            }
            Ok(ExecSignal::Stop(line, column)) => Err(LucaError::new("stop outside loop", line, column)),
            Ok(ExecSignal::Skip(line, column)) => Err(LucaError::new("skip outside loop", line, column)),
            Err(e) => Err(e),
        }
    }
    fn binary(&self, left: Value, op: BinaryOp, right: Value, line: usize, column: usize) -> Result<Value, LucaError> {
        if left.is_none() || right.is_none() {
            return Err(LucaError::new("Cannot use 'none' in arithmetic", line, column));
        }
        if matches!(op, BinaryOp::Add) {
            if let (Value::Str(a), Value::Str(b)) = (&left, &right) { return Ok(Value::Str(format!("{a}{b}"))); }
        }
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => {
                match op {
                    BinaryOp::Add => a.checked_add(b).map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", line, column)),
                    BinaryOp::Subtract => a.checked_sub(b).map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", line, column)),
                    BinaryOp::Multiply => a.checked_mul(b).map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", line, column)),
                    BinaryOp::Divide => {
                        if b == 0 {
                            return Err(LucaError::new("Division by zero", line, column));
                        }
                        let da = Dec::from_int(a);
                        let db = Dec::from_int(b);
                        da.div(&db).map(Value::Dec).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))
                    }
                    BinaryOp::Modulo => {
                        if b == 0 {
                            return Err(LucaError::new("Modulo by zero", line, column));
                        }
                        a.checked_rem(b).map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", line, column))
                    }
                    BinaryOp::And | BinaryOp::Or => unreachable!("logical operators handled in evaluate"),
                }
            }
            (a, b) if matches!(&a, Value::Int(_) | Value::Dec(_)) && matches!(&b, Value::Int(_) | Value::Dec(_)) => {
                let da = match a {
                    Value::Int(v) => Dec::from_int(v),
                    Value::Dec(v) => v,
                    _ => unreachable!(),
                };
                let db = match b {
                    Value::Int(v) => Dec::from_int(v),
                    Value::Dec(v) => v,
                    _ => unreachable!(),
                };
                let result = match op {
                    BinaryOp::Add => da.add(&db),
                    BinaryOp::Subtract => da.sub(&db),
                    BinaryOp::Multiply => da.mul(&db),
                    BinaryOp::Divide => {
                        if db.is_zero() {
                            return Err(LucaError::new("Division by zero", line, column));
                        }
                        da.div(&db)
                    }
                    BinaryOp::Modulo => {
                        if db.is_zero() {
                            return Err(LucaError::new("Modulo by zero", line, column));
                        }
                        da.rem(&db)
                    }
                    BinaryOp::And | BinaryOp::Or => unreachable!("logical operators handled in evaluate"),
                }
                .ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
                Ok(Value::Dec(result))
            }
            _ => Err(LucaError::new("Arithmetic requires matching numeric types; '+' also accepts two str values", line, column)),
        }
    }
    fn compare(&self, left: Value, op: CompareOp, right: Value, line: usize, column: usize) -> Result<bool, LucaError> {
        // Handle none for equality: none == none true, none != value false etc.
        let left_is_none = left.is_none();
        let right_is_none = right.is_none();
        if left_is_none || right_is_none {
            match op {
                CompareOp::Equal => return Ok(left_is_none && right_is_none),
                CompareOp::NotEqual => return Ok(left_is_none != right_is_none),
                _ => return Err(LucaError::new("Cannot order 'none'", line, column)),
            }
        }
        match op {
            CompareOp::Equal | CompareOp::NotEqual => {
                // Collections use content-based equality (recursive)
                if left.is_collection() || right.is_collection() {
                    match values_equal(&left, &right) {
                        Some(equal) => {
                            if op == CompareOp::Equal {
                                return Ok(equal);
                            } else {
                                return Ok(!equal);
                            }
                        }
                        None => {
                            return Err(LucaError::new("Incompatible types for comparison", line, column));
                        }
                    }
                }
                let equal = match (&left, &right) {
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Dec(a), Value::Dec(b)) => a.cmp_numeric(b).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))? == std::cmp::Ordering::Equal,
                    (Value::Int(a), Value::Dec(b)) => Dec::from_int(*a).cmp_numeric(b).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))? == std::cmp::Ordering::Equal,
                    (Value::Dec(a), Value::Int(b)) => a.cmp_numeric(&Dec::from_int(*b)).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))? == std::cmp::Ordering::Equal,
                    (Value::Str(a), Value::Str(b)) => a == b,
                    (Value::Bool(a), Value::Bool(b)) => a == b,
                    _ => return Err(LucaError::new("Incompatible types for comparison", line, column)),
                };
                if op == CompareOp::Equal { Ok(equal) } else { Ok(!equal) }
            }
            CompareOp::Greater | CompareOp::GreaterEqual | CompareOp::Less | CompareOp::LessEqual => {
                let ordering = match (&left, &right) {
                    (Value::Int(a), Value::Int(b)) => a.cmp(b),
                    (Value::Dec(a), Value::Dec(b)) => a.cmp_numeric(b).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))?,
                    (Value::Int(a), Value::Dec(b)) => Dec::from_int(*a).cmp_numeric(b).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))?,
                    (Value::Dec(a), Value::Int(b)) => a.cmp_numeric(&Dec::from_int(*b)).ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))?,
                    (Value::Str(a), Value::Str(b)) => a.cmp(b),
                    _ => return Err(LucaError::new("Cannot order given types", line, column)),
                };
                let result = match op {
                    CompareOp::Greater => ordering == std::cmp::Ordering::Greater,
                    CompareOp::GreaterEqual => ordering == std::cmp::Ordering::Greater || ordering == std::cmp::Ordering::Equal,
                    CompareOp::Less => ordering == std::cmp::Ordering::Less,
                    CompareOp::LessEqual => ordering == std::cmp::Ordering::Less || ordering == std::cmp::Ordering::Equal,
                    _ => unreachable!(),
                };
                Ok(result)
            }
        }
    }

    fn interpolate(&self, template: &str, line: usize, column: usize) -> Result<String, LucaError> {
        let mut output = String::new();
        let mut rest = template;
        while let Some(open) = rest.find('{') {
            output.push_str(&rest[..open]);
            let after_open = &rest[open + 1..];
            let close = after_open.find('}').ok_or_else(|| LucaError::new("Unclosed interpolation", line, column))?;
            let name = &after_open[..close];
            if name.is_empty() || !name.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '_') { return Err(LucaError::new("Interpolation supports variable names only in this milestone", line, column)); }
            let binding = self.find_binding(name).ok_or_else(|| LucaError::new(format!("'{name}' is not declared"), line, column))?;
            output.push_str(&binding.value.display());
            rest = &after_open[close + 1..];
        }
        output.push_str(rest);
        Ok(output)
    }
    /// If `source` is a direct `ask` expression and `target` needs converted
    /// input (`int`/`dec`/`bool`), parse the already-read input string.
    /// Otherwise returns the evaluated value unchanged for normal checking.
    /// Collection targets never convert (a string input mismatches naturally).
    fn apply_ask_conversion(&self, source: &Expr, evaluated: Value, target: Option<Type>) -> Result<Value, LucaError> {
        let Expr::Ask { line, column, .. } = source else {
            return Ok(evaluated);
        };
        let Some(t) = target else {
            return Ok(evaluated);
        };
        if !matches!(t, Type::Int | Type::Dec | Type::Bool) {
            return Ok(evaluated);
        }
        let Value::Str(input) = evaluated else {
            return Ok(evaluated);
        };
        convert_input_to_type(&input, t, *line, *column)
    }

    fn require_type(&self, value: &Value, expected: Type, line: usize, column: usize) -> Result<(), LucaError> {
        if value.is_none() {
            return Ok(());
        }
        if expected == Type::Uni {
            return Ok(());
        }
        if value.is_collection() {
            return Err(LucaError::new(
                format!("Expected {expected}, found {}", type_name_for_value(value)),
                line,
                column,
            ));
        }
        if value.value_type() == expected {
            Ok(())
        } else {
            Err(LucaError::new(
                format!("Expected {expected}, found {}", type_name_for_value(value)),
                line,
                column,
            ))
        }
    }
}

fn statement_location(statement: &Statement) -> (usize, usize) {
    match statement {
        Statement::Declare { line, column, .. }
        | Statement::DeclareCollection { line, column, .. }
        | Statement::Assign { line, column, .. }
        | Statement::Print { line, column, .. }
        | Statement::Forget { line, column, .. }
        | Statement::Clear { line, column, .. }
        | Statement::Change { line, column, .. }
        | Statement::ChangeIndex { line, column, .. }
        | Statement::CollectionAdd { line, column, .. }
        | Statement::CollectionRemove { line, column, .. }
        | Statement::If { line, column, .. }
        | Statement::FuncDef { line, column, .. }
        | Statement::Return { line, column, .. }
        | Statement::Call { line, column, .. }
        | Statement::MemberCall { line, column, .. }
        | Statement::Import { line, column, .. }
        | Statement::Try { line, column, .. }
        | Statement::ErrorDef { line, column, .. }
        | Statement::Raise { line, column, .. }
        | Statement::ForTimes { line, column, .. }
        | Statement::ForEach { line, column, .. }
        | Statement::While { line, column, .. }
        | Statement::Stop { line, column }
        | Statement::Skip { line, column } => (*line, *column),
    }
}

fn convert_input_to_type(input: &str, target: Type, line: usize, column: usize) -> Result<Value, LucaError> {
    let trimmed = input.trim();
    let invalid = || LucaError::new(format!("Invalid input '{trimmed}': expected {target}"), line, column);
    match target {
        Type::Int => trimmed.parse::<i64>().map(Value::Int).map_err(|_| invalid()),
        Type::Dec => parse_dec_input(trimmed).map(Value::Dec).map_err(|_| invalid()),
        Type::Bool => match trimmed {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(invalid()),
        },
        _ => Ok(Value::Str(input.to_owned())),
    }
}

fn parse_dec_input(trimmed: &str) -> Result<Dec, String> {
    let (negative, rest) = match trimmed.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    if rest.is_empty() {
        return Err("Invalid decimal input".to_owned());
    }
    let mut dec = if rest.contains('.') {
        Dec::parse_literal(rest)?
    } else {
        if !rest.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid decimal input".to_owned());
        }
        let unscaled: i128 = rest.parse().map_err(|_| "Decimal input is out of range".to_owned())?;
        Dec::new(unscaled, 0)
    };
    if negative {
        dec = dec.neg().ok_or_else(|| "Decimal input is out of range".to_owned())?;
    }
    Ok(dec)
}

pub(crate) fn warn_at(line: usize, column: usize, message: &str) {
    eprintln!("Warning at {line}:{column}: {message}");
}

fn in_module_error(module_name: &str, error: LucaError) -> LucaError {
    // Process termination keeps its identity instead of gaining the prefix.
    // Raised custom errors keep their caught value while still gaining origin
    // context, like any other error crossing into the importer.
    if error.is_exit() {
        return error;
    }
    let mut wrapped = LucaError::new(format!("In module '{module_name}': {}", error.message), error.line, error.column);
    wrapped.error_value = error.error_value;
    wrapped
}

pub(crate) fn type_name_for_value(value: &Value) -> String {
    match value {
        Value::Int(_) => "int".to_owned(),
        Value::Dec(_) => "dec".to_owned(),
        Value::Str(_) => "str".to_owned(),
        Value::Bool(_) => "bool".to_owned(),
        Value::None => "none".to_owned(),
        Value::List(_) => "list".to_owned(),
        Value::Tuple(_) => "tuple".to_owned(),
        Value::Set(_) => "set".to_owned(),
        Value::Dict(_) => "dict".to_owned(),
    }
}

fn infer_element_type(items: &[Value]) -> Option<Type> {
    let mut found: Option<Type> = None;
    for item in items {
        if item.is_none() {
            continue;
        }
        if item.is_collection() {
            // Nested collections infer as uni (heterogeneous allowed)
            // If we already found a scalar type, then hetero => None
            match found {
                None => found = Some(Type::Uni),
                Some(Type::Uni) => {},
                Some(_) => return None,
            }
            continue;
        }
        let t = item.value_type();
        match found {
            None => found = Some(t),
            Some(existing) if existing == t => {},
            Some(_) => return None,
        }
    }
    match found {
        // If all Uni (nested) or empty, return None (permissive)
        Some(Type::Uni) => None,
        other => other,
    }
}

fn convert_empty_set_dict(value: Value, expected: CollectionKind) -> Value {
    match (&value, expected) {
        (Value::Set(items), CollectionKind::Dict) if items.borrow().is_empty() => Value::new_dict(Vec::new()),
        (Value::Dict(items), CollectionKind::Set) if items.borrow().is_empty() => Value::new_set(Vec::new()),
        _ => value,
    }
}
