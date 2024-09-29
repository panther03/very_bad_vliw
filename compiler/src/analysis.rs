use std::{cmp::max, collections::HashMap, fmt, usize::MAX};

use serde_with::SerializeDisplay;

use crate::isa::{ExecutionUnit, Inst, Opcode, Operand};

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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum BasicBlockSource {
    BB0,
    BB1,
    BB2,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Producer {
    pub addr: usize,
    pub eu: ExecutionUnit,
    pub reg: u32,
}

#[derive(Debug, Clone)]
pub enum Dep {
    Local(Producer),
    Interloop(Producer,Producer),
    Invariant(Producer),
    Postloop(Producer),
    Fixed(usize),
    None
}

impl Dep {
    pub fn base_addr(&self, starts: &HashMap<usize,usize>) -> Option<usize> {
        match self {
            Dep::Local(prod)|Dep::Interloop(_, prod)|Dep::Invariant(prod)|Dep::Postloop(prod) => {
                starts.get(&prod.addr).copied()
            }
            Dep::Fixed(s) => Some(*s),
            Dep::None => None,
        }
    }
    pub fn latency(&self) -> usize {
        match self {
            Dep::Local(prod) => prod.eu.latency(),
            Dep::Interloop(_, prod) => prod.eu.latency(),
            Dep::Invariant(prod) => prod.eu.latency(),
            Dep::Postloop(prod) => prod.eu.latency(),
            Dep::Fixed(_) => 0,
            Dep::None => 0,
        }
    }
    pub fn loop_addr(&self) -> Option<usize> {
        match self {
            Dep::Interloop(prod, _) => Some(prod.addr),
            _ => None,
        }
    }
    pub fn loop_latency(&self) -> usize {
        match self {
            Dep::Interloop(prod, _) => prod.eu.latency(),
            _ => 0,
        }
    }
}

impl fmt::Display for Dep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Dep::Local(prod) => write!(f, "x{}=LC({})", prod.reg, prod.addr),
            Dep::Interloop(prod1, prod2) => write!(f, "x{}=IL({}, {})", prod1.reg, prod1.addr, prod2.addr),
            Dep::Invariant(prod) => write!(f, "x{}=IN({})", prod.reg, prod.addr),
            Dep::Postloop(prod) => write!(f, "x{}=PL({})", prod.reg, prod.addr),
            Dep::None | Dep::Fixed(_) => write!(f, " "),
        }
    }
}

#[derive(Debug, Clone, SerializeDisplay)]
pub struct DepInst {
    pub inst: Inst,
    pub bb: BasicBlockSource,
    pub dep1: Dep,
    pub dep2: Dep,
    pub stage: usize,
    pub pred: Option<u32>
}

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

pub struct BasicBlockAnalysis {
    pub bb0_end: usize,
    pub bb1_end: usize,
}

#[derive(Clone)]
pub struct AnalyzedProgram {
    pub bb0: Vec<DepInst>,
    pub bb1: Vec<DepInst>,
    pub bb2: Vec<DepInst>
}

impl fmt::Display for AnalyzedProgram {
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

fn match_deps(cons_da: &mut DepInst, prod_inst: &Inst, dep_ctx: &dyn Fn(Producer, &Dep) -> Dep) {
    if let Operand::Gpr(prod_dest) = prod_inst.dest {
        let cons_inst = cons_da.inst;
        let prod = Producer{ addr: prod_inst.addr, eu: prod_inst.opcode.eu_type(), reg: prod_dest };
        if cons_inst.src1.is_some() && prod_dest == cons_inst.src1.unwrap() {
            cons_da.dep1 = dep_ctx(prod.clone(), &cons_da.dep1);
        }
        if let Operand::Gpr(src2) = cons_inst.src2 {
            if prod_dest == src2 {
                cons_da.dep2 = dep_ctx(prod, &cons_da.dep2);
            }
        }
    }
}

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
}

pub fn basicblock_analysis(insts: &Vec<Inst>) -> BasicBlockAnalysis {
    let mut analysis = BasicBlockAnalysis {
        bb0_end: insts.len(),
        bb1_end: insts.len(),
    };
    let mut multiple_loops = false;
    for inst in insts.iter() {
        if inst.opcode == Opcode::LOOP {
            if multiple_loops {
                panic!("Multiple loops in trace, not supported");
            }

            let Operand::Immediate(loop_start) = inst.src2 else {
                panic!("malformed internal loop instruction");
            };

            if (loop_start as usize) >= inst.addr || loop_start < 0 {
                panic!("Malformed loop instruction.. loop branches below itself?");
            }
            analysis = BasicBlockAnalysis {
                bb0_end: loop_start as usize,
                bb1_end: inst.addr + 1,
            };
            multiple_loops = true;
        }
    }
    analysis
}

pub fn dep_analysis(insts: Vec<Inst>, bb_analysis: &BasicBlockAnalysis) -> AnalyzedProgram {
    let mut dep_analysis_table: Vec<DepInst> = Vec::new();

    for inst in insts {
        if inst.addr < bb_analysis.bb0_end {
            dep_analysis_table.push(bb0_dep_analysis(inst, &dep_analysis_table));
        } else if inst.addr < bb_analysis.bb1_end {
            let inst_clone = inst.clone();
            let da = bb1_dep_analysis(inst, &dep_analysis_table);
            dep_analysis_table.push(da);
            for cons_da in dep_analysis_table.iter_mut().skip(bb_analysis.bb0_end) {
                promote_bb1_dep(cons_da, &inst_clone);
            }
        } else {
            dep_analysis_table.push(bb2_dep_analysis(inst, &dep_analysis_table));
        }
    }

    let mut bb1 = dep_analysis_table.split_off(bb_analysis.bb0_end);
    let bb2 = bb1.split_off(bb_analysis.bb1_end - bb_analysis.bb0_end);
    AnalyzedProgram{bb0: dep_analysis_table, bb1, bb2}
}
