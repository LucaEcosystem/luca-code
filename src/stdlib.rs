use crate::{
    error::LucaError,
    types::{Dec, DictEntry, Value},
};

pub const BUILTIN_MODULES: [&str; 4] = ["Math", "Text", "File", "System"];

pub fn is_builtin(name: &str) -> bool {
    BUILTIN_MODULES.contains(&name)
}

/// True for `Module.func(...)` members implemented natively.
pub fn is_native_function(module: &str, member: &str) -> bool {
    matches!(
        (module, member),
        ("Math", "sqrt")
            | ("Math", "pow")
            | ("Math", "abs")
            | ("Math", "round")
            | ("Math", "floor")
            | ("Math", "ceil")
            | ("Math", "min")
            | ("Math", "max")
            | ("Text", "upper")
            | ("Text", "lower")
            | ("Text", "trim")
            | ("Text", "replace")
            | ("Text", "split")
            | ("Text", "join")
            | ("File", "read")
            | ("File", "write")
            | ("File", "append")
            | ("File", "exists")
            | ("File", "delete")
            | ("System", "exit")
            | ("System", "args")
            | ("System", "env")
    )
}

/// Bare property reads (`System.os`, `System.arch`). Everything else builtin
/// is a function and must be called.
pub fn builtin_member_value(module: &str, member: &str) -> Option<Value> {
    match (module, member) {
        ("System", "os") => Some(Value::Str(std::env::consts::OS.to_owned())),
        ("System", "arch") => Some(Value::Str(std::env::consts::ARCH.to_owned())),
        _ => None,
    }
}

fn arg_count(args: &[Value], expected: usize, what: &str, line: usize, column: usize) -> Result<(), LucaError> {
    if args.len() != expected {
        return Err(LucaError::new(
            format!("Expected {expected} arguments, found {} for {what}", args.len()),
            line,
            column,
        ));
    }
    Ok(())
}

fn expect_str(value: &Value, what: &str, line: usize, column: usize) -> Result<String, LucaError> {
    match value {
        Value::Str(s) => Ok(s.clone()),
        _ => Err(LucaError::new(
            format!("Expected str, found {} for {what}", crate::interpreter::type_name_for_value(value)),
            line,
            column,
        )),
    }
}

fn expect_number(value: &Value, what: &str, line: usize, column: usize) -> Result<Number, LucaError> {
    match value {
        Value::Int(n) => Ok(Number::Int(*n)),
        Value::Dec(d) => Ok(Number::Dec(d.clone())),
        _ => Err(LucaError::new(
            format!("Expected int or dec, found {} for {what}", crate::interpreter::type_name_for_value(value)),
            line,
            column,
        )),
    }
}

#[derive(Debug, Clone)]
enum Number {
    Int(i64),
    Dec(Dec),
}

impl Number {
    fn as_dec(&self) -> Dec {
        match self {
            Self::Int(n) => Dec::from_int(*n),
            Self::Dec(d) => d.clone(),
        }
    }

    fn is_negative(&self) -> bool {
        match self {
            Self::Int(n) => *n < 0,
            Self::Dec(d) => d.unscaled < 0,
        }
    }
}

/// Integer square root (floor) via Newton's method. No floating point.
fn isqrt(n: i128) -> i128 {
    if n <= 0 {
        return 0;
    }
    let mut x = 1i128 << (128 - n.leading_zeros()).div_ceil(2);
    loop {
        let y = (x + n / x) / 2;
        if y >= x {
            return x;
        }
        x = y;
    }
}

fn math_sqrt(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "Math.sqrt", line, column)?;
    let x = expect_number(&args[0], "Math.sqrt", line, column)?;
    if x.is_negative() {
        return Err(LucaError::new("Math.sqrt of negative number", line, column));
    }
    let dec = x.as_dec();
    // Exact truncated root at up to 10 decimal places, trailing zeros trimmed.
    // Smaller scales are retried if the full-precision product overflows.
    let mut result: Option<Dec> = None;
    for &scale in &[10u32, 8, 6, 4, 2, 0] {
        let twice = 2 * scale;
        let scaled = if dec.scale <= twice {
            Dec::pow10(twice - dec.scale)
                .and_then(|p| dec.unscaled.checked_mul(p))
                .map(isqrt)
        } else {
            Dec::pow10(dec.scale - twice).map(|p| isqrt(dec.unscaled / p))
        };
        if let Some(unscaled) = scaled {
            result = Some(Dec::new(unscaled, scale));
            break;
        }
    }
    let mut result = result.ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
    while result.scale > 0 && result.unscaled % 10 == 0 {
        result.unscaled /= 10;
        result.scale -= 1;
    }
    Ok(Value::Dec(result))
}

