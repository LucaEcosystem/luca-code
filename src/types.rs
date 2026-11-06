use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    Dec,
    Str,
    Bool,
    Uni,
}

impl Type {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "int" => Some(Self::Int),
            "dec" => Some(Self::Dec),
            "str" => Some(Self::Str),
            "bool" => Some(Self::Bool),
            "uni" => Some(Self::Uni),
            _ => None,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Int => "int",
            Self::Dec => "dec",
            Self::Str => "str",
            Self::Bool => "bool",
            Self::Uni => "uni",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dec {
    pub unscaled: i128,
    pub scale: u32,
}

impl Dec {
    pub fn new(unscaled: i128, scale: u32) -> Self {
        Self { unscaled, scale }
    }

    pub fn from_int(value: i64) -> Self {
        Self { unscaled: value as i128, scale: 0 }
    }

    pub fn parse_literal(text: &str) -> Result<Self, String> {
        // text is like "20.00" or "3.14", always contains '.'
        let mut parts = text.splitn(2, '.');
        let int_part = parts.next().unwrap_or("");
        let frac_part = parts.next().unwrap_or("");
        if frac_part.is_empty() {
            return Err("Invalid decimal literal".to_owned());
        }
        if !int_part.chars().all(|c| c.is_ascii_digit()) || !frac_part.chars().all(|c| c.is_ascii_digit()) {
            return Err("Invalid decimal literal".to_owned());
        }
        let scale = frac_part.len() as u32;
        if scale > 28 {
            return Err("Decimal literal scale is too large".to_owned());
        }
        let combined = format!("{int_part}{frac_part}");
        // Handle leading zeros
        let unscaled: i128 = combined.parse().map_err(|_| "Decimal literal is out of range".to_owned())?;
        // combined is positive; sign handled by unary
        Ok(Self { unscaled, scale })
    }

    pub fn pow10(scale: u32) -> Option<i128> {
        let mut result: i128 = 1;
        for _ in 0..scale {
            result = result.checked_mul(10)?;
        }
        Some(result)
    }

    pub fn display(&self) -> String {
        if self.scale == 0 {
            return format!("{}.0", self.unscaled);
        }
        let sign = if self.unscaled < 0 { "-" } else { "" };
        let abs = self.unscaled.abs();
        let pow = Self::pow10(self.scale).expect("scale pow10 valid");
        let int_part = abs / pow;
        let frac_part = abs % pow;
        let frac_str = format!("{:0width$}", frac_part, width = self.scale as usize);
        format!("{}{}.{}", sign, int_part, frac_str)
    }

    pub fn neg(&self) -> Option<Self> {
        Some(Self { unscaled: self.unscaled.checked_neg()?, scale: self.scale })
    }

    pub fn add(&self, other: &Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let pow_self = Self::pow10(scale - self.scale)?;
        let pow_other = Self::pow10(scale - other.scale)?;
        let a = self.unscaled.checked_mul(pow_self)?;
        let b = other.unscaled.checked_mul(pow_other)?;
        let unscaled = a.checked_add(b)?;
        Some(Self { unscaled, scale })
    }

    pub fn sub(&self, other: &Self) -> Option<Self> {
        let scale = self.scale.max(other.scale);
        let pow_self = Self::pow10(scale - self.scale)?;
        let pow_other = Self::pow10(scale - other.scale)?;
        let a = self.unscaled.checked_mul(pow_self)?;
        let b = other.unscaled.checked_mul(pow_other)?;
        let unscaled = a.checked_sub(b)?;
        Some(Self { unscaled, scale })
    }

    pub fn mul(&self, other: &Self) -> Option<Self> {
        let unscaled = self.unscaled.checked_mul(other.unscaled)?;
        let scale = self.scale.checked_add(other.scale)?;
        if scale > 28 {
            // For product scale beyond 28, we could keep but limit; error out
            return None;
        }
        let mut result = Self { unscaled, scale };
        // Preserve meaningful scale: trim trailing zeros but keep at least max scale
        // This satisfies spec example 2.50*2.00 => 5.00 (scale 2) instead of 5.0000
        let max_scale = self.scale.max(other.scale);
        while result.scale > max_scale && result.unscaled % 10 == 0 {
            result.unscaled /= 10;
            result.scale -= 1;
        }
        Some(result)
    }

