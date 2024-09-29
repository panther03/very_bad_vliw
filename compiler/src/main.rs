//use scheduling::{loop_schedule, ScheduleSlot};
use serde_json;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

use isa::Inst;

//mod analysis;
mod isa;
//mod regalloc;
//mod scheduling;
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



fn core(inp_json_path: &Path) -> Vec<Inst> {
    let trace = read_trace(inp_json_path);
    trace
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
    let out_asm_path  = std::env::args().nth(2).expect("No output ASM given!");
    let inp_asm_path = Path::new(&inp_asm_path);
    let out_asm_path = Path::new(&out_asm_path);

    let out_insns = core(inp_asm_path);
    let out_asm: String = out_insns.into_iter().map(|i| i.to_string()).collect::<Vec<String>>().join("\n");
    let out_asm = format!(".globl _start\n
_start:\n{}", out_asm);
    std::fs::write(out_asm_path, out_asm).unwrap();
}
