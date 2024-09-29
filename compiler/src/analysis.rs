use std::fmt;

use crate::isa::{Inst, Label, Operand};

/*#[derive(Debug)]
pub struct Producers {
    addr: usize,
    loop_addr: Option<usize>,
}

impl fmt::Display for Producers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.loop_addr {
            Some(loop_addr) => write!(f, "{} or {}", self.addr, loop_addr),
            None => write!(f, "{}", self.addr),
        }
    }
}

fn deps_fmt(deps: &Deps, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut first = true;
    let mut remaining = 10;
    for (reg, producers) in deps.iter() {
        if !first {
            write!(f, ", ")?;
            remaining -= 2;
        }
        let producers_string = format!("{}: {}", reg, producers);
        write!(f, "{}", producers_string)?;
        remaining -= producers_string.len();
        first = false;
    }
    if remaining > 0 {
        write!(f, "{: <1$}", "", remaining)?;
    }
    write!(f, "| ")?;
    Ok(()) 
}*/

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
/*
impl DepInst {
    pub fn new(inst: Inst, bb: BasicBlockSource) -> Self {
        DepInst {
            inst,
            bb,
            dep1: Dep::None,
            dep2: Dep::None,
            stage: 0,
            pred: None
        }
    }
    
    pub fn with_dep(mut self, dep: Dep) -> Self {
        self.dep1 = dep;
        self
    }
}

impl fmt::Display for DepInst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = self.pred {
            write!(f, "(p{}) {}", p, self.inst)
        } else {
            write!(f, "{}", self.inst)
        }
    }
}

impl DepInst {
    pub fn analysis_dbg_fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:<3} | {:<5} | {:?} | {:<3} | {} | {} ??? ",
            self.inst.addr,
            self.inst.opcode.to_str(),
            self.bb,
            format!("{}", self.inst.dest).as_str(),
            self.dep1,
            self.dep2
        )?;
        //deps_fmt(&self.local_deps, f)?;
        //deps_fmt(&self.interloop_deps, f)?;
        //deps_fmt(&self.invariant_deps, f)?;
        //deps_fmt(&self.postloop_deps, f)?;
        Ok(())
    }

    pub fn all_deps<'a>(&'a mut self) -> Vec<(&'a Dep, &'a mut u32)> {
        let mut deps: Vec<(&'a Dep, &'a mut u32)> = Vec::new();
        if let Some(reg1) = &mut self.inst.src1 {
            deps.push((&self.dep1, reg1));
        }
        if let Operand::Gpr(reg2) = &mut self.inst.src2 {
            deps.push((&self.dep2, reg2));
        }
        deps
    }
}



impl fmt::Display for SerialProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BB0:\n")?;
        for dep_inst in self.bb0.iter() {
            dep_inst.analysis_dbg_fmt(f)?;
            write!(f, "\n")?;
        }
        write!(f, "BB1:\n")?;
        for dep_inst in self.bb1.iter() {
            dep_inst.analysis_dbg_fmt(f)?;
            write!(f, "\n")?;
        }
        write!(f, "BB2:\n")?;
        for dep_inst in self.bb2.iter() {
            dep_inst.analysis_dbg_fmt(f)?;
            write!(f, "\n")?;
        }
        Ok(())
    }

}

//Output dependencies: Vec<Consumer>
//Dep1: Producer
//Dep2: Producer 
//
//
//add x0, x1, x1
//add x1, x2, x3
//add x3, x1, x0
//
//add x1, x0, x0
//
//add x1, x2, x3
//add x3, x1, x0




fn handle_local_dep(
    cons_da: &mut DepInst,
    prod_bb: BasicBlockSource,
    prod_inst: &Inst,
) {
    // must be in same basic block for a local dependency
    if cons_da.bb != prod_bb {
        return 
    }
    match_deps(cons_da, prod_inst, &|p, _| { Dep::Local(p) });
}

fn handle_invariant_dep(
    cons_da: &mut DepInst,
    prod_bb: BasicBlockSource,
    prod_inst: &Inst,
) {
    // Invariants are produced in BB0
    if prod_bb != BasicBlockSource::BB0 {
        return;
    }
    match_deps(cons_da, prod_inst, &|p, _| { Dep::Invariant(p) });
}

fn handle_postloop_dep(
    cons_da: &mut DepInst,
    prod_bb: BasicBlockSource,
    prod_inst: &Inst,
) {
    // Postloop deps come from the loop
    if prod_bb != BasicBlockSource::BB1 {
        return;
    }
    match_deps(cons_da, prod_inst, &|p, _| { Dep::Postloop(p) });
}

fn bb0_dep_analysis(inst: Inst, da_table: &Vec<DepInst>) -> DepInst {
    let mut cons_da_entry = DepInst::new(inst, BasicBlockSource::BB0);
    for da_entry in da_table.iter() {
        handle_local_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst,
        );
    }
    cons_da_entry
}

fn bb1_dep_analysis(inst: Inst, da_table: &Vec<DepInst>) -> DepInst {
    let mut cons_da_entry = DepInst::new(inst, BasicBlockSource::BB1);
    for da_entry in da_table.iter() {
        handle_invariant_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst,
        );
        handle_local_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst
        );
    }
    cons_da_entry
}

fn promote_bb1_dep(cons_da: &mut DepInst, prod_inst: &Inst) {
    match_deps(cons_da, prod_inst, &|p, d| {
        match d {
            Dep::Invariant(i) => {
                Dep::Interloop(p, i.clone())
            }
            Dep::Interloop(_, ii) => {
                Dep::Interloop(p, ii.clone())
            }
            Dep::Local(_) => { d.clone() }
            _ => {
                panic!("Interloop dependency has no producer in BB0")
            }
        }
    });
}

fn bb2_dep_analysis(inst: Inst, da_table: &Vec<DepInst>) -> DepInst {
    let mut cons_da_entry = DepInst::new(inst, BasicBlockSource::BB2);
    for da_entry in da_table.iter() {
        handle_invariant_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst,
        );
        handle_postloop_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst,
        );
        handle_local_dep(
            &mut cons_da_entry,
            da_entry.bb,
            &da_entry.inst,
        );
    }
    cons_da_entry
}*/

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