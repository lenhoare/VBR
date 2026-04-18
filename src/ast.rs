#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Dim { 
        mutable: bool, 
        typ: Type, 
        name: String, 
        init: Option<Box<Expression>> 
    },
    Const { 
        pub_: bool, 
        name: String, 
        value: Expression 
    },
    Set { 
        mutable: bool, 
        target: Expression, 
        value: Box<Expression> 
    },
    If { 
        condition: Box<Expression>, 
        then_block: Vec<Statement>, 
        else_block: Vec<Statement> 
    },
    Select { 
        value: Box<Expression>, 
        arms: Vec<SelectArm>, 
        else_arm: Option<Vec<Statement>> 
    },
    For { 
        variable: String, 
        start: Box<Expression>, 
        end: Box<Expression>, 
        step: Option<Box<Expression>>, 
        body: Vec<Statement> 
    },
    ForEach { 
        variable: String, 
        collection: Box<Expression>, 
        body: Vec<Statement> 
    },
    While { 
        condition: Box<Expression>, 
        body: Vec<Statement> 
    },
    DoWhile { 
        condition: Box<Expression>, 
        body: Vec<Statement>, 
        until: bool 
    },
    ExitLoop,
    Continue,
    Function { 
        name: String, 
        params: Vec<Param>, 
        return_type: Option<Type>, 
        body: Vec<Statement> 
    },
    Return(Option<Box<Expression>>),
    Match { 
        value: Box<Expression>, 
        arms: Vec<MatchArm>,
    },
    Expr(Box<Expression>),

    Try(Box<Expression>),
    Cast { 
        expr: Box<Expression>, 
        typ: Type 
    },
    Clone(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum SelectArm {
    Value { value: Expression, body: Vec<Statement> },
    Range { start: Expression, end: Expression, body: Vec<Statement> },
    Else(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub typ: Type,
    pub by_ref: bool,
    pub mut_: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    I16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    U8,
    String,
    Vec(Box<Type>),
    HashMap(Box<Type>, Box<Type>),
    UserDefined(String),
    Result(Box<Type>, Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    StringLiteral(String),
    Nothing,
    Null,
    Ident(String),
    Call { 
        function: Box<Expression>, 
        args: Vec<Expression> 
    },
    Unary { op: UnaryOp, expr: Box<Expression> },
    Binary { 
        left: Box<Expression>, 
        op: BinaryOp, 
        right: Box<Expression> 
    },
    Index { 
        collection: Box<Expression>, 
        index: Box<Expression> 
    },
    FieldAccess { 
        object: Box<Expression>, 
        field: String 
    },
    Ok(Box<Expression>),
    Err(Box<Expression>),
    Match { 
        value: Box<Expression>, 
        arms: Vec<MatchArm>,
    },
    Try(Box<Expression>),
    Cast { 
        expr: Box<Expression>, 
        typ: Type 
    },
    Clone(Box<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    As,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Expression,
    pub body: Vec<Statement>,
}