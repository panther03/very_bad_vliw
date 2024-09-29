use std::{collections::{HashMap, HashSet}, hash::Hash, fmt};

use crate::{
    analysis::{Dep, DepInst, Producer},
    isa::{Inst, Opcode, Operand},
    scheduling::{finish_bb1_nopip_schedule, flat_block_schedule, prune_schedule, ScheduleSlot, ScheduledProgram},
};
struct RegisterRemapping {
    remappings: HashMap<Producer, u32>,
    renamed_tracker: HashSet<usize>,
    fresh_cnt: u32,
    pipelined_cnt: u32,
    stages: u32
}

impl fmt::Display for RegisterRemapping {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (prod, reg) in self.remappings.iter() {
            write!(f, "{},x{} -> x{}\n", prod.addr, prod.reg, reg)?;
        }
        Ok(())
    }
}

impl RegisterRemapping {
    fn new() -> Self {
        RegisterRemapping {
            remappings: HashMap::new(),
            renamed_tracker: HashSet::new(),
            fresh_cnt: 0,
            pipelined_cnt: 0,
            stages: 0
        }
    }

    fn new_pipelined(stages: u32) -> Self {
        RegisterRemapping {
            remappings: HashMap::new(),
            renamed_tracker: HashSet::new(),
            fresh_cnt: 0,
            pipelined_cnt: 0,
            stages: stages
        }
    }

    fn fresh(&mut self) -> u32 {
        if self.fresh_cnt >= 32 {
            panic!("ran out of non-pipelined registers")
        }
        self.fresh_cnt += 1;
        self.fresh_cnt
    }

    fn fresh_pipelined(&mut self) -> u32 {
        let fresh = self.pipelined_cnt + 32;
        self.pipelined_cnt += self.stages + 1; 
        fresh
    }

    fn get(&self, prod: &Producer) -> Option<u32> {
        self.remappings.get(prod).copied()
    }

    fn insert(&mut self, prod: Producer, reg: u32, addr: usize) {
        self.remappings.insert(prod, reg);
        self.renamed_tracker.insert(addr);
    }

    fn already_renamed(&self, addr: usize) -> bool{
        self.renamed_tracker.contains(&addr)
    }
}

fn rename_operand(dep: &Dep, remappings: &mut RegisterRemapping) -> u32 {
    let prod = match dep {
        Dep::Local(p) => p,
        Dep::Interloop(_, p) => p,
        Dep::Invariant(p) => p,
        Dep::Postloop(p) => p,
        Dep::Fixed(_) | Dep::None => {
            // this function is only called if the operand has a register,
            // so since there's no dependency it means there's no remapping and 
            // it hasn't been used; we assign a new one
            // no remapping for this register; assign fresh one
            return remappings.fresh()
        }
    };
    if let Some(new_reg) = remappings.get(prod) {
        new_reg
    } else {
        panic!("there is a dependency for this operand, but it was never assigned a fresh register?");
    }
}

impl DepInst {
    fn fresh_rename(&mut self, remappings: &mut RegisterRemapping) {
        if let Operand::Gpr(rd) = self.inst.dest {
            let fresh = remappings.fresh();
            remappings.insert(Producer { addr: self.inst.addr, eu: self.inst.opcode.eu_type(), reg: rd}, fresh, self.inst.addr);
            self.inst.dest = Operand::Gpr(fresh)
        }
    }

    fn fresh_rename_pip(&mut self, remappings: &mut RegisterRemapping) {
        if let Operand::Gpr(rd) = self.inst.dest {
            let fresh = remappings.fresh_pipelined();
            remappings.insert(Producer { addr: self.inst.addr, eu: self.inst.opcode.eu_type(), reg: rd}, fresh, self.inst.addr);
            self.inst.dest = Operand::Gpr(fresh)
        }
    }
}

