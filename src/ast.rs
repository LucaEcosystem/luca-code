use crate::types::{CollectionKind, Type, Value};

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
    pub default: Option<Expr>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct DictEntryExpr {
    pub key: String,
    pub value: Expr,
    pub value_type: Option<Type>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RaiseArg {
    pub field: String,
    pub value: Expr,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainsMode {
    Single,
    Key,
    Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LengthArg {
    Decimals,
    Items,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Declare { name: String, declared_type: Option<Type>, mutable: bool, value: Expr, line: usize, column: usize },
    DeclareCollection { kind: CollectionKind, name: String, element_type: Option<Type>, value: Expr, line: usize, column: usize },
    Assign { name: String, asserted_type: Option<Type>, value: Expr, line: usize, column: usize },
    Print { value: Expr, line: usize, column: usize },
    Forget { name: String, is_const: bool, asserted_type: Option<Type>, line: usize, column: usize },
    Clear { name: String, line: usize, column: usize },
    Change { name: String, new_type: Type, value: Expr, line: usize, column: usize },
    ChangeIndex { name: String, index: Expr, value: Expr, line: usize, column: usize },
    CollectionAdd { name: String, args: Vec<Expr>, line: usize, column: usize },
    CollectionRemove { name: String, arg: Expr, line: usize, column: usize },
    If { branches: Vec<(Expr, Block)>, else_branch: Option<Block>, line: usize, column: usize },
    FuncDef { name: String, params: Vec<Param>, return_type: Option<Type>, body: Block, line: usize, column: usize },
    Return { value: Option<Expr>, line: usize, column: usize },
    Call { name: String, args: Vec<Expr>, line: usize, column: usize },
    MemberCall { object: Box<Expr>, member: String, args: Vec<Expr>, line: usize, column: usize },
    Import { name: String, line: usize, column: usize },
    Try { try_block: Block, capture_block: Option<Block>, finally_block: Option<Block>, line: usize, column: usize },
    ErrorDef { name: String, body: Block, line: usize, column: usize },
    Raise { name: String, overrides: Vec<RaiseArg>, line: usize, column: usize },
    ForTimes { count: Expr, body: Block, line: usize, column: usize },
    ForEach { item: String, collection: Expr, body: Block, line: usize, column: usize },
    While { condition: Expr, body: Block, line: usize, column: usize },
    Stop { line: usize, column: usize },
    Skip { line: usize, column: usize },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Value),
    StringTemplate { value: String, line: usize, column: usize },
    Variable { name: String, asserted_type: Option<Type>, line: usize, column: usize },
    Unary { op: UnaryOp, right: Box<Expr>, line: usize, column: usize },
    Binary { left: Box<Expr>, op: BinaryOp, right: Box<Expr>, line: usize, column: usize },
    Comparison { left: Box<Expr>, op: CompareOp, right: Box<Expr>, line: usize, column: usize },
    Call { name: String, args: Vec<Expr>, line: usize, column: usize },
    Ask { prompt: Box<Expr>, line: usize, column: usize },
    Member { base: Box<Expr>, member: String, line: usize, column: usize },
    MemberCall { base: Box<Expr>, member: String, args: Vec<Expr>, line: usize, column: usize },
    List { items: Vec<Expr>, line: usize, column: usize },
    Tuple { items: Vec<Expr>, line: usize, column: usize },
    Set { items: Vec<Expr>, line: usize, column: usize },
    Dict { entries: Vec<DictEntryExpr>, line: usize, column: usize },
    Index { base: Box<Expr>, index: Box<Expr>, line: usize, column: usize },
    Contains { object: Box<Expr>, mode: ContainsMode, value: Box<Expr>, line: usize, column: usize },
    Length { object: Box<Expr>, arg: Option<LengthArg>, line: usize, column: usize },
    Group(Box<Expr>),
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp { Negate, Not }

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp { Add, Subtract, Multiply, Divide, Modulo, And, Or }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp { Equal, NotEqual, Greater, GreaterEqual, Less, LessEqual }