    pub fn div(&self, other: &Self) -> Option<Self> {
        if other.unscaled == 0 {
            return None;
        }
        // Compute exact division with scale determination
        // We want minimal scale that makes division exact, up to 28
        // Handle sign
        let negative = (self.unscaled < 0) ^ (other.unscaled < 0);
        let a_abs = self.unscaled.abs();
        let b_abs = other.unscaled.abs();
        // Rational: (a_abs * 10^(other.scale)) / (b_abs * 10^(self.scale))
        // Let numerator = a_abs * 10^(other.scale)
        // denominator = b_abs * 10^(self.scale)
        let pow_a = Self::pow10(self.scale)?;
        let pow_b = Self::pow10(other.scale)?;
        // Actually numerator = a_abs * pow_b, denominator = b_abs * pow_a
        // To avoid double scaling, we can compute directly with scale handling in division loop
        let numerator = a_abs.checked_mul(pow_b)?;
        let denominator = b_abs.checked_mul(pow_a)?;
        // Find minimal k (0..=28) where (numerator * 10^k) % denominator == 0
        // But also need to account for existing scales: result scale = k
        // We search for smallest k that yields exact division
        let mut k: u32 = 0;
        let max_k = 28;
        let mut scaled_num = numerator;
        // For k=0, check
        loop {
            if k > max_k {
                break;
            }
            if scaled_num % denominator == 0 {
                let mut unscaled = scaled_num / denominator;
                if negative {
                    unscaled = -unscaled;
                }
                // For integer exact results where k==0, we want scale at least 1 to indicate dec (since division always dec)
                // But our display for scale 0 gives "5.0", which already indicates dec, so k==0 is okay.
                // However to retain meaningful scale, if k==0 we could keep 0 and display will add .0
                return Some(Self { unscaled, scale: k });
            }
            if k == max_k {
                break;
            }
            // Scale up
            scaled_num = scaled_num.checked_mul(10)?;
            k += 1;
        }
        // If not exact within max_k, produce truncated result with max_k scale
        // Use max_k scale truncated division
        // Recompute scaled_num for max_k
        let pow_max = Self::pow10(max_k)?;
        let scaled = numerator.checked_mul(pow_max)?;
        let mut unscaled = scaled / denominator;
        if negative {
            unscaled = -unscaled;
        }
        // Trim trailing zeros but keep at least 1? For non-terminating, we could keep max_k
        Some(Self { unscaled, scale: max_k })
    }

    pub fn rem(&self, other: &Self) -> Option<Self> {
        if other.unscaled == 0 {
            return None;
        }
        let scale = self.scale.max(other.scale);
        let pow_self = Self::pow10(scale - self.scale)?;
        let pow_other = Self::pow10(scale - other.scale)?;
        let a = self.unscaled.checked_mul(pow_self)?;
        let b = other.unscaled.checked_mul(pow_other)?;
        let unscaled = a % b;
        Some(Self { unscaled, scale })
    }

    pub fn cmp_numeric(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let scale = self.scale.max(other.scale);
        let pow_self = Self::pow10(scale - self.scale)?;
        let pow_other = Self::pow10(scale - other.scale)?;
        let a = self.unscaled.checked_mul(pow_self)?;
        let b = other.unscaled.checked_mul(pow_other)?;
        Some(a.cmp(&b))
    }