fn gen_mov_fix_interloop(local_prod: &Producer, bb0_prod: &Producer, rrmap: &RegisterRemapping, bb1_end: usize) -> DepInst {  
    // Source is the value produced in the loop
    let renamed_src = rrmap.get(local_prod).unwrap();
    // Destination is the value produced in bb0, which we want to update for the next iteration
    let renamed_dest = rrmap.get(bb0_prod).unwrap();

    let inst = Inst { opcode: Opcode::MOV, addr: 0, dest: Operand::Gpr(renamed_dest), src1: None, src2: Operand::Gpr(renamed_src), offset: None };
    DepInst { inst: inst, bb: crate::analysis::BasicBlockSource::BB1, dep1: Dep::Fixed(bb1_end-1), dep2: Dep::Local(local_prod.clone()), stage: 0, pred: None }
}

pub fn reg_alloc_nopip(mut program: ScheduledProgram) -> Vec<ScheduleSlot> {
    let mut rrmap: RegisterRemapping = RegisterRemapping::new();

    // Phase 1: allocate fresh registers to each variable
    fn phase1(slots: &mut Vec<ScheduleSlot>, rrmap: &mut RegisterRemapping) {
        for slot in slots.iter_mut() {
            for inst in slot.valid_insts_mut() {
                inst.fresh_rename(rrmap);
            }
        }
    }

    phase1(&mut program.bb0, &mut rrmap);
    phase1(&mut program.bb1, &mut rrmap);
    phase1(&mut program.bb2, &mut rrmap);

    // Phase 2: rename operands according to phase1
    // don't worry about loop dependencies right now
    fn phase2(slots: &mut Vec<ScheduleSlot>, rrmap: &mut RegisterRemapping) {
        for slot in slots.iter_mut() {
            for inst in slot.valid_insts_mut() {
                for (dep, reg) in inst.all_deps() {
                    *reg = rename_operand(dep, rrmap);
                }
            }
        }
    }

    phase2(&mut program.bb0, &mut rrmap);
    phase2(&mut program.bb1, &mut rrmap);
    phase2(&mut program.bb2, &mut rrmap);

    // phase 3
    let mut new_movs: HashMap<(u32,u32),DepInst> = HashMap::new();
    for slot in program.bb1.iter() {
        for inst in slot.valid_insts() {
            if let Dep::Interloop(loc, bb0_prod) = &inst.dep1 {
                let new_mov = gen_mov_fix_interloop(loc, bb0_prod, &rrmap, program.bb1_len_preadj.unwrap());
                new_movs.insert((new_mov.inst.dest.unwrap_gpr(), new_mov.inst.src2.unwrap_gpr()), new_mov);
            }
            if let Dep::Interloop(loc, bb0_prod) = &inst.dep2 {
                let new_mov = gen_mov_fix_interloop(loc, bb0_prod, &rrmap, program.bb1_len_preadj.unwrap());
                new_movs.insert((new_mov.inst.dest.unwrap_gpr(), new_mov.inst.src2.unwrap_gpr()), new_mov);
            }
        }
    }
    if program.bb1.len() > 0 {
        let bb0_len = program.bb0.len();
        program.bb0.append(&mut program.bb1);
        let new_movs: Vec<DepInst> = new_movs.values().cloned().collect();
        finish_bb1_nopip_schedule(new_movs, &mut program.starts, &mut program.bb0, bb0_len);
    }

    program.bb0.append(&mut program.bb2);
    program.bb0
}

fn fix_bb0_invariant(p: &Producer, starts: &HashMap<usize,usize>, bb0: &mut Vec<ScheduleSlot>, rrmap: &mut RegisterRemapping) -> Result<(),()> {
    let start = starts.get(&p.addr).unwrap();
    let slot = bb0.get_mut(*start).unwrap();
    for inst in slot.valid_insts_mut() {
        if inst.inst.addr == p.addr {
            inst.fresh_rename(rrmap);
            return Ok(());
        }
    }
    Err(())
}

fn fix_bb0_interloop(p: &Producer, xs: u32, st_s: u32, starts: &HashMap<usize,usize>, bb0: &mut Vec<ScheduleSlot>, rrmap: &mut RegisterRemapping) -> Result<(),()> {
    let start = starts.get(&p.addr).unwrap();
    let slot = bb0.get_mut(*start).unwrap();
    for inst in slot.valid_insts_mut() {
        if inst.inst.addr == p.addr {
            let new_dst = xs - st_s + 1;
            inst.inst.dest = Operand::Gpr(new_dst);
            rrmap.renamed_tracker.insert(inst.inst.addr);
            return Ok(());
        }
    }
    Err(())
}