fn as_exact_int(value: &Value) -> Option<i64> {
    match value {
        Value::Int(n) => Some(*n),
        Value::Dec(d) => {
            let factor = Dec::pow10(d.scale)?;
            if d.unscaled % factor != 0 {
                return None;
            }
            (d.unscaled / factor).try_into().ok()
        }
        _ => None,
    }
}

fn dec_pow(mut base: Dec, mut exp: i64, line: usize, column: usize) -> Result<Dec, LucaError> {
    if exp < 0 {
        let positive = exp.checked_neg().ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
        let powered = dec_pow(base, positive, line, column)?;
        if powered.is_zero() {
            return Err(LucaError::new("Division by zero", line, column));
        }
        return Dec::new(1, 0).div(&powered).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column));
    }
    let mut result = Dec::new(1, 0);
    while exp > 0 {
        if exp & 1 == 1 {
            result = result.mul(&base).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
        }
        exp >>= 1;
        if exp > 0 {
            base = base.mul(&base).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
        }
    }
    Ok(result)
}

fn math_pow(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 2, "Math.pow", line, column)?;
    let base = expect_number(&args[0], "Math.pow", line, column)?;
    let exp = match &args[1] {
        Value::Int(n) => *n,
        Value::Dec(_) => as_exact_int(&args[1])
            .ok_or_else(|| LucaError::new("Math.pow exponent must be int", line, column))?,
        _ => {
            return Err(LucaError::new(
                format!(
                    "Expected int, found {} for Math.pow exponent",
                    crate::interpreter::type_name_for_value(&args[1])
                ),
                line,
                column,
            ));
        }
    };
    // 0^0 is defined as 1.
    dec_pow(base.as_dec(), exp, line, column).map(Value::Dec)
}

fn math_abs(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "Math.abs", line, column)?;
    match expect_number(&args[0], "Math.abs", line, column)? {
        Number::Int(n) => n.checked_abs().map(Value::Int).ok_or_else(|| LucaError::new("Integer overflow", line, column)),
        Number::Dec(d) => {
            if d.unscaled < 0 {
                d.neg().map(Value::Dec).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))
            } else {
                Ok(Value::Dec(d))
            }
        }
    }
}

fn dec_to_int_half_away(value: &Dec, line: usize, column: usize) -> Result<i64, LucaError> {
    let factor = Dec::pow10(value.scale).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
    let quotient = value.unscaled / factor;
    let remainder = (value.unscaled % factor).abs();
    let adjust = remainder.checked_mul(2).map(|doubled| doubled >= factor).unwrap_or(true);
    let rounded = if adjust {
        if value.unscaled < 0 {
            quotient.checked_sub(1)
        } else {
            quotient.checked_add(1)
        }
    } else {
        Some(quotient)
    };
    rounded
        .and_then(|n| n.try_into().ok())
        .ok_or_else(|| LucaError::new("Integer overflow", line, column))
}

fn math_round(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "Math.round", line, column)?;
    match expect_number(&args[0], "Math.round", line, column)? {
        Number::Int(n) => Ok(Value::Int(n)),
        Number::Dec(d) => dec_to_int_half_away(&d, line, column).map(Value::Int),
    }
}

fn math_floor(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "Math.floor", line, column)?;
    match expect_number(&args[0], "Math.floor", line, column)? {
        Number::Int(n) => Ok(Value::Int(n)),
        Number::Dec(d) => {
            let factor = Dec::pow10(d.scale).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
            let quotient = d.unscaled / factor;
            let remainder = d.unscaled % factor;
            let floored = if remainder != 0 && d.unscaled < 0 { quotient.checked_sub(1) } else { Some(quotient) };
            floored
                .and_then(|n| n.try_into().ok())
                .map(Value::Int)
                .ok_or_else(|| LucaError::new("Integer overflow", line, column))
        }
    }
}

