use crate::generate::generate_exp;
use koopa::back::KoopaGenerator;
use koopa::ir::builder::{BasicBlockBuilder, LocalInstBuilder};
use koopa::ir::{Program, Type, ValueKind};
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fmt::Write;
use std::fs::read_to_string;
use std::io::Result;

use crate::types::CompUnit;
use crate::vreg::{BinOp, RegAllocator, VRegManager};

mod generate;
mod types;
mod vreg;

lalrpop_mod!(sysy);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();

    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    if mode.eq("-koopa") {
        let input = read_to_string(input)?;
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let program = generate_ir(ast);
        let mut g = KoopaGenerator::from_path(&output)?;
        g.generate_on(&program)?;
    } else if mode.eq("-riscv") {
        let input = read_to_string(input)?;
        let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
        let program = generate_ir(ast);
        generate_asm(&program, &output)?;
    }

    Ok(())
}

fn generate_asm(program: &Program, output: &str) -> Result<()> {
    let mut asm = String::new();
    writeln!(&mut asm, "  .text").unwrap();
    for &func in program.func_layout() {
        let func_data = program.func(func);
        let name = func_data.name();
        let name = name.strip_prefix('@').unwrap_or(name);
        writeln!(&mut asm, "  .globl {}", name).unwrap();
        writeln!(&mut asm, "{}:", name).unwrap();
        for (&bb, node) in func_data.layout().bbs() {
            let mut vreg_manager = VRegManager::new();
            let allocator = RegAllocator::new();
            for &inst in node.insts().keys() {
                let value_data = func_data.dfg().value(inst);
                match value_data.kind() {
                    ValueKind::Return(ret) => {
                        let ret_value = ret.value();
                        if ret_value.is_some() {
                            let v = func_data.dfg().value(ret_value.unwrap());
                            match v.kind() {
                                ValueKind::Integer(v) => ret_i32_asm(v.value(), &mut asm),
                                _ => unreachable!(),
                            }
                        }
                    }
                    ValueKind::Binary(val) => {
                        let op = val.op();
                        let op = match op {
                            koopa::ir::BinaryOp::Add => BinOp::Add,
                            koopa::ir::BinaryOp::Sub => BinOp::Sub,
                            koopa::ir::BinaryOp::Mul => BinOp::Mul,
                            _ => unreachable!(),
                        };
                        let lr =
                            vreg_manager.operand_reg(val.lhs(), func_data.dfg().value(val.lhs()));
                        let rr =
                            vreg_manager.operand_reg(val.rhs(), func_data.dfg().value(val.lhs()));
                        let result = vreg_manager.emit_binary(op, lr, rr);

                        vreg_manager.insert(inst, result);
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
    std::fs::write(output, &asm)?;
    Ok(())
}

fn ret_i32_asm(ret: i32, asm: &mut String) {
    writeln!(asm, "  li a0, {}", ret).unwrap();
    writeln!(asm, "  ret").unwrap();
}

fn generate_ir(ast: CompUnit) -> Program {
    let mut program = Program::new();
    let func_def = ast.func_def;
    let ret_type = match func_def.func_type {
        types::FuncType::Int => Type::get_i32(),
    };

    let func = program.new_func_def(format!("@{}", func_def.ident), Vec::new(), ret_type);
    let func_data = program.func_mut(func);

    let entry = func_data
        .dfg_mut()
        .new_bb()
        .basic_block(Some("%entry".into()));
    func_data.layout_mut().bbs_mut().extend([entry]);

    let value = generate_exp(&func_def.block.stmt.exp, func_data, entry);
    let ret = func_data.dfg_mut().new_value().ret(Some(value));

    func_data
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .extend([ret]);

    program
}
