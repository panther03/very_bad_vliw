use scheduling::{loop_schedule, ScheduleSlot};
use serde_json;
use std::fs;
use std::path::Path;

use isa::Inst;

mod analysis;
mod isa;
mod regalloc;
mod scheduling;
mod finalization;

fn read_trace(inp_json_path: &Path) -> Result<Vec<Inst>, String> {
    let inp_json_string = fs::read_to_string(inp_json_path)
        .map_err(|err| format!("Error opening trace file: {}", err))?;

    let value: serde_json::Value =
        serde_json::from_str(&inp_json_string).map_err(|e| format!("Error parsing JSON: {}", e))?;

    if let serde_json::Value::Array(arr) = value {
        let insts: Result<Vec<Inst>, String> = arr
            .into_iter()
            .enumerate()
            .map(|(i, s)| -> Result<Inst, String> {
                // TODO once again proper error escalation
                let line: String = serde_json::from_value(s)
                    .map_err(|e| format!("Error parsing JSON element: {}", e))?;
                Inst::from_str(&line, i)
            })
            .collect();

        insts
    } else {
        Err("JSON does not contain an array".to_string())
    }
}

pub fn write_json(schedule: Vec<ScheduleSlot>, out_json_path: &Path) -> Result<(), String> {
    let out_json = serde_json::to_string_pretty(&schedule)
        .map_err(|err| format!("Error converting instructions to JSON: {err}"))?;
    std::fs::write(out_json_path, out_json)
        .map_err(|err| format!("Error writing output JSON: {}", err))?;
    Ok(())
}

fn core(inp_json_path: &Path) -> (Vec<ScheduleSlot>, Vec<ScheduleSlot>) {
    let trace = read_trace(inp_json_path).unwrap_or_else(|err| {
        panic!("Error reading input JSON: {}", err);
    });
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
    }    
}

fn main() {
    let inp_json_path = std::env::args().nth(1).expect("No input JSON given!");
    let out_json_path_nopip  = std::env::args().nth(2).expect("No output JSON for non-pipelined given!");
    let out_json_path_pip  = std::env::args().nth(3).expect("No output JSON for pipelined given!");
    let inp_json_path = Path::new(&inp_json_path);
    let out_json_path_nopip = Path::new(&out_json_path_nopip);
    let out_json_path_pip = Path::new(&out_json_path_pip);

    let (nopip_sched, pip_sched) = core(inp_json_path);
    write_json(nopip_sched, out_json_path_nopip)
        .unwrap_or_else(|err| {
            panic!("Error writing output JSON for non-pipelined: {}", err);
        });
    write_json(pip_sched, out_json_path_pip)
        .unwrap_or_else(|err| {
            panic!("Error writing output JSON for pipelined: {}", err);
        });
}
