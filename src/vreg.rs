use koopa::ir::entities::ValueData;
use koopa::ir::{Value, ValueKind};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VReg(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PReg {
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
}

#[derive(Debug)]
pub struct VRegManager {
    value_regs: std::collections::HashMap<Value, VReg>,
    vreg_allocator: VRegAllocator,
    block: Block,
}

#[derive(Debug)]
pub struct RegAllocator {
    locations: std::collections::HashMap<VReg, PReg>,
    free: Vec<PReg>,
}

#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug)]
pub enum Inst {
    Li {
        dst: VReg,
        value: i32,
    },
    Binary {
        op: BinOp,
        dst: VReg,
        lhs: VReg,
        rhs: VReg,
    },
    Ret {
        value: VReg,
    },
}

#[derive(Debug)]
pub struct Block {
    instructions: Vec<Inst>,
}

impl Block {
    pub fn emit(&mut self, inst: Inst) {
        self.instructions.push(inst);
    }
}

// Keep this counter per function, shared across its blocks.
#[derive(Debug)]
pub struct VRegAllocator {
    next: u32,
}

impl VRegAllocator {
    pub fn new() -> Self {
        Self { next: 0 }
    }
    pub fn fresh(&mut self) -> VReg {
        let reg = VReg(self.next);
        self.next += 1;
        reg
    }
}

impl Inst {
    fn uses(&self) -> Vec<VReg> {
        match self {
            Inst::Li { dst, .. } => vec![],
            Inst::Binary { dst, lhs, rhs, .. } => vec![*lhs, *rhs],
            Inst::Ret { value } => vec![*value],
        }
    }
}

impl RegAllocator {
    pub fn new() -> Self {
        Self {
            locations: std::collections::HashMap::new(),
            free: vec![
                PReg::T0,
                PReg::T1,
                PReg::T2,
                PReg::T3,
                PReg::T4,
                PReg::T5,
                PReg::T6,
            ],
        }
    }
    pub fn take_free_register(&mut self) -> PReg {
        let free = self.free.pop().unwrap();
        free
    }
}

impl VRegManager {
    pub fn new() -> Self {
        Self {
            value_regs: HashMap::new(),
            vreg_allocator: VRegAllocator::new(),
            block: Block {
                instructions: vec![],
            },
        }
    }

    pub fn operand_reg(&mut self, value: Value, value_data: &ValueData) -> VReg {
        if let Some(&reg) = self.value_regs.get(&value) {
            return reg;
        }

        match value_data.kind() {
            ValueKind::Integer(n) => {
                let reg = self.vreg_allocator.fresh();
                self.block.emit(Inst::Li {
                    dst: reg,
                    value: n.value(),
                });
                reg
            }
            _ => {
                // Instruction results and parameters should already have
                // register mappings established by your lowering pass.
                panic!("missing register mapping for operand");
            }
        }
    }

    pub fn insert(&mut self, value: Value, vreg: VReg) {
        self.value_regs.insert(value, vreg);
    }

    pub fn emit_constant(&mut self, value: i32) -> VReg {
        let dst = self.vreg_allocator.fresh();
        self.block.emit(Inst::Li { dst, value });
        dst
    }

    pub fn emit_binary(&mut self, op: BinOp, lhs: VReg, rhs: VReg) -> VReg {
        let dst = self.vreg_allocator.fresh();
        self.block.emit(Inst::Binary { op, dst, lhs, rhs });
        dst
    }

    fn last_use(&mut self) -> HashMap<VReg, usize> {
        let mut result = HashMap::new();
        for (i, inst) in self.block.instructions.iter().enumerate() {
            for reg in inst.uses() {
                result.insert(reg, i);
            }
        }
        result
    }
}
