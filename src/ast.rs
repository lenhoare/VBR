#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Expression),
    Wildcard,
    Variable(String),
    Tuple(Vec<Pattern>),
    Struct { name: String, fields: Vec<(String, Pattern)> },
}

impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Pattern::Literal(a), Pattern::Literal(b)) => a == b,
            (Pattern::Wildcard, Pattern::Wildcard) => true,
            (Pattern::Variable(a), Pattern::Variable(b)) => a == b,
            (Pattern::Tuple(a), Pattern::Tuple(b)) => a == b,
            (Pattern::Struct { name: an, fields: af }, Pattern::Struct { name: bn, fields: bf }) => an == bn && af == bf,
            _ => false,
        }
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expression::Integer(a), Expression::Integer(b)) => a == b,
            (Expression::Float(a), Expression::Float(b)) => a == b,
            (Expression::Boolean(a), Expression::Boolean(b)) => a == b,
            (Expression::StringLiteral(a), Expression::StringLiteral(b)) => a == b,
            (Expression::Nothing, Expression::Nothing) => true,
            (Expression::Null, Expression::Null) => true,
            (Expression::Ident(a), Expression::Ident(b)) => a == b,
            (Expression::Call { function: a, args: av }, Expression::Call { function: b, args: bv }) => a == b && av == bv,
            (Expression::Unary { op: ao, expr: ae }, Expression::Unary { op: bo, expr: be }) => ao == bo && ae == be,
            (Expression::Binary { left: al, op: ao, right: ar }, Expression::Binary { left: bl, op: bo, right: br }) => ao == bo && al == bl && ar == br,
            (Expression::Index { collection: ac, index: ai }, Expression::Index { collection: bc, index: bi }) => ac == bc && ai == bi,
            (Expression::FieldAccess { object: ao, field: af }, Expression::FieldAccess { object: bo, field: bf }) => af == bf && ao == bo,
            _ => false,
        }
    }
}

impl Eq for Expression {}

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

}

#[derive(Debug, Clone)]
#[doc(hidden)]
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
    UserDefined(String),
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
    Cast { 
        expr: Box<Expression>, 
        typ: Type 
    },
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
    pub pattern: Pattern,
    pub guard: Option<Box<Expression>>,
    pub body: Vec<Statement>,
}