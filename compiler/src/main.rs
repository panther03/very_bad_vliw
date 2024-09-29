use analysis::{trace_to_basicblocks,dep_analysis};
use analysis::AnalyzedProgram;
use isa::Label;
use scheduling::{ScheduledProgram,schedule_program};
//use scheduling::{loop_schedule, ScheduleSlot};
use serde_json;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

use isa::Inst;
use isa::Opcode;

mod analysis;
mod isa;
//mod regalloc;
mod scheduling;
//mod finalization;

fn read_trace(inp_asm_path: &Path) -> Vec<Inst> {

    let inp_json_string = if inp_asm_path.as_os_str() == "STDIN" {
        io::read_to_string(io::stdin())
        .map_err(|err| format!("Error opening trace file: {}", err)).unwrap()
    } else {
        fs::read_to_string(inp_asm_path)
        .map_err(|err| format!("Error opening trace file: {}", err)).unwrap()
    };
    
    inp_json_string
        .lines()
        .enumerate()
        .map(|(i,s)| Inst::from_str(s, i*4).unwrap()).collect()
}

fn label_auipc(trace: &mut Vec<Inst>) {
    let mut auipc_pc = -1;
    for inst in trace.iter_mut() {
        if auipc_pc >= 0 {
            if inst.opcode != Opcode::ADDI {
                panic!("Expected ADDI offset right after AUIPC! Got {}", inst);
            }
            inst.offset = Some(auipc_pc);
            inst.label = Label::SrcAddrSpace(auipc_pc as usize);
            auipc_pc = -1;
        } else if inst.opcode == Opcode::AUIPC {
            auipc_pc = inst.addr as i64;
        }
    }
}

fn fix_labels(sp: &mut ScheduledProgram) {
    for bundle in sp.schedule.iter_mut() {
        for inst in bundle.valid_insts_mut() {
            if let Label::SrcAddrSpace(l) = inst.inst.label {
                let new_addr = sp.starts.get(&l)
                    .unwrap_or_else(|| panic!("Could not find new label for: {} (inst addr = {})", l, inst.inst.addr));
                inst.inst.label = Label::DstAddrSpace(*new_addr);
            }
        }
    }
}

fn core(inp_json_path: &Path) -> ScheduledProgram {
    let mut trace = read_trace(inp_json_path);
    label_auipc(&mut trace);
    let ap_insns = trace_to_basicblocks(trace).into_iter().map(|bb| dep_analysis(bb)).collect();
    let ap = AnalyzedProgram {
        bbs: ap_insns
    };
    println!("{}", ap);
    let mut sp = schedule_program(ap);
    fix_labels(&mut sp);
    sp
    /*
    let bb_analysis = analysis::basicblock_analysis(&trace);
    let deps_trace = analysis::dep_analysis(trace, &bb_analysis);
    // println!("{}", deps_trace);
    if deps_trace.bb1.len() > 0 {
        let nopip_schedule = loop_schedule(deps_trace.clone(), false);
        let pip_schedule = loop_schedule(deps_trace, true);
        // println!("Non-Pipelined schedule");
        // println!("{}", nopip_schedule);

        // println!("Pipelined schedule (II={})", pip_schedule.bb1.len());
        // println!("{}", pip_schedule);

        let regalloced_nopip = regalloc::reg_alloc_nopip(nopip_schedule);
        let regalloced_pip = regalloc::reg_alloc_pip(pip_schedule);
        let finalized_pip = finalization::finalize(regalloced_pip);
        (regalloced_nopip, finalized_pip)
    } else {
        let schedule = loop_schedule(deps_trace, false);
        // println!("Schedule (no BB1)");
        // println!("{}", schedule);
        let regalloced = regalloc::reg_alloc_nopip(schedule);
        (regalloced.clone(), regalloced)
    }*/
}

fn main() {
    let inp_asm_path = std::env::args().nth(1).expect("No input ASM given! (Use STDIN for standard input)");
    //let out_asm_path  = std::env::args().nth(2).expect("No output ASM given!");
    let inp_asm_path = Path::new(&inp_asm_path);
    //let out_asm_path = Path::new(&out_asm_path);

    let out_insns = core(inp_asm_path);
    /*for (i,bb) in out_insns.into_iter().enumerate() {
        println!("BasicBlock {}", i);
        let out_asm: String = bb.into_iter().map(|i| i.to_string()).collect::<Vec<String>>().join("\n");
        println!("{}", out_asm);
    }*/
    println!("{}", out_insns);
    //let out_asm = format!(".globl _start\n
//_start:\n{}", out_asm);
    //std::fs::write(out_asm_path, out_asm).unwrap();
}
