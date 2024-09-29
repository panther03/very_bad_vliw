use std::collections::HashMap;
use std::fmt;
use std::usize::MAX;

use serde::ser::{Serialize, Serializer, SerializeSeq};

use crate::analysis::{AnalyzedProgram, BasicBlockAnalysis, BasicBlockSource, DepInst};
use crate::isa::{ExecutionUnit, Inst};

#[derive(Debug, Clone)]
pub struct ScheduleSlot {
    pub bb: Option<BasicBlockSource>, // A "clean" slot has no basic block yet, it is just a dummy
    pub addr: usize,
    pub alu0: Option<DepInst>,
    pub alu1: Option<DepInst>,
    pub mult: Option<DepInst>,
    pub mem: Option<DepInst>,
    pub branch: Option<DepInst>,
}

impl ScheduleSlot {
    fn new(addr: usize) -> Self {
        ScheduleSlot {
            bb: None,
            addr: addr,
            alu0: None,
            alu1: None,
            mult: None,
            mem: None,
            branch: None,
        }
    }

    pub fn valid_insts_mut<'a>(&'a mut self) -> Vec<&'a mut DepInst> {
        let mut insts = Vec::new();
        if let Some(inst) = &mut self.alu0 {
            insts.push(inst);
        }
        if let Some(inst) = &mut self.alu1 {
            insts.push(inst);
        }
        if let Some(inst) = &mut self.mult {
            insts.push(inst);
        }
        if let Some(inst) = &mut self.mem {
            insts.push(inst);
        }
        if let Some(inst) = &mut self.branch {
            insts.push(inst);
        }
        insts
    }

    pub fn valid_insts<'a>(&'a self) -> Vec<&'a DepInst> {
        let mut insts = Vec::new();
        if let Some(inst) = &self.alu0 {
            insts.push(inst);
        }
        if let Some(inst) = &self.alu1 {
            insts.push(inst);
        }
        if let Some(inst) = &self.mult {
            insts.push(inst);
        }
        if let Some(inst) = &self.mem {
            insts.push(inst);
        }
        if let Some(inst) = &self.branch {
            insts.push(inst);
        }
        insts
    }


    pub fn all_insts(&self) -> Vec<DepInst> {
        let mut insts = Vec::new();
        if let Some(inst) = &self.alu0 {
            insts.push(inst.clone());
        } else {
            insts.push(DepInst::new(Inst::nop(), BasicBlockSource::BB0));
        }
        if let Some(inst) = &self.alu1 {
            insts.push(inst.clone());
        } else {
            insts.push(DepInst::new(Inst::nop(), BasicBlockSource::BB0));
        }
        if let Some(inst) = &self.mult {
            insts.push(inst.clone());
        } else {
            insts.push(DepInst::new(Inst::nop(), BasicBlockSource::BB0));
        }
        if let Some(inst) = &self.mem {
            insts.push(inst.clone());
        } else {
            insts.push(DepInst::new(Inst::nop(), BasicBlockSource::BB0));
        }
        if let Some(inst) = &self.branch {
            insts.push(inst.clone());
        } else {
            insts.push(DepInst::new(Inst::nop(), BasicBlockSource::BB0));
        }
        insts
    
    }
}

impl Serialize for ScheduleSlot {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let insts = self.all_insts();
        let mut seq = serializer.serialize_seq(Some(insts.len()))?;
        for inst in insts {
            seq.serialize_element(&inst)?;
        }
        seq.end()
    }
}

impl fmt::Display for ScheduleSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_inst(f: &mut fmt::Formatter<'_>, inst: &Option<DepInst>) -> fmt::Result {
            match inst {
                Some(inst) => write!(
                    f,
                    " {:<3}({:<2}S) [{}]|",
                    inst.inst.addr, inst.stage, inst.inst.dest
                ),
                None => write!(f, "     |"),
            }
        }
        write!(f, "{} | {:?} |", self.addr, self.bb)?;
        fmt_inst(f, &self.alu0)?;
        fmt_inst(f, &self.alu1)?;
        fmt_inst(f, &self.mult)?;
        fmt_inst(f, &self.mem)?;
        fmt_inst(f, &self.branch)
    }
}

