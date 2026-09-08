use koopa::back::LlvmGenerator;
use koopa::ir::builder::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder};
use koopa::ir::{Program, Type};
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::read_to_string;
use std::io::Result;

use crate::types::CompUnit;

mod types;

lalrpop_mod!(sysy);

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let _mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    let input = read_to_string(input)?;
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    let program = generate_ir(ast);
    let mut g = LlvmGenerator::from_path(output)?;
    g.generate_on(&program)?;
    Ok(())
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

    let value = func_data
        .dfg_mut()
        .new_value()
        .integer(func_def.block.stmt.num);
    let ret = func_data.dfg_mut().new_value().ret(Some(value));
    func_data
        .layout_mut()
        .bb_mut(entry)
        .insts_mut()
        .extend([ret]);

    program
}