pub fn reg_alloc_pip(mut program: ScheduledProgram) -> ScheduledProgram {
    let stages = program.stages.as_ref().unwrap();
    let num_stages = stages.values().max().unwrap() + 1;
    let mut rrmap: RegisterRemapping = RegisterRemapping::new_pipelined(num_stages as u32);

    // phase 1: assign each producer in bb1 a fresh register
    for slot in program.bb1.iter_mut() {
        for inst in slot.valid_insts_mut() {
            inst.fresh_rename_pip(&mut rrmap);
        }
    }

    // phase 2/3: 
    // assign each loop invariant producer a fresh register
    // fix all the operands in the loop
    for slot in program.bb1.iter_mut() {
        for inst in slot.valid_insts_mut() {
            let st_d = inst.stage;
            for (dep, reg) in inst.all_deps() {
                match dep {
                    Dep::Invariant(p) => {
                        if let None = rrmap.get(p) {
                            fix_bb0_invariant(p, &program.starts, &mut program.bb0, &mut rrmap).unwrap();
                        }
                        *reg = rrmap.get(p).unwrap();
                    },
                    Dep::Local(p) => {
                        let xs = rrmap.get(p).unwrap();
                        let st_s = stages.get(&p.addr).unwrap();
                        *reg = xs + (st_d - st_s) as u32;
                    }
                    Dep::Interloop(bb1_p, bb0_p) => {
                        let xs = rrmap.get(bb1_p).unwrap();
                        let st_s = stages.get(&bb1_p.addr).unwrap();
                        if let None = rrmap.get(bb0_p) {
                            fix_bb0_interloop(bb0_p, xs, *st_s as u32, &program.starts, &mut program.bb0, &mut rrmap).unwrap();
                        }
                        *reg = xs + (st_d - st_s) as u32 + 1;
                    }
                    Dep::None => {
                        
                    }
                    _ => {}
                }
            }
        }
    }
    
    // phase 4: fix remaining problems in bb0/bb2
    for slot in program.bb0.iter_mut() {
        pip_phase4(slot, &mut rrmap, stages, num_stages);
    }

    // phase 4.5: fix the remaining None dependencies in BB1
    // they should have fresh values, but it needs to be done after BB0
    // so that it is equivalent to the provided spec
    for slot in program.bb1.iter_mut() {
        for inst in slot.valid_insts_mut() {
            for (dep, reg) in inst.all_deps() {
                if let Dep::None = dep {
                    *reg = rrmap.fresh();
                }                
            }
        }
    }

    for slot in program.bb2.iter_mut() {
        pip_phase4(slot, &mut rrmap, stages, num_stages);
    }
    program
}

fn pip_phase4(slot: &mut ScheduleSlot, rrmap: &mut RegisterRemapping, stages: &HashMap<usize, usize>, num_stages: usize) {
    for inst in slot.valid_insts_mut() {
        // first process this instruction's destination
        // rename if not already in rename table
        if let Operand::Gpr(rd) = inst.inst.dest {
            if !rrmap.already_renamed(inst.inst.addr) {
                let prod = Producer { addr: inst.inst.addr, eu: inst.inst.opcode.eu_type(), reg: rd};
                let fresh = rrmap.fresh();
                rrmap.insert(prod, fresh, inst.inst.addr);
                inst.inst.dest = Operand::Gpr(fresh);
            }
        }
        // second process operands
        for (dep, reg) in inst.all_deps() {
            match dep {
                Dep::Postloop(p) => {
                    let xs = rrmap.get(p).unwrap();
                    let st_s = stages.get(&p.addr).unwrap();
                    *reg = xs + (num_stages - *st_s) as u32 - 1;
                }
                _ => { *reg = rename_operand(dep, rrmap); }
            }
        }
    }
}