#[derive(Debug)]
pub struct ScheduledProgram {
    pub bb0: Vec<ScheduleSlot>,
    pub bb1: Vec<ScheduleSlot>,
    pub bb2: Vec<ScheduleSlot>,
    pub starts: HashMap<usize, usize>,
    pub bb1_len_preadj: Option<usize>,
    pub stages: Option<HashMap<usize,usize>>
}

impl fmt::Display for ScheduledProgram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BB0:\n")?;
        for slot in self.bb0.iter() {
            write!(f, "{}\n", slot)?;
        }
        write!(f, "BB1:\n")?;
        for slot in self.bb1.iter() {
            write!(f, "{}\n", slot)?;
        }
        write!(f, "BB2:\n")?;
        for slot in self.bb2.iter() {
            write!(f, "{}\n", slot)?;
        }
        Ok(())
    }
}

fn compatible(slot: &ScheduleSlot, inst: &DepInst) -> bool {
    if let Some(bb) = slot.bb {
        if bb != inst.bb {
            return false;
        }
    }
    match inst.inst.opcode.eu_type() {
        ExecutionUnit::ALU => slot.alu0.is_none() || slot.alu1.is_none(),
        ExecutionUnit::Mult => slot.mult.is_none(),
        ExecutionUnit::Mem => slot.mem.is_none(),
        ExecutionUnit::Branch => slot.branch.is_none(),
    }
}

fn schedule_single(slot: &mut ScheduleSlot, inst: DepInst) {
    if slot.bb == None {
        slot.bb = Some(inst.bb);
    }

    match inst.inst.opcode.eu_type() {
        ExecutionUnit::ALU => {
            if slot.alu0.is_none() {
                slot.alu0 = Some(inst);
            } else {
                slot.alu1 = Some(inst);
            }
        }
        ExecutionUnit::Mult => {
            slot.mult = Some(inst);
        }
        ExecutionUnit::Mem => {
            slot.mem = Some(inst);
        }
        ExecutionUnit::Branch => {
            slot.branch = Some(inst);
        }
    }
}

fn fill_schedule(cyc_end: usize, schedule: &mut Vec<ScheduleSlot>) {
    while schedule.len() <= cyc_end {
        schedule.push(ScheduleSlot::new(schedule.len()));
    }
}

fn reserve_slot(schedule: &mut Vec<ScheduleSlot>) {
    if schedule.last().is_none() || schedule.last().unwrap().bb.is_some() {
        schedule.push(ScheduleSlot::new(schedule.len()));
    }
}

pub fn prune_schedule(schedule: &mut Vec<ScheduleSlot>) {
    while schedule.len() > 0 && schedule.last().unwrap().bb.is_none() {
        schedule.pop();
    }
}

fn min_cycle(starts: &HashMap<usize, usize>, inst: &DepInst) -> usize {
    let dep1_start = &inst
        .dep1
        .base_addr(starts)
        .and_then(|start| Some(start + inst.dep1.latency()));
    let dep2_start = &inst
        .dep2
        .base_addr(starts)
        .and_then(|start| Some(start + inst.dep2.latency()));
    match (dep1_start, dep2_start) {
        (Some(a), Some(b)) => std::cmp::max(*a, *b),
        (Some(a), None) => *a,
        (None, Some(b)) => *b,
        (None, None) => 0,
    }
}

fn min_cycle_pip(
    starts: &HashMap<usize, usize>,
    stages: &HashMap<usize, usize>,
    ii: usize,
    inst: &DepInst,
) -> usize {
    let dep1_start = &inst
        .dep1
        .base_addr(starts)
        .and_then(|start| Some(start + inst.dep1.latency()));
    let dep1_offset = &inst
        .dep1
        .base_addr(stages)
        .and_then(|stage| Some(stage * ii))
        .unwrap_or(0);
    let dep2_start = &inst
        .dep2
        .base_addr(starts)
        .and_then(|start| Some(start + inst.dep2.latency()));
    let dep2_offset = &inst
        .dep2
        .base_addr(stages)
        .and_then(|stage| Some(stage * ii))
        .unwrap_or(0);
    match (dep1_start, dep2_start) {
        (Some(a), Some(b)) => std::cmp::max(*a + dep1_offset, *b + dep2_offset),
        (Some(a), None) => *a + dep1_offset,
        (None, Some(b)) => *b + dep2_offset,
        (None, None) => 0,
    }
}

