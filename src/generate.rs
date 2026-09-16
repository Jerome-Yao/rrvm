use koopa::ir::{
    BasicBlock, FunctionData, Value,
    builder::{LocalInstBuilder, ValueBuilder},
};

use crate::types::*;

pub fn generate_exp(exp: &Exp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    generate_add(&exp.expr, func_data, bb)
}

fn generate_primary(exp: &PrimaryExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        PrimaryExp::Number(i) => func_data.dfg_mut().new_value().integer(*i),
        PrimaryExp::Parenthesized(p) => generate_add(&p.expr, func_data, bb),
    }
}

fn generate_add(exp: &AddExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        AddExp::Mul(mul_exp) => generate_mul(mul_exp, func_data, bb),
        AddExp::Add { addexpr, op, expr } => {
            let lhs = generate_add(addexpr, func_data, bb);
            let rhs = generate_mul(expr, func_data, bb);
            let op = match op {
                AddOp::Plus => koopa::ir::BinaryOp::Add,
                AddOp::Minus => koopa::ir::BinaryOp::Sub,
            };
            let inst = func_data.dfg_mut().new_value().binary(op, lhs, rhs);

            func_data.layout_mut().bb_mut(bb).insts_mut().extend([inst]);

            inst
        }
    }
}

fn generate_mul(exp: &MulExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        MulExp::Unary(p) => generate_unary(p, func_data, bb),
        MulExp::Mul { mulexpr, op, expr } => {
            let lhs = generate_mul(mulexpr, func_data, bb);
            let rhs = generate_unary(expr, func_data, bb);
            let op = match op {
                MulOp::Mul => koopa::ir::BinaryOp::Mul,
                MulOp::Div => koopa::ir::BinaryOp::Div,
                MulOp::Mod => koopa::ir::BinaryOp::Mod,
            };
            let inst = func_data.dfg_mut().new_value().binary(op, lhs, rhs);

            func_data.layout_mut().bb_mut(bb).insts_mut().extend([inst]);

            inst
        }
    }
}

fn generate_unary(exp: &UnaryExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        UnaryExp::Primary(p) => generate_primary(p, func_data, bb),
        UnaryExp::Unary { op, expr } => {
            let operand = generate_unary(expr, func_data, bb);

            let inst = match op {
                UnaryOp::Plus => return operand,
                UnaryOp::Minus => {
                    let zero = func_data.dfg_mut().new_value().integer(0);
                    func_data
                        .dfg_mut()
                        .new_value()
                        .binary(koopa::ir::BinaryOp::Sub, zero, operand)
                }
                UnaryOp::Not => {
                    let zero = func_data.dfg_mut().new_value().integer(0);
                    func_data
                        .dfg_mut()
                        .new_value()
                        .binary(koopa::ir::BinaryOp::Eq, operand, zero)
                }
            };

            func_data.layout_mut().bb_mut(bb).insts_mut().extend([inst]);

            inst
        }
    }
}