    pub fn is_zero(&self) -> bool {
        self.unscaled == 0
    }
}

impl PartialOrd for Dec {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.cmp_numeric(other)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionKind {
    List,
    Tuple,
    Set,
    Dict,
}

impl fmt::Display for CollectionKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::List => "list",
            Self::Tuple => "tuple",
            Self::Set => "set",
            Self::Dict => "dict",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DictEntry {
    pub key: String,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Dec(Dec),
    Str(String),
    Bool(bool),
    None,
    // Collections are reference values: cloning shares the same storage, so
    // mutations through one binding (including function parameters, per the
    // locked "passed collections remain mutable" rule) are visible through
    // all bindings to that collection. Scalars and `none` are copied.
    List(Rc<RefCell<Vec<Value>>>),
    Tuple(Rc<RefCell<Vec<Value>>>),
    Set(Rc<RefCell<Vec<Value>>>),
    Dict(Rc<RefCell<Vec<DictEntry>>>),
}

impl Value {
    pub fn new_list(items: Vec<Value>) -> Self {
        Self::List(Rc::new(RefCell::new(items)))
    }

    pub fn new_tuple(items: Vec<Value>) -> Self {
        Self::Tuple(Rc::new(RefCell::new(items)))
    }

    pub fn new_set(items: Vec<Value>) -> Self {
        Self::Set(Rc::new(RefCell::new(items)))
    }

    pub fn new_dict(entries: Vec<DictEntry>) -> Self {
        Self::Dict(Rc::new(RefCell::new(entries)))
    }

    /// Deep-copy a collection into fresh storage. Used when reading a
    /// `const` binding so constant contents can never change through an alias.
    pub fn deep_copy_collection(&self) -> Self {
        match self {
            Self::List(items) => Self::new_list(items.borrow().iter().map(|v| v.deep_copy_value()).collect()),
            Self::Tuple(items) => Self::new_tuple(items.borrow().iter().map(|v| v.deep_copy_value()).collect()),
            Self::Set(items) => Self::new_set(items.borrow().iter().map(|v| v.deep_copy_value()).collect()),
            Self::Dict(entries) => Self::new_dict(
                entries
                    .borrow()
                    .iter()
                    .map(|e| DictEntry { key: e.key.clone(), value: e.value.deep_copy_value() })
                    .collect(),
            ),
            _ => self.clone(),
        }
    }

    fn deep_copy_value(&self) -> Self {
        self.deep_copy_collection()
    }
}

impl Value {
    pub fn value_type(&self) -> Type {
        match self {
            Self::Int(_) => Type::Int,
            Self::Dec(_) => Type::Dec,
            Self::Str(_) => Type::Str,
            Self::Bool(_) => Type::Bool,
            Self::None => Type::Bool, // placeholder; none can occupy any type, see require_type
            Self::List(_) | Self::Tuple(_) | Self::Set(_) | Self::Dict(_) => Type::Uni, // placeholder; use collection_kind() for real kind
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn is_collection(&self) -> bool {
        matches!(self, Self::List(_) | Self::Tuple(_) | Self::Set(_) | Self::Dict(_))
    }

    pub fn collection_kind(&self) -> Option<CollectionKind> {
        match self {
            Self::List(_) => Some(CollectionKind::List),
            Self::Tuple(_) => Some(CollectionKind::Tuple),
            Self::Set(_) => Some(CollectionKind::Set),
            Self::Dict(_) => Some(CollectionKind::Dict),
            _ => None,
        }
    }

    pub fn display(&self) -> String {
        match self {
            Self::Int(value) => value.to_string(),
            Self::Dec(value) => value.display(),
            Self::Str(value) => value.clone(),
            Self::Bool(value) => value.to_string(),
            Self::None => "none".to_owned(),
            Self::List(items) => {
                let items = items.borrow();
                let inner: Vec<String> = items.iter().map(display_value_in_collection).collect();
                format!("[{}]", inner.join(", "))
            }
            Self::Tuple(items) => {
                let items = items.borrow();
                if items.is_empty() {
                    "()".to_owned()
                } else if items.len() == 1 {
                    format!("({},)", display_value_in_collection(&items[0]))
                } else {
                    let inner: Vec<String> = items.iter().map(display_value_in_collection).collect();
                    format!("({})", inner.join(", "))
                }
            }
            Self::Set(items) => {
                let items = items.borrow();
                let inner: Vec<String> = items.iter().map(display_value_in_collection).collect();
                format!("{{{}}}", inner.join(", "))
            }
            Self::Dict(entries) => {
                let entries = entries.borrow();
                let inner: Vec<String> = entries
                    .iter()
                    .map(|e| format!("{} = {}", display_string_quoted(&e.key), display_value_in_collection(&e.value)))
                    .collect();
                format!("{{{}}}", inner.join(", "))
            }
        }
    }
}

fn display_string_quoted(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn display_value_in_collection(value: &Value) -> String {
    match value {
        Value::Str(s) => display_string_quoted(s),
        _ => value.display(),
    }
}

/// Content-based equality for collections (recursive), numeric equality for int/dec.
/// Used for `is == to`, `contains`, set uniqueness, dict value checks.
pub fn values_equal(a: &Value, b: &Value) -> Option<bool> {
    // Returns None if incompatible types (caller turns into Error), Some(equal) otherwise.
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Some(x == y),
        (Value::Dec(x), Value::Dec(y)) => x.cmp_numeric(y).map(|o| o == std::cmp::Ordering::Equal),
        (Value::Int(x), Value::Dec(y)) => Dec::from_int(*x).cmp_numeric(y).map(|o| o == std::cmp::Ordering::Equal),
        (Value::Dec(x), Value::Int(y)) => x.cmp_numeric(&Dec::from_int(*y)).map(|o| o == std::cmp::Ordering::Equal),
        (Value::Str(x), Value::Str(y)) => Some(x == y),
        (Value::Bool(x), Value::Bool(y)) => Some(x == y),
        (Value::None, Value::None) => Some(true),
        (Value::None, _) | (_, Value::None) => Some(false),
        (Value::List(x), Value::List(y)) => {
            let x = x.borrow();
            let y = y.borrow();
            if x.len() != y.len() {
                return Some(false);
            }
            for (xi, yi) in x.iter().zip(y.iter()) {
                match values_equal(xi, yi) {
                    Some(true) => {},
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Tuple(x), Value::Tuple(y)) => {
            let x = x.borrow();
            let y = y.borrow();
            if x.len() != y.len() {
                return Some(false);
            }
            for (xi, yi) in x.iter().zip(y.iter()) {
                match values_equal(xi, yi) {
                    Some(true) => {},
                    Some(false) => return Some(false),
                    None => return None,
                }
            }
            Some(true)
        }
        (Value::Set(x), Value::Set(y)) => {
            let x = x.borrow();
            let y = y.borrow();
            if x.len() != y.len() {
                return Some(false);
            }
            // Unordered: each element in x must have a distinct match in y
            let mut used = vec![false; y.len()];
            for xi in x.iter() {
                let mut found = false;
                for (j, yj) in y.iter().enumerate() {
                    if used[j] {
                        continue;
                    }
                    match values_equal(xi, yj) {
                        Some(true) => {
                            used[j] = true;
                            found = true;
                            break;
                        }
                        Some(false) => {},
                        None => return None,
                    }
                }
                if !found {
                    return Some(false);
                }
            }
            Some(true)
        }
        (Value::Dict(x), Value::Dict(y)) => {
            let x = x.borrow();
            let y = y.borrow();
            if x.len() != y.len() {
                return Some(false);
            }
            for xe in x.iter() {
                match y.iter().find(|ye| ye.key == xe.key) {
                    Some(ye) => match values_equal(&xe.value, &ye.value) {
                        Some(true) => {},
                        Some(false) => return Some(false),
                        None => return None,
                    },
                    None => return Some(false),
                }
            }
            Some(true)
        }
        // Different collection kinds are incompatible
        (a, b) if a.is_collection() || b.is_collection() => None,
        // Scalar type mismatch
        _ => None,
    }
}