fn worst_interloop_producer_single(starts: &HashMap<usize, usize>, inst: &DepInst) -> i32 {
    let dep1_start: &Option<i32> = &inst
        .dep1
        .loop_addr()
        .and_then(|addr| starts.get(&addr))
        .and_then(|start| Some((*start + inst.dep1.loop_latency()) as i32));
    let dep2_start = &inst
        .dep2
        .loop_addr()
        .and_then(|addr| starts.get(&addr))
        .and_then(|start| Some((*start + inst.dep2.loop_latency()) as i32));
    match (dep1_start, dep2_start) {
        (Some(a), Some(b)) => std::cmp::max(*a, *b),
        (Some(a), None) => *a,
        (None, Some(b)) => *b,
        (None, None) => 0,
    }
}

fn asap_local(
    starts: &mut HashMap<usize, usize>,
    inst: DepInst,
    schedule: &mut Vec<ScheduleSlot>,
) -> Result<usize, String> {
    let min_cycle = min_cycle(starts, &inst);
    fill_schedule(min_cycle, schedule);
    reserve_slot(schedule);

    for slot in schedule.iter_mut().skip(min_cycle) {
        if compatible(slot, &inst) {
            let addr = inst.inst.addr;
            schedule_single(slot, inst);
            starts.insert(addr, slot.addr);
            return Ok(slot.addr);
        }
    }
    Err("No compatible slot found".to_string())
}

fn asap_local_pip(
    starts: &mut HashMap<usize, usize>,
    stages: &mut HashMap<usize, usize>,
    inst: &DepInst,
    schedule: &mut Vec<ScheduleSlot>,
    bb0_len: usize,
    ii: usize,
) -> Result<usize, ()> {
    let min_cycle_unroll = std::cmp::max(
        0,
        (min_cycle_pip(starts, &stages, ii, inst) - bb0_len) as i32,
    ) as usize;
    let min_cycle = min_cycle_unroll % ii;
    let min_stage = min_cycle_unroll / ii;

    for slot in schedule.iter_mut().skip(min_cycle) {
        if compatible(slot, inst) {
            let addr = inst.inst.addr;
            let mut inst_with_stage = inst.clone();
            inst_with_stage.stage = min_stage;
            schedule_single(slot, inst_with_stage);
            starts.insert(addr, slot.addr);
            stages.insert(addr, min_stage);
            return Ok(slot.addr);
        }
    }
    for slot in schedule.iter_mut().take(min_cycle) {
        if compatible(slot, inst) {
            let addr = inst.inst.addr;
            let mut inst_with_stage = inst.clone();
            inst_with_stage.stage = min_stage + 1;
            schedule_single(slot, inst_with_stage);
            starts.insert(addr, slot.addr);
            stages.insert(addr, min_stage + 1);
            return Ok(slot.addr);
        }
    }
    Err(())
}

fn worst_interloop_slack(
    schedule: &Vec<ScheduleSlot>,
    starts: &HashMap<usize, usize>,
    bb0_len: usize,
) -> i32 {
    let mut min_slack = 0;
    for slot in schedule.iter().skip(bb0_len) {
        let mut slack = 0;
        let sc = slot.addr as i32;
        if let Some(inst) = &slot.alu0 {
            slack = std::cmp::min(slack, sc - worst_interloop_producer_single(starts, inst));
        }
        if let Some(inst) = &slot.alu1 {
            slack = std::cmp::min(slack, sc - worst_interloop_producer_single(starts, inst));
        }
        if let Some(inst) = &slot.mult {
            slack = std::cmp::min(slack, sc - worst_interloop_producer_single(starts, inst));
        }
        if let Some(inst) = &slot.mem {
            slack = std::cmp::min(slack, sc - worst_interloop_producer_single(starts, inst));
        }
        if let Some(inst) = &slot.branch {
            slack = std::cmp::min(slack, sc - worst_interloop_producer_single(starts, inst));
        }
        min_slack = std::cmp::min(min_slack, slack);
    }
    min_slack
}

