use std::fmt;

use crate::isa::{Inst, Label, Operand};


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Dep {
    pub addr: usize,
    //pub eu: ExecutionUnit,
    pub reg: u32,
}

impl fmt::Display for Dep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{} @ {}", self.reg, self.addr)
    }
}

#[derive(Debug, Clone)]
pub struct DepInst {
    pub inst: Inst,
    pub false_deps: Vec<Dep>,
    pub src1: Option<Dep>,
    pub src2: Option<Dep>
}

impl DepInst {
    pub fn all_deps<'a>(&'a self) -> Vec<&'a Dep> {
        let mut deps: Vec<&'a Dep> = Vec::new();
        if let Some(src1) = &self.src1 {
            deps.push(src1);
        }
        if let Some(src2) = &self.src2 {
            deps.push(src2);
        }
        for dep in self.false_deps.iter() {
            deps.push(dep);
        }
        deps
    }
}

impl fmt::Display for DepInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:\t{:<20}|", self.inst.addr, self.inst)?;
        write!(f, "FD: ")?;
        for false_dep in self.false_deps.iter() {
            write!(f, "{} ", false_dep)?;
        }
        write!(f, "; TD: ")?;
        if let Some(src1) = &self.src1 {
            write!(f, "{} ", src1)?;
        }
        if let Some(src2) = &self.src2 {
            write!(f, "{} ", src2)?;
        }
        write!(f, "]\n")
    }
}

#[derive(Clone)]
pub struct AnalyzedBasicBlock { 
    pub insns: Vec<DepInst>,
    pub cf_insn: Option<DepInst>
}

impl fmt::Display for AnalyzedBasicBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for dep_inst in &self.insns {
            write!(f, "{}", dep_inst)?;
        }
        if let Some(cf_insn) = &self.cf_insn {
            write!(f, "{}", cf_insn)?;
        }
        Ok(())
    }

}

#[derive(Clone)]
pub struct AnalyzedProgram {
    pub bbs: Vec<AnalyzedBasicBlock>
}

impl fmt::Display for AnalyzedProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i,bb) in (&self.bbs).iter().enumerate() {
            write!(f, "BasicBlock {}:\n{}", i, bb)?;
        }
        Ok(())
    }
}


pub fn trace_to_basicblocks(trace: Vec<Inst>) -> Vec<Vec<Inst>> {
    let mut bb_starts: Vec<usize> = Vec::new();
    for inst in trace.iter() {
        if let Label::SrcAddrSpace(l) = inst.label {
            assert!(l < trace.len()*4);
            assert!(l % 4 == 0);
            bb_starts.push(l);
        }
    }

    bb_starts.sort();
    let mut bbs: Vec<Vec<Inst>> = Vec::new();
    let mut curr_bb: Vec<Inst> = Vec::new();
    let trace_len = trace.len();
    for inst in trace {
        let cond = inst.opcode.is_control_flow() || (inst.addr == (trace_len - 1)*4);
        // at the start of a basic block
        if *(bb_starts.first()).unwrap_or(&0) == inst.addr {
            bbs.push(curr_bb);
            curr_bb = vec![inst];
        } else {
            curr_bb.push(inst);
        }

        // at the end of the basic block? Push what we have
        // Note that the instruction can be both the start and end of the basic block, in this case we push the one we just created
        if cond {
            bbs.push(curr_bb);
            curr_bb = Vec::new();
        }
    }
    bbs
}

fn match_deps(new_da: &mut DepInst, old_inst: &Inst) {
    // True dependency
    // We depend on the value from the previous instruction
    let new_inst = new_da.inst;
    if let Operand::Gpr(old_dest) = old_inst.dest {
        let dep = Dep{ addr: old_inst.addr, reg: old_dest };
        if new_inst.src1.is_some() && old_dest == new_inst.src1.unwrap() {
            new_da.src1 = Some(dep.clone());
        }
        if let Operand::Gpr(src2) = new_inst.src2 {
            if old_dest == src2 {
                new_da.src2 = Some(dep);
            }
        }
    }
    // False dependency
    // We are writing to a location either used or written to by a previous instruction
    // With no register renaming, we must be careful to schedule this instruction after that one
    if let Operand::Gpr(new_dest) = new_inst.dest {
        let dep = Dep { addr: old_inst.addr, reg: new_dest };
        if old_inst.src1.is_some() && new_dest == old_inst.src1.unwrap() {
            new_da.false_deps.push(dep);
        } else if let Operand::Gpr(src2) = old_inst.src2 {
            if src2 == new_dest { new_da.false_deps.push(dep); }
        } else if let Operand::Gpr(old_dest) = old_inst.dest {
            if old_dest == new_dest { new_da.false_deps.push(dep); }
        }
    }
}

pub fn dep_analysis(basicblock: Vec<Inst>) -> AnalyzedBasicBlock {
    let mut da_table: Vec<DepInst> = Vec::new();
    let mut cf_insn: Option<DepInst> = None;
    for inst in basicblock {
        let mut dep_inst = DepInst {
            inst: inst,
            false_deps: Vec::new(),
            src1: None,
            src2: None
        };
        for da_entry in da_table.iter() {
            match_deps(&mut dep_inst, &da_entry.inst);
        }

        if inst.opcode.is_control_flow() {
            cf_insn = Some(dep_inst);
        } else {
            da_table.push(dep_inst);
        }
    }
    AnalyzedBasicBlock {
        insns: da_table,
        cf_insn
    }
}