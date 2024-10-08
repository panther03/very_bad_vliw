use analysis::{trace_to_basicblocks,dep_analysis};
use analysis::AnalyzedProgram;
use assembler::{assemble, assemble_ap};
use isa::{Label, Operand};
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
mod scheduling;
mod assembler;

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

fn fix_addresses(sp: &mut ScheduledProgram) {
    for bundle in sp.schedule.iter_mut() {
        bundle.addr = bundle.addr * 16;
        for inst in bundle.valid_insts_mut() {
            if let Label::SrcAddrSpace(l) = inst.inst.label {
                let new_addr = sp.starts.get(&l)
                    .unwrap_or_else(|| panic!("Could not find new label for: {} (inst addr = {})", l, inst.inst.addr));
                inst.inst.label = Label::DstAddrSpace(*new_addr * 16);
            }
            if let Operand::Immediate(mut imm) = inst.inst.src2 {
                if let Some(offset) = inst.inst.offset {
                    imm += offset;
                }
                if let Label::DstAddrSpace(d) = inst.inst.label {
                    imm -= d as i64;
                }
                inst.inst.src2 = Operand::Immediate(imm);
            }
        }
    }
}



fn core(inp_json_path: &Path, args: &Args) -> String {
    let mut trace = read_trace(inp_json_path);
    let orig_size = trace.len() * 4;
    if !args.skip_vliw {
        // remove nops
        //trace = trace.into_iter().filter(|i| i.opcode != Opcode::NOP).collect();
        label_auipc(&mut trace);
    }
    let ap_insns = trace_to_basicblocks(trace).into_iter().map(|bb| dep_analysis(bb)).collect();
    let ap = AnalyzedProgram {
        bbs: ap_insns
    };
    if !args.skip_vliw {  
        let mut sp = schedule_program(ap);
        fix_addresses(&mut sp);
        if !args.skip_assemble {
            assemble(&sp, orig_size, args.bytes_hex)
        } else {
            format!("{}", sp)
        }
    } else {
        if !args.skip_assemble {
            assemble_ap(&ap, args.bytes_hex, args.disassembly)
        } else { 
            format!("{}", ap)
        }
    }
}

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    // Input ASM file (STDIN works)
    inpasm: String,

    // Output file (default is STDOUT)
    #[arg(short,long,default_value="STDOUT")]
    out: String,

    #[arg(short='a',long)]
    skip_assemble: bool,

    #[arg(short='b',long)]
    bytes_hex: bool,

    #[arg(short='d',long)]
    disassembly: bool,

    #[arg(short='v',long)]
    skip_vliw: bool,
}

fn main() {
    let args = Args::parse();
    let inp_asm_path = Path::new(&args.inpasm);
    //let out_asm_path = Path::new(&out_asm_path);
    
    let out_insns = core(inp_asm_path,&args);
        
    if &args.out == "STDOUT" {
        println!("{}", out_insns);
    } else {
        let out_path = Path::new(&args.out);
        std::fs::write(out_path, out_insns).unwrap();
    }
}