pub fn flat_block_schedule(
    insts: Vec<DepInst>,
    starts: &mut HashMap<usize, usize>,
    schedule: &mut Vec<ScheduleSlot>,
) -> usize {
    let mut last_addr: i32 = -1;
    for inst in insts {
        let Ok(addr) = asap_local(starts, inst, schedule) else {
            panic!("Error scheduling instruction");
        };
        last_addr = addr as i32;
    }
    (last_addr + 1) as usize
}

pub fn handle_bb0_bubble(
    bb1_insts: &Vec<DepInst>,
    starts: &mut HashMap<usize, usize>,
    schedule: &mut Vec<ScheduleSlot>
) {
    let mut worst_start = 0;
    let bb0_len = schedule.len();
    let mut prods: Vec<&crate::analysis::Producer> = Vec::new();
    for inst in bb1_insts {
        if let crate::analysis::Dep::Invariant(p) = &inst.dep1 {
            prods.push(p);
        }
        if let crate::analysis::Dep::Invariant(p) = &inst.dep2 {
            prods.push(p);
        }
        if let crate::analysis::Dep::Interloop(_, p) = &inst.dep1 {
            prods.push(p);
        }
        if let crate::analysis::Dep::Interloop(_, p) = &inst.dep2 {
            prods.push(p);
        }
    }
    for prod in prods {
        let start = starts.get(&prod.addr).unwrap();
        worst_start = std::cmp::max(worst_start, prod.eu.latency() + *start);
    }
    if worst_start <= bb0_len { return; }
    for _ in 0..(worst_start-bb0_len) {
        let mut slot = ScheduleSlot::new(schedule.len());
        slot.bb = Some(BasicBlockSource::BB0);
        schedule.push(slot);
    }
}

fn bb1_nopip_schedule(
    insts: Vec<DepInst>,
    starts: &mut HashMap<usize, usize>,
    schedule: &mut Vec<ScheduleSlot>,
    bb0_len: usize,
) -> usize {
    let mut loop_instr: Option<DepInst> = None;
    for inst in insts {
        if inst.inst.opcode.eu_type() == ExecutionUnit::Branch {
            loop_instr = Some(inst);
            continue;
        }
        asap_local(starts, inst, schedule);
    }
    prune_schedule(schedule);
    let bb1_len_preadj = schedule.len();
    // negative slack is when we need to add things
    let required_nops = std::cmp::max(
        0,
        -(worst_interloop_slack(schedule, starts, bb0_len) + (schedule.len() - bb0_len) as i32),
    );
    for _ in 0..required_nops {
        let mut slot = ScheduleSlot::new(schedule.len());
        slot.bb = Some(BasicBlockSource::BB1);
        schedule.push(slot);
    }
    bb1_len_preadj
}

pub fn finish_bb1_nopip_schedule(
    mov_insts: Vec<DepInst>,
    starts: &mut HashMap<usize, usize>,
    schedule: &mut Vec<ScheduleSlot>,
    bb0_len: usize,
) {
    for mov_inst in mov_insts {
        asap_local(starts, mov_inst, schedule);
    }
    // remove extra slots generated by the scheduler
    prune_schedule(schedule);
    let loop_instr = DepInst { inst: crate::isa::Inst::gen_loop(false, bb0_len), bb: BasicBlockSource::BB1, dep1: crate::analysis::Dep::None, dep2: crate::analysis::Dep::None, stage: 0, pred: None};
    schedule_single(schedule.last_mut().unwrap(), loop_instr);
}

fn calc_ii_lowerbound(insts: &Vec<DepInst>) -> usize {
    let mut counts: HashMap<ExecutionUnit, usize> = HashMap::new();
    for inst in insts {
        let count = counts.entry(inst.inst.opcode.eu_type()).or_insert(0);
        *count += 1;
    }
    counts
        .iter()
        .map(|(eu, count)| {
            if *count == 0 {
                0
            } else {
                ((eu.latency() as f32) / (*count as f32)).ceil() as usize
            }
        })
        .max()
        .unwrap()
}

