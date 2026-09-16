use std::string::String;

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

type Number = i32;

#[derive(Debug)]
pub struct Stmt {
    pub exp: Exp,
}

#[derive(Debug)]
pub struct Exp {
    pub expr: AddExp,
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

#[derive(Debug)]
pub enum MulOp {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum AddOp {
    Plus,
    Minus,
}

#[derive(Debug)]
pub enum PrimaryExp {
    Parenthesized(Box<Exp>),
    Number(Number),
}

#[derive(Debug)]
pub enum UnaryExp {
    Primary(PrimaryExp),
    Unary { op: UnaryOp, expr: Box<UnaryExp> },
}

#[derive(Debug)]
pub enum MulExp {
    Unary(UnaryExp),
    Mul {
        mulexpr: Box<MulExp>,
        op: MulOp,
        expr: Box<UnaryExp>,
    },
}

#[derive(Debug)]
pub enum AddExp {
    Mul(MulExp),
    Add {
        addexpr: Box<AddExp>,
        op: AddOp,
        expr: Box<MulExp>,
    },
}