fn math_ceil(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "Math.ceil", line, column)?;
    match expect_number(&args[0], "Math.ceil", line, column)? {
        Number::Int(n) => Ok(Value::Int(n)),
        Number::Dec(d) => {
            let factor = Dec::pow10(d.scale).ok_or_else(|| LucaError::new("Decimal result is out of range", line, column))?;
            let quotient = d.unscaled / factor;
            let remainder = d.unscaled % factor;
            let ceiled = if remainder != 0 && d.unscaled > 0 { quotient.checked_add(1) } else { Some(quotient) };
            ceiled
                .and_then(|n| n.try_into().ok())
                .map(Value::Int)
                .ok_or_else(|| LucaError::new("Integer overflow", line, column))
        }
    }
}

fn math_min_max(member: &str, args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    if args.len() < 2 {
        return Err(LucaError::new(
            format!("Expected at least 2 arguments, found {} for Math.{member}", args.len()),
            line,
            column,
        ));
    }
    let mut numbers = Vec::with_capacity(args.len());
    for arg in args {
        numbers.push(expect_number(arg, &format!("Math.{member}"), line, column)?);
    }
    let mut best = 0usize;
    for (i, candidate) in numbers.iter().enumerate().skip(1) {
        let ordering = match (&numbers[best], candidate) {
            (Number::Int(a), Number::Int(b)) => a.cmp(b),
            (a, b) => a
                .as_dec()
                .cmp_numeric(&b.as_dec())
                .ok_or_else(|| LucaError::new("Decimal comparison failed", line, column))?,
        };
        let take = if member == "min" {
            ordering == std::cmp::Ordering::Greater
        } else {
            ordering == std::cmp::Ordering::Less
        };
        if take {
            best = i;
        }
    }
    Ok(match &numbers[best] {
        Number::Int(n) => Value::Int(*n),
        Number::Dec(d) => Value::Dec(d.clone()),
    })
}

fn text_one_arg(member: &str, args: &[Value], line: usize, column: usize) -> Result<String, LucaError> {
    arg_count(args, 1, &format!("Text.{member}"), line, column)?;
    expect_str(&args[0], &format!("Text.{member}"), line, column)
}

fn text_upper(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    Ok(Value::Str(text_one_arg("upper", args, line, column)?.to_uppercase()))
}

fn text_lower(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    Ok(Value::Str(text_one_arg("lower", args, line, column)?.to_lowercase()))
}

fn text_trim(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    Ok(Value::Str(text_one_arg("trim", args, line, column)?.trim().to_owned()))
}

fn text_replace(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 3, "Text.replace", line, column)?;
    let text = expect_str(&args[0], "Text.replace", line, column)?;
    let old = expect_str(&args[1], "Text.replace", line, column)?;
    let new = expect_str(&args[2], "Text.replace", line, column)?;
    Ok(Value::Str(text.replace(old.as_str(), new.as_str())))
}

fn text_split(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 2, "Text.split", line, column)?;
    let text = expect_str(&args[0], "Text.split", line, column)?;
    let separator = expect_str(&args[1], "Text.split", line, column)?;
    Ok(Value::new_list(text.split(separator.as_str()).map(|part| Value::Str(part.to_owned())).collect()))
}

fn text_join(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 2, "Text.join", line, column)?;
    let items = match &args[0] {
        Value::List(items) => items.borrow().clone(),
        _ => {
            return Err(LucaError::new(
                format!("Expected list, found {} for Text.join", crate::interpreter::type_name_for_value(&args[0])),
                line,
                column,
            ));
        }
    };
    let separator = expect_str(&args[1], "Text.join", line, column)?;
    let mut parts = Vec::with_capacity(items.len());
    for item in &items {
        match item {
            Value::Str(s) => parts.push(s.clone()),
            _ => {
                return Err(LucaError::new(
                    format!("Expected str, found {} for Text.join", crate::interpreter::type_name_for_value(item)),
                    line,
                    column,
                ));
            }
        }
    }
    Ok(Value::Str(parts.join(separator.as_str())))
}

fn file_path(arg: &Value, what: &str, line: usize, column: usize) -> Result<String, LucaError> {
    expect_str(arg, what, line, column)
}

fn file_read(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "File.read", line, column)?;
    let path = file_path(&args[0], "File.read", line, column)?;
    std::fs::read_to_string(&path)
        .map(Value::Str)
        .map_err(|e| LucaError::new(format!("Could not read file '{path}': {e}"), line, column))
}