fn try_bb1_pip_schedule(
    ii: usize,
    insts: &Vec<DepInst>,
    starts: &mut HashMap<usize, usize>,
    bb0_len: usize,
) -> Option<(Vec<ScheduleSlot>, HashMap<usize,usize>)> {
    let mut stages: HashMap<usize, usize> = HashMap::new();
    let mut schedule: Vec<ScheduleSlot> = Vec::with_capacity(ii);
    for i in 0..ii {
        let mut slot = ScheduleSlot::new(bb0_len + i);
        slot.bb = Some(BasicBlockSource::BB1);
        schedule.push(slot);
    }

    // let mut loop_instr: Option<DepInst> = None;
    for inst in insts {
        if inst.inst.opcode.eu_type() == ExecutionUnit::Branch {
            // loop_instr = Some(inst.clone());
            break;
        }
        match asap_local_pip(starts, &mut stages, inst, &mut schedule, bb0_len, ii) {
            Ok(_) => {}
            Err(_) => {
                // println!("Failed schedule!");
                // for slot in schedule {
                //     println!("{}", slot);
                // }
                return None;
            }
        }
    }
    Some((schedule, stages))
}

fn bb1_pip_schedule(
    insts: Vec<DepInst>,
    starts: &HashMap<usize, usize>,
    schedule: &mut Vec<ScheduleSlot>,
    bb0_len: usize,
) -> Option<HashMap<usize,usize>> {
    let mut ii = calc_ii_lowerbound(&insts);
    let mut limit = 0;
    let mut stages: Option<HashMap<usize,usize>>  = None;
    loop {
        if limit > 100 {
            panic!("Loop.pip scheduling appears to be making no progress");
        }
        let mut starts = starts.clone();
        limit += 1;
        let tried_schedule = try_bb1_pip_schedule(ii, &insts, &mut starts, bb0_len);
        if tried_schedule.is_none() {
            ii += 1;
            continue;
        }
        let (mut bb1_schedule, stages_result) = tried_schedule.unwrap();
        let wis = worst_interloop_slack(&bb1_schedule, &starts, 0);
        if wis + (ii as i32) >= 0 {
            stages = Some(stages_result);
            schedule.append(&mut bb1_schedule);
            break;
        } else {
            // TODO could be redundant if you know the difference, alternatively the algorithm might make it work anyway?
            ii += 1;
        }
    }
    // Assuming the last instruction would be a loop
    // The last slot in the schedule should be free here
    stages
}

pub fn loop_schedule(program: AnalyzedProgram, pipeline: bool) -> ScheduledProgram {
    let mut schedule: Vec<ScheduleSlot> = Vec::new();
    let mut starts: HashMap<usize, usize> = HashMap::new();

    flat_block_schedule(program.bb0, &mut starts, &mut schedule);
    prune_schedule(&mut schedule);
    if program.bb1.is_empty() {
        return ScheduledProgram {
            bb0: schedule,
            bb1: Vec::new(),
            bb2: Vec::new(),
            starts: starts,
            bb1_len_preadj: None,
            stages: None
        }
    }
    handle_bb0_bubble(&program.bb1, &mut starts, &mut schedule);
    let bb0_len = schedule.len();
    let mut stages = None;
    let bb1_len_preadj = if pipeline {
        stages = bb1_pip_schedule(program.bb1, &mut starts, &mut schedule, bb0_len);
        None
    } else {
        Some(bb1_nopip_schedule(program.bb1, &mut starts, &mut schedule, bb0_len))
    };
    let bb1_len = schedule.len() - bb0_len;

    flat_block_schedule(program.bb2, &mut starts, &mut schedule);
    prune_schedule(&mut schedule);

    let mut bb1 = schedule.split_off(bb0_len);
    let bb2 = bb1.split_off(bb1_len);
    ScheduledProgram {
        bb0: schedule,
        bb1,
        bb2,
        starts,
        bb1_len_preadj,
        stages
    }
}
