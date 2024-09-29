use crate::isa::{Inst,Opcode, Operand};
use crate::analysis::{Dep, DepInst};
use crate::scheduling::{flat_block_schedule, prune_schedule, ScheduleSlot, ScheduledProgram};

pub fn finalize(mut program: ScheduledProgram) -> Vec<ScheduleSlot> {
    let stages = program.stages.unwrap();

    let mut new_insts: Vec<DepInst> = Vec::new();
    let ec_val = stages.values().max().unwrap();
    let ec_instr = Inst {addr: 0, opcode: Opcode::MOV, dest: Operand::Ec, src1: None, src2: Operand::Immediate(*ec_val as i64), offset: None};
    // TODO: unsure why these instructions can't just go in the first free slot in the schedule,
    // but the assignment specifies it this way, so
    new_insts.push(DepInst::new(ec_instr, crate::analysis::BasicBlockSource::BB0).with_dep(Dep::Fixed(program.bb0.len()-1)));
    let pred_instr = Inst {addr: 0, opcode: Opcode::MOV, dest: Operand::Predicate(32), src1: None, src2: Operand::PredicateVal(true), offset: None};
    new_insts.push(DepInst::new(pred_instr, crate::analysis::BasicBlockSource::BB0).with_dep(Dep::Fixed(program.bb0.len()-1)));
    flat_block_schedule(new_insts, &mut program.starts, &mut program.bb0);
    prune_schedule(&mut program.bb0);

    for slot in program.bb1.iter_mut() {
        for inst in slot.valid_insts_mut() {
            inst.pred = Some(32 + inst.stage as u32);
        }
    }

    let bb0_len = program.bb0.len();
    let last_slot = program.bb1.last_mut().unwrap();
    last_slot.branch = Some(DepInst::new(Inst::gen_loop(true, bb0_len), crate::analysis::BasicBlockSource::BB1));
    
    program.bb0.append(&mut program.bb1);
    program.bb0.append(&mut program.bb2);
    program.bb0
}