fn file_write(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 2, "File.write", line, column)?;
    let path = file_path(&args[0], "File.write", line, column)?;
    let content = expect_str(&args[1], "File.write", line, column)?;
    std::fs::write(&path, content)
        .map(|()| Value::None)
        .map_err(|e| LucaError::new(format!("Could not write file '{path}': {e}"), line, column))
}

fn file_append(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 2, "File.append", line, column)?;
    let path = file_path(&args[0], "File.append", line, column)?;
    let content = expect_str(&args[1], "File.append", line, column)?;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| {
            use std::io::Write;
            file.write_all(content.as_bytes())
        })
        .map(|()| Value::None)
        .map_err(|e| LucaError::new(format!("Could not append to file '{path}': {e}"), line, column))
}

fn file_exists(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "File.exists", line, column)?;
    let path = file_path(&args[0], "File.exists", line, column)?;
    Ok(Value::Bool(std::path::Path::new(&path).exists()))
}

fn file_delete(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "File.delete", line, column)?;
    let path = file_path(&args[0], "File.delete", line, column)?;
    if !std::path::Path::new(&path).exists() {
        crate::interpreter::warn_at(line, column, &format!("File '{path}' does not exist"));
        return Ok(Value::None);
    }
    std::fs::remove_file(&path)
        .map(|()| Value::None)
        .map_err(|e| LucaError::new(format!("Could not delete file '{path}': {e}"), line, column))
}

fn system_args(args: &[Value], cli_args: &[String], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 0, "System.args", line, column)?;
    Ok(Value::new_list(cli_args.iter().map(|a| Value::Str(a.clone())).collect()))
}

fn system_env(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 0, "System.env", line, column)?;
    let mut entries: Vec<DictEntry> = std::env::vars()
        .map(|(key, value)| DictEntry { key, value: Value::Str(value) })
        .collect();
    entries.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(Value::new_dict(entries))
}

fn system_exit(args: &[Value], line: usize, column: usize) -> Result<Value, LucaError> {
    arg_count(args, 1, "System.exit", line, column)?;
    if args[0].is_none() {
        return Err(LucaError::new("Cannot use 'none' as an argument to 'System.exit'", line, column));
    }
    match &args[0] {
        Value::Int(code) => match i32::try_from(*code) {
            // Termination is marked, not thrown: `try to` will not capture it,
            // `finally` still runs, and pending output is flushed on the way out.
            Ok(code) => Err(LucaError::exit(code, line, column)),
            Err(_) => Err(LucaError::new("System.exit code out of range", line, column)),
        },
        _ => Err(LucaError::new(
            format!("Expected int, found {} for System.exit", crate::interpreter::type_name_for_value(&args[0])),
            line,
            column,
        )),
    }
}

/// Dispatch a built-in call. `none` arguments are rejected by the caller.
pub fn call(
    module: &str,
    member: &str,
    args: Vec<Value>,
    cli_args: &[String],
    line: usize,
    column: usize,
) -> Result<Value, LucaError> {
    let args = args.as_slice();
    match (module, member) {
        ("Math", "sqrt") => math_sqrt(args, line, column),
        ("Math", "pow") => math_pow(args, line, column),
        ("Math", "abs") => math_abs(args, line, column),
        ("Math", "round") => math_round(args, line, column),
        ("Math", "floor") => math_floor(args, line, column),
        ("Math", "ceil") => math_ceil(args, line, column),
        ("Math", "min") => math_min_max("min", args, line, column),
        ("Math", "max") => math_min_max("max", args, line, column),
        ("Text", "upper") => text_upper(args, line, column),
        ("Text", "lower") => text_lower(args, line, column),
        ("Text", "trim") => text_trim(args, line, column),
        ("Text", "replace") => text_replace(args, line, column),
        ("Text", "split") => text_split(args, line, column),
        ("Text", "join") => text_join(args, line, column),
        ("File", "read") => file_read(args, line, column),
        ("File", "write") => file_write(args, line, column),
        ("File", "append") => file_append(args, line, column),
        ("File", "exists") => file_exists(args, line, column),
        ("File", "delete") => file_delete(args, line, column),
        ("System", "args") => system_args(args, cli_args, line, column),
        ("System", "env") => system_env(args, line, column),
        ("System", "exit") => system_exit(args, line, column),
        _ => Err(LucaError::new(format!("Module '{module}' has no member '{member}'"), line, column)),
    }
}
