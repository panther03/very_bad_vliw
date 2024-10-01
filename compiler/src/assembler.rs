use crate::{analysis::{AnalyzedBasicBlock, AnalyzedProgram, DepInst}, isa::{Inst, InstParseFormat, Label, Opcode, Operand}, scheduling::{schedule_program, ScheduledProgram}};

fn parse_i_format(inst: &Inst) -> Result<u32, String> {
    let mut word = 0x0;
    let Operand::Immediate(imm) = inst.src2 else {return Err(String::from("I format should have immediate in src2")); };
    let mut imm = imm as i32;
    if let Some(offset) = inst.offset {
        imm += offset as i32;
    }
    if let Label::DstAddrSpace(d) = inst.label {
        imm += (d * 16) as i32;
    }
    
    word |= inst.opcode.opcode_bits();
    word |= inst.dest.unwrap_gpr() << 7;
    word |= inst.opcode.funct3() << 12;
    word |= inst.src1.unwrap() << 15;
    word |= (imm as u32) << 20;
    Ok(word)
}

fn bits(word: u32, start: u32, end: u32) -> u32 {
    return (word & ((1 << (end+1)) - 1)) >> start
}

fn get_offset_from_label(label: &Label, addr: usize) -> Result<i32,String> {
    match label {
        Label::DstAddrSpace(label) => {
            Ok(((*label as i32) - (addr as i32)) * 16)
        }
        Label::SrcAddrSpace(label) => {
            Ok((*label as i32) - (addr as i32))
        }
        _ => Err(String::from("jump instruction should have address in destination address space"))
    }
}

fn assemble_insn(inst: &Inst, addr: usize) -> Result<u32, String> {
    let mut word = 0x0;
    match inst.opcode.parse_format() {
        InstParseFormat::R => {
            word |= inst.opcode.opcode_bits();
            word |= inst.dest.unwrap_gpr() << 7;
            word |= inst.opcode.funct3() << 12;
            word |= inst.src1.unwrap() << 15;
            word |= inst.src2.unwrap_gpr() << 20;
            word |= inst.opcode.funct7() << 25;
        },
        InstParseFormat::I => return parse_i_format(inst),
        InstParseFormat::S => {
            word |= inst.opcode.opcode_bits();
            let imm = inst.offset.unwrap() as u32;
            word |= (imm & 31) << 7;
            word |= inst.opcode.funct3() << 12;
            word |= inst.src2.unwrap_gpr() << 15;
            word |= inst.src1.unwrap() << 20;
            word |= (imm >> 5) << 25;
        }
        InstParseFormat::L => {
            word |= inst.opcode.opcode_bits();
            word |= inst.dest.unwrap_gpr() << 7;
            word |= inst.opcode.funct3() << 12;
            word |= inst.src1.unwrap() << 15;
            let imm = inst.offset.unwrap() as u32;
            word |= (imm as u32) << 20;
        }
        InstParseFormat::B => {
            word |= inst.opcode.opcode_bits();
            let label = get_offset_from_label(&inst.label, addr)? as u32;
            //if label > ((1<<13)-1) { return Err(format!("label too large: {}", label)) }
            word |= (label & 30 | ((label >> 11) & 0x1)) << 7;
            word |= (label & 30 | ((label >> 11) & 0x1)) << 7;
            word |= inst.opcode.funct3() << 12;
            word |= inst.src2.unwrap_gpr() << 15;
            word |= inst.src1.unwrap() << 20;
            word |= (bits(label, 5, 10) 
                | (bits(label, 12, 12) << 6)) << 25;
        }
        InstParseFormat::J => {
            let label = get_offset_from_label(&inst.label, addr)? as u32;
            //if label > ((1<<21)-1) { return Err(format!("label too large: {}", label))}
            word |= inst.opcode.opcode_bits();
            word |= inst.dest.unwrap_gpr() << 7;
            word |= (bits(label, 20, 20) << 19 
                | bits(label, 1, 10) << 9
                | bits(label, 11, 11) << 8
                | bits(label, 12, 19)) << 12;
        }
        _ => { 
            match inst.opcode {
                Opcode::RET => { word = 0x00008067; },
                Opcode::LI => { return parse_i_format(inst)},
                Opcode::MOV => {
                    let mut inst2 = inst.clone();
                    inst2.src1 = Some(inst2.src2.unwrap_gpr());
                    inst2.src2 = Operand::Immediate(0);
                    return parse_i_format(&inst2)
                }
                Opcode::AUIPC | Opcode::LUI => {
                    word |= inst.opcode.opcode_bits();
                    word |= inst.dest.unwrap_gpr() << 7;
                    let Operand::Immediate(imm) = inst.src2 else {return Err(String::from("auipc should have immediate offset"))};
                    word |= (imm as u32) << 12; 
                },
                _ => {unreachable!()}
            }
        }
    }
    Ok(word)
}

fn le_word(word: u32) -> String {
    let mut output = String::new();
    output.push_str(format!("{:02X} ", (word) & 0xFF).as_str());
    output.push_str(format!("{:02X} ", (word >> 8) & 0xFF).as_str());
    output.push_str(format!("{:02X} ", (word >> 16) & 0xFF).as_str());
    output.push_str(format!("{:02X}\n", (word >> 24) & 0xFF).as_str());
    output
}

pub fn assemble (sp: &ScheduledProgram, orig_size: usize, bytes_hex: bool) -> String { 
    let mut output = String::new();

    let offset = (sp.schedule.len()*16 - orig_size + 16) as i32;
    assert!(offset > 0);
    output.push_str(format!("@0\n{}\n0\n0\n0\n", offset).as_str());
    for bundle in sp.schedule.iter() {
        for inst in bundle.insts() {
            let word = if let Some(inst) = inst {
                match assemble_insn(&inst.inst, bundle.addr) {
                    Ok(w) => w,
                    Err(e) => panic!("Can't assemble instruction {}:\n {}", &inst.inst, e),
                }
            } else {
                0
            };
            if bytes_hex {
                output.push_str(&le_word(word));
            } else {
                output.push_str(format!("{:08x}\n", word).as_str());
            }
        }
    }
    output.push_str(format!("@{:x}", sp.schedule.len()*16 + 16).as_str());
    output
}

pub fn assemble_ap_single(inst: &Inst, bytes_hex: bool, disassembly: bool, output: &mut String) {
    let word = match assemble_insn(&inst, inst.addr) {
        Ok(w) => w,
        Err(e) => panic!("Can't assemble instruction {}:\n {}", &inst, e),
    };
    if disassembly {
        let inst_str = format!("{}", inst);
        let gap = std::cmp::max(22-inst_str.len(), 0);
        output.push_str(inst_str.as_str());
        for _ in 0..gap {
            output.push(' ');
        }
        output.push_str(" | ");
    }
    if bytes_hex {
        output.push_str(&le_word(word));
    } else {
        output.push_str(format!("{:08x}\n", word).as_str());
    }
}

pub fn assemble_ap (ap: &AnalyzedProgram, bytes_hex: bool, disassembly: bool) -> String { 
    let mut output = String::new();

    for bb in ap.bbs.iter() {
        for inst in bb.insns.iter() {
            assemble_ap_single(&inst.inst, bytes_hex, disassembly, &mut output);
        }
        if let Some(cf_insn) = &bb.cf_insn {
            assemble_ap_single(&cf_insn.inst, bytes_hex, disassembly, &mut output);
        }
    }

    output
}