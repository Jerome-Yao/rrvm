use koopa::ir::{
    BasicBlock, FunctionData, Value,
    builder::{LocalInstBuilder, ValueBuilder},
};

use crate::types::{Exp, PrimaryExp, UnaryExp, UnaryOp};

pub fn generate_exp(exp: &Exp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    generate_unary(&exp.unary, func_data, bb)
}

fn generate_primary(exp: &PrimaryExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        PrimaryExp::Number(i) => func_data.dfg_mut().new_value().integer(*i),
        PrimaryExp::Parenthesized(p) => generate_unary(&p.unary, func_data, bb),
    }
}

fn generate_unary(exp: &UnaryExp, func_data: &mut FunctionData, bb: BasicBlock) -> Value {
    match exp {
        UnaryExp::Primary(p) => generate_primary(p, func_data, bb),
        UnaryExp::Unary { op, expr } => {
            let operand = generate_unary(expr, func_data, bb);

            let inst = match op {
                UnaryOp::Plus => operand,
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
