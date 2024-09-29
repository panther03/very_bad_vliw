use std::collections::HashMap;
use std::fmt;
use std::usize::MAX;

use serde::ser::{Serialize, Serializer, SerializeSeq};

use crate::analysis::{AnalyzedBasicBlock, AnalyzedProgram, DepInst};
use crate::isa::{ExecutionUnit, Inst};

#[derive(Debug, Clone)]
pub struct Bundle {
    pub addr: usize,
    pub alu0: Option<DepInst>,
    pub alu1: Option<DepInst>,
    pub mem: Option<DepInst>,
    pub branch: Option<DepInst>,
}

impl Bundle {
    fn new(addr: usize) -> Self {
        Bundle {
            addr: addr,
            alu0: None,
            alu1: None,
            mem: None,
            branch: None,
        }
    }
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_inst(f: &mut fmt::Formatter<'_>, inst: &Option<DepInst>) -> fmt::Result {
            match inst {
                Some(inst) => write!(
                    f,
                    " {:<20} |",
                    inst.inst
                ),
                None => write!(f, "                    |"),
            }
        }
        write!(f, "{} |", self.addr)?;
        fmt_inst(f, &self.alu0)?;
        fmt_inst(f, &self.alu1)?;
        fmt_inst(f, &self.mem)?;
        fmt_inst(f, &self.branch)?;
        write!(f, "\n")
    }
}

type ScheduledBasicBlock = Vec<Bundle>;

pub struct ScheduledProgram {
    pub bbs: Vec<ScheduledBasicBlock>,
    pub starts: HashMap<usize, usize>,
}

impl fmt::Display for ScheduledProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i,bb) in (&self.bbs).iter().enumerate() {
            write!(f, "BasicBlock {}:\n", i)?;
            for bundle in bb {
                write!(f, "{}", bundle)?;
            }
        }
        Ok(())
    }
}

fn compatible(slot: &Bundle, inst: &DepInst) -> bool {
    match inst.inst.opcode.eu_type() {
        ExecutionUnit::ALU => slot.alu0.is_none() || slot.alu1.is_none(),
        ExecutionUnit::Mult => false,
        ExecutionUnit::Mem => slot.mem.is_none(),
        ExecutionUnit::Branch => slot.branch.is_none(),
    }
}

fn schedule_single(slot: &mut Bundle, inst: DepInst) {
    match inst.inst.opcode.eu_type() {
        ExecutionUnit::ALU => {
            if slot.alu0.is_none() {
                slot.alu0 = Some(inst);
            } else {
                slot.alu1 = Some(inst);
            }
        }
        ExecutionUnit::Mult => {}
        ExecutionUnit::Mem => {
            slot.mem = Some(inst);
        }
        ExecutionUnit::Branch => {
            slot.branch = Some(inst);
        }
    }
}

fn fill_schedule(cyc_end: usize, schedule: &mut Vec<Bundle>) {
    while schedule.len() <= cyc_end {
        schedule.push(Bundle::new(schedule.len()));
    }
}

fn reserve_slot(schedule: &mut Vec<Bundle>) {
    if schedule.last().is_none() {
        schedule.push(Bundle::new(schedule.len()));
    }
}

fn min_cycle(starts: &HashMap<usize, usize>, inst: &DepInst) -> usize {
    inst.all_deps().iter().map(|d| 
        starts.get(&d.addr)
        // unit latency assumed on all units, since we have an interlocking pipeline
        .and_then(|s| Some(*s + 1))
        .unwrap_or(0))
    .max().unwrap_or(0)
}

fn asap_local(
    starts: &mut HashMap<usize, usize>,
    inst: DepInst,
    schedule: &mut Vec<Bundle>,
) {
    let min_cycle = min_cycle(starts, &inst);
    fill_schedule(min_cycle, schedule);
    reserve_slot(schedule);
    for slot in schedule.iter_mut().skip(min_cycle) {
        if compatible(slot, &inst) {
            let addr = inst.inst.addr;
            schedule_single(slot, inst);
            starts.insert(addr, slot.addr);
            return;
        }
    }
    // unable to find a compatible slot; add a new one to the end
    let mut bundle = Bundle::new(schedule.len());
    let addr = inst.inst.addr;
    schedule_single(&mut bundle, inst);
    starts.insert(addr, bundle.addr);
    schedule.push(bundle);
}

fn schedule_basicblock(bb: AnalyzedBasicBlock, starts: &mut HashMap<usize, usize>) -> ScheduledBasicBlock {
    let mut schedule = Vec::new();
    for inst in bb.insns {
        asap_local(starts, inst, &mut schedule);
    }
    reserve_slot(&mut schedule);
    if let Some(cf_insn) = bb.cf_insn {
        schedule.last_mut().unwrap().branch = Some(cf_insn);
    }
    schedule
}

pub fn schedule_program(prog: AnalyzedProgram) -> ScheduledProgram {
    let mut starts: HashMap<usize, usize> = HashMap::new();
    let mut bbs: Vec<ScheduledBasicBlock> = Vec::new();

    for bb in prog.bbs {
        bbs.push(schedule_basicblock(bb, &mut starts));
    }

    ScheduledProgram {
        starts,
        bbs
    }
}