use koopa::ir::entities::ValueData;
use koopa::ir::{Value, ValueKind, values};
use std::collections::HashMap;
use std::fmt::Write;

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

impl PReg {
    pub fn to_string(&self) -> String {
        match self {
            PReg::T0 => String::from("t0"),
            PReg::T1 => String::from("t1"),
            PReg::T2 => String::from("t2"),
            PReg::T3 => String::from("t3"),
            PReg::T4 => String::from("t4"),
            PReg::T5 => String::from("t5"),
            PReg::T6 => String::from("t6"),
        }
    }
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
        value: Option<VReg>,
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
            Inst::Ret { value } => {
                if value.is_some() {
                    vec![value.unwrap()]
                } else {
                    vec![]
                }
            }
        }
    }
}

impl RegAllocator {
    pub fn new() -> Self {
        Self {
            locations: std::collections::HashMap::new(),
            free: vec![
                PReg::T6,
                PReg::T5,
                PReg::T4,
                PReg::T3,
                PReg::T2,
                PReg::T1,
                PReg::T0,
            ],
        }
    }
    pub fn take_free_register(&mut self) -> PReg {
        let free = self.free.pop().unwrap();
        free
    }

    pub fn free_register(&mut self, preg: PReg) {
        debug_assert!(!self.free.contains(&preg));
        self.free.push(preg);
    }

    fn alloc_preg(&mut self, vreg: VReg, preg: PReg) {
        self.locations.insert(vreg, preg);
    }

    fn delete_preg(&mut self, vreg: &VReg) {
        self.locations.remove(vreg);
    }

    fn lookup_preg(&self, vreg: &VReg) -> Option<PReg> {
        self.locations.get(vreg).copied()
    }

    pub fn scan_block(&mut self, last_use: HashMap<VReg, usize>, block: &Block, asm: &mut String) {
        for (i, inst) in block.instructions.iter().enumerate() {
            match inst {
                Inst::Li { dst, value } => {
                    let dst_reg = self.take_free_register();
                    self.alloc_preg(*dst, dst_reg);
                    let dst_str = dst_reg.to_string();
                    writeln!(asm, "    li {}, {}", dst_str, value).unwrap();
                }
                Inst::Binary { op, dst, lhs, rhs } => {
                    let lhs_reg = self.lookup_preg(&lhs).unwrap();
                    let rhs_reg = self.lookup_preg(&rhs).unwrap();
                    // for initialize
                    let mut dst_reg = PReg::T0;
                    if last_use.get(&lhs).is_some() {
                        if *last_use.get(&lhs).unwrap() == i {
                            dst_reg = lhs_reg;
                            self.alloc_preg(*dst, dst_reg);
                            self.delete_preg(lhs);
                        }
                    } else if last_use.get(&rhs).is_some() {
                        if *last_use.get(&rhs).unwrap() == i {
                            dst_reg = rhs_reg;
                            self.alloc_preg(*dst, dst_reg);
                            self.delete_preg(rhs);
                        }
                    } else {
                        dst_reg = self.take_free_register();
                        self.alloc_preg(*dst, dst_reg);
                    }
                    match op {
                        BinOp::Add => writeln!(
                            asm,
                            "    add {}, {}, {}",
                            dst_reg.to_string(),
                            lhs_reg.to_string(),
                            rhs_reg.to_string()
                        )
                        .unwrap(),
                        BinOp::Sub => writeln!(
                            asm,
                            "    sub {}, {}, {}",
                            dst_reg.to_string(),
                            lhs_reg.to_string(),
                            rhs_reg.to_string()
                        )
                        .unwrap(),
                        BinOp::Mul => writeln!(
                            asm,
                            "    mul {}, {}, {}",
                            dst_reg.to_string(),
                            lhs_reg.to_string(),
                            rhs_reg.to_string()
                        )
                        .unwrap(),
                    }
                }

                Inst::Ret { value } => {
                    if value.is_some() {
                        let value_reg = self.lookup_preg(&value.unwrap()).unwrap();
                        writeln!(asm, "    mv a0, {}", value_reg.to_string()).unwrap();
                        writeln!(asm, "    ret").unwrap();
                    } else {
                        writeln!(asm, "    ret").unwrap();
                    }
                }
            }

            for (vreg, idx) in &last_use {
                if *idx == i {
                    let preg = self.lookup_preg(vreg);
                    self.delete_preg(vreg);
                    if preg.is_some() {
                        self.free.push(preg.unwrap());
                    }
                }
            }
        }
    }

    fn emit_physical(asm: &mut String, op: BinOp, dst: VReg, lhs: VReg, rhs: VReg) {}
    pub fn allocate_binary(&mut self) {}
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

    pub fn block(&self) -> &Block {
        &self.block
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

    pub fn emit_ret(&mut self, value: Value) -> VReg {
        let dst = self.value_regs.get(&value).unwrap().clone();
        self.block.emit(Inst::Ret { value: Some(dst) });
        dst
    }

    pub fn emit_ret_void(&mut self) {
        self.block.emit(Inst::Ret { value: None });
    }

    pub fn last_use(&mut self) -> HashMap<VReg, usize> {
        let mut result = HashMap::new();
        for (i, inst) in self.block.instructions.iter().enumerate() {
            for reg in inst.uses() {
                result.insert(reg, i);
            }
        }
        result
    }
}
