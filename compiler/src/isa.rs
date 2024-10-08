use std::{collections::HashMap, fmt};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Opcode {
    // Arithmetic registers
    ADD,
    SUB,
    XOR,
    OR,
    AND,
    SLL,
    SRL,
    SRA,
    SLT,
    SLTU,
    // Arithmetic immediates
    ADDI,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    SLTI,
    SLTIU,
    // Loads
    LB,
    LH,
    LW,
    LBU,
    LHU,
    // Stores
    SB,
    SH,
    SW,
    // Branches
    J,
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
    // Jump and link
    JAL,
    JALR,
    // Others
    LUI,
    AUIPC,
    // Pseudos
    LI,
    MOV,
    NOP,
    RET,
}

impl Opcode {
    fn from_str(op: &str) -> Result<Self, String> {
        match op {
            "add" => Ok(Self::ADD),
            "sub" => Ok(Self::SUB),
            "xor" => Ok(Self::XOR),
            "or" => Ok(Self::OR),
            "and" => Ok(Self::AND),
            "sll" => Ok(Self::SLL),
            "srl" => Ok(Self::SRL),
            "sra" => Ok(Self::SRA),
            "slt" => Ok(Self::SLT),
            "sltu" => Ok(Self::SLTU),
            "addi" => Ok(Self::ADDI),
            "xori" => Ok(Self::XORI),
            "ori" => Ok(Self::ORI),
            "andi" => Ok(Self::ANDI),
            "slli" => Ok(Self::SLLI),
            "srli" => Ok(Self::SRLI),
            "srai" => Ok(Self::SRAI),
            "slti" => Ok(Self::SLTI),
            "sltiu" => Ok(Self::SLTIU),
            "lb" => Ok(Self::LB),
            "lh" => Ok(Self::LH),
            "lw" => Ok(Self::LW),
            "lbu" => Ok(Self::LBU),
            "lhu" => Ok(Self::LHU),
            "sb" => Ok(Self::SB),
            "sh" => Ok(Self::SH),
            "sw" => Ok(Self::SW),
            "j" => Ok(Self::J),
            "beq" => Ok(Self::BEQ),
            "bne" => Ok(Self::BNE),
            "blt" => Ok(Self::BLT),
            "bge" => Ok(Self::BGE),
            "bltu" => Ok(Self::BLTU),
            "bgeu" => Ok(Self::BGEU),
            "jal" => Ok(Self::JAL),
            "jalr" => Ok(Self::JALR),
            "lui" => Ok(Self::LUI),
            "auipc" => Ok(Self::AUIPC),
            "li" => Ok(Self::LI),
            "mv" => Ok(Self::MOV),
            "nop" => Ok(Self::NOP),
            "ret" => Ok(Self::RET),
            _ => Err(format!("Unrecognized opcode: {}", op))
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::ADD => "add",
            Self::SUB => "sub",
            Self::XOR => "xor",
            Self::OR => "or",
            Self::AND => "and",
            Self::SLL => "sll",
            Self::SRL => "srl",
            Self::SRA => "sra",
            Self::SLT => "slt",
            Self::SLTU => "sltu",
            Self::ADDI => "addi",
            Self::XORI => "xori",
            Self::ORI => "ori",
            Self::ANDI => "andi",
            Self::SLLI => "slli",
            Self::SRLI => "srli",
            Self::SRAI => "srai",
            Self::SLTI => "slti",
            Self::SLTIU => "sltiu",
            Self::LB => "lb",
            Self::LH => "lh",
            Self::LW => "lw",
            Self::LBU => "lbu",
            Self::LHU => "lhu",
            Self::SB => "sb",
            Self::SH => "sh",
            Self::SW => "sw",
            Self::J => "j",
            Self::BEQ => "beq",
            Self::BNE => "bne",
            Self::BLT => "blt",
            Self::BGE => "bge",
            Self::BLTU => "bltu",
            Self::BGEU => "bgeu",
            Self::JAL => "jal",
            Self::JALR => "jalr",
            Self::LUI => "lui",
            Self::AUIPC => "auipc",
            Self::LI => "li",
            Self::MOV => "mv",
            Self::NOP => "nop",
            Self::RET => "ret",
        }
    }

    pub fn eu_type(&self) -> ExecutionUnit {
        match self {
            Self::ADD | Self::SUB | Self::XOR | Self::OR | 
            Self::AND | Self::SLL | Self::SRL | Self::SRA |
            Self::SLT | Self::SLTU | Self::ADDI | Self::XORI |
            Self::ORI | Self::ANDI | Self::SLLI | Self::SRLI |
            Self::SRAI | Self::SLTI | Self::SLTIU |
            Self::LUI | Self::AUIPC | Self::MOV | Self::LI => ExecutionUnit::ALU,
    
            Self::BEQ | Self::BNE | Self::BLT |
            Self::BGE | Self::BLTU | Self::BGEU |
            Self::J | Self::JAL | Self::RET | Self::JALR => ExecutionUnit::Branch,
            Self::LB | Self::LH | Self::LW | Self::LBU |
            Self::LHU | Self::SB | Self::SH | Self::SW => ExecutionUnit::Mem,
            _ => panic!("unrecognized execution unit: {:#?}", self),
        }
    }

    pub fn parse_format(&self) -> InstParseFormat {
        match self {
            Self::ADD | Self::SUB | Self::XOR | Self::OR | 
            Self::AND | Self::SLL | Self::SRL | Self::SRA |
            Self::SLT | Self::SLTU => InstParseFormat::R,
            Self::ADDI | Self::XORI | Self::ORI | Self::ANDI |
            Self::SLLI | Self::SRLI | Self::SRAI | Self::SLTI |
            Self::SLTIU => InstParseFormat::I,
            Self::BEQ | Self::BNE | Self::BLT |
            Self::BGE | Self::BLTU | Self::BGEU => InstParseFormat::B,
            Self::LB | Self::LH | Self::LW | Self::LBU | Self::LHU => InstParseFormat::L,
            Self::SB | Self::SH | Self::SW => InstParseFormat::S,
            Self::J | Self::JAL => InstParseFormat::J,
            Self::LUI | Self::AUIPC | Self::MOV | Self::LI => InstParseFormat::MOV,
            Self::NOP | Self::RET => InstParseFormat::NOP,
            Self::JALR => panic!("jalr will make the universe explode"),
        }
    }

    pub fn opcode_bits(&self) -> u32 {
        match self.parse_format() {
            InstParseFormat::R => 0b0110011,
            InstParseFormat::I => 0b0010011,
            InstParseFormat::L => 0b0000011,
            InstParseFormat::S => 0b0100011,
            InstParseFormat::B => 0b1100011,
            InstParseFormat::J => {
                match self {
                    Self::J => 0b1101111,
                    Self::JAL => 0b1101111,
                    Self::JALR => 0b1100111,
                    _ => unreachable!()
                }
            },
            _ => {
                match self {
                    Self::LUI => 0b0110111,
                    Self::AUIPC => 0b0010111,
                    Self::MOV => 0b0010011, // TODO
                    Self::LI => 0b0010011,
                    Self::RET => 0b1100111,
                    _ => unreachable!(),
                }
            }
        }
    }

    pub fn funct3(&self) -> u32 {
        match self {
            Self::ADD => 0x0,
            Self::SUB => 0x0,
            Self::XOR => 0x4,
            Self::OR => 0x6,
            Self::AND => 0x7,
            Self::SLL => 0x1,
            Self::SRL => 0x5,
            Self::SRA => 0x5,
            Self::SLT => 0x2,
            Self::SLTU => 0x3,
            Self::ADDI => 0x0,
            Self::XORI => 0x4,
            Self::ORI => 0x6,
            Self::ANDI => 0x7,
            Self::SLLI => 0x1,
            Self::SRLI => 0x5,
            Self::SRAI => 0x5,
            Self::SLTI => 0x2,
            Self::SLTIU => 0x3,
            Self::LB => 0x0,
            Self::LH => 0x1,
            Self::LW => 0x2,
            Self::LBU => 0x4,
            Self::LHU => 0x5,
            Self::SB => 0x0,
            Self::SH => 0x1,
            Self::SW => 0x2,
            Self::BEQ => 0x0,
            Self::BNE => 0x1,
            Self::BLT => 0x4,
            Self::BGE => 0x5, 
            Self::BLTU => 0x6,
            Self::BGEU => 0x7,
            Self::JAL => 0x0,
            Self::JALR => 0x0,
            Self::RET => 0x0,
            Self::LUI => 0x0,
            Self::AUIPC => 0x0,
            // TODO
            Self::LI => 0x0,
            Self::MOV => 0x0,
            _ => unreachable!(),
        }
    }

    pub fn funct7(&self) -> u32 {
        match self {
            Self::SUB => 0x20,
            Self::SRA => 0x20,
            _ => 0x0,
        }
    }

    pub fn is_control_flow(&self) -> bool {
        match self {
            Self::BEQ | Self::BNE | Self::BLT |
            Self::BGE | Self::BLTU | Self::BGEU |
            Self::J | Self::JAL | Self::JALR | Self::RET => true,
            _ => false
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstParseFormat {
    R,
    I,
    L,
    S,
    B,
    J,
    MOV,
    NOP
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExecutionUnit {
    ALU,
    Mult,
    Mem,
    Branch,
}

impl ExecutionUnit {
    pub fn latency(&self) -> usize {
        match self {
            Self::ALU => 1,
            Self::Mult => 3,
            Self::Mem => 1,
            Self::Branch => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Operand {
    Gpr(u32),
    Predicate(u32),
    Immediate(i64),
    PredicateVal(bool),
    Lc,
    Ec,
    None,
}

impl Operand {
    fn from_str(op: &str) -> Result<Self, String> {
        let mut chars = op.chars();
        match chars.next() {
            Some('x') => {
                let Ok(reg) = chars.collect::<String>().parse::<u32>() else { return Err(format!("Register parse error: {}", op)) };
                if reg > 95 {
                    Err(format!("Unrecognized architectural register {}", reg))
                } else {
                    Ok(Self::Gpr(reg))
                }
            }
            Some('p') => {
                let Ok(reg) = chars.collect::<String>().parse::<u32>() else { return Err(format!("Register parse error: {}", op)) };
                if reg > 95 {
                    Err(format!("Unrecognized predicate register {}", reg))
                } else {
                    Ok(Self::Predicate(reg))
                }
            }
            Some('f') | Some('t') => {
                let Ok(pval) = op.parse::<bool>() else { return Err(format!("Predicate value parse error: {}", op)) };
                Ok(Self::PredicateVal(pval))
            }
            Some('L') => {
                if chars.next() == Some('C') { 
                    Ok(Self::Lc)
                } else {
                    Err(format!("Unrecognized operand: {}", op))
                }
            }
            Some('E') => {
                if chars.next() == Some('C') { 
                    Ok(Self::Ec)
                } else {
                    Err(format!("Unrecognized operand: {}", op))
                }
            }
            _ => {
                let imm = match op.parse::<i64>() {
                    Ok(imm) => imm,
                    Err(_) => {
                        if op.starts_with("0x") {
                            i64::from_str_radix(&op[2..], 16).map_err(|e| format!("Hex parse error: {}", e))?
                        } else {
                            return Err(format!("Unrecognized token: {}", op))
                        }
                    }
                };
                Ok(Self::Immediate(imm))
            }
        }
    }

    pub fn unwrap_gpr(&self) -> u32 {
        match self {
            Self::Gpr(r) => *r,
            _ => panic!("Operand is not a GPR")
        }
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Ec => write!(f, "EC"),
            Operand::Lc => write!(f, "LC"),
            Operand::Gpr(r) => write!(f, "x{}", r),
            Operand::Predicate(p) => write!(f, "p{}", p),
            Operand::Immediate(i) => write!(f, "{}", i),
            Operand::PredicateVal(b) => write!(f, "{}", b),
            Operand::None => write!(f, ""),
        }?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Label {
    SrcAddrSpace(usize),
    DstAddrSpace(usize),
    None
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Label::DstAddrSpace(i) | Label::SrcAddrSpace(i) => write!(f, "{:x}", i),
            Label::None => write!(f, ""),
        }?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(into="String")]
pub struct Inst {
    pub opcode: Opcode,
    pub addr: usize,
    pub dest: Operand,
    pub src1: Option<u32>,  // can only be a register or nothing
    pub src2: Operand,
    pub label: Label,
    pub offset: Option<i64>
}

fn parse_i_r_b_format_inst(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
    let mut remaining = remaining_line.split(",");
    let operand_cnt = remaining.clone().count();

    if operand_cnt != 3 {
        return Err(format!("Incorrect number of operands, expected 3, got {}", operand_cnt));
    }

    let dest = Operand::from_str(remaining.next().unwrap())?;
    let Operand::Gpr(_) = dest else { return Err(format!("dest must be a register of the form xN."))};
    let src1 = Operand::from_str(remaining.next().unwrap())?;
    let Operand::Gpr(src1) = src1 else { return Err(format!("src1 must be a register of the form xN."))};
    let src1 = Some(src1);
    let src2 = Operand::from_str(remaining.next().unwrap())?;
    if let InstParseFormat::I = opcode.parse_format() {
        let Operand::Immediate(_) = src2 else { 
            return Err(String::from("src2 must be an immediate for I-format instruction."));
        };
        Ok(Inst {
            opcode,
            addr: 0,
            dest,
            src1,
            src2,
            label: Label::None,
            offset: None          
        })
    } else if let InstParseFormat::B = opcode.parse_format()  {
        if let Operand::Immediate(i) = src2 {
            Ok(Inst {
                opcode,
                addr: 0,
                dest: Operand::None,
                // TODO REMEMBER TO FLIP SRC1 AND SRC2 !!!!!!!!
                src1,
                src2: dest,
                label: Label::SrcAddrSpace(i as usize),
                offset: None          
            })
        } else { 
            Err(String::from("src2 must be an immediate for I-format instruction."))
        }
    } else {
        let Operand::Gpr(_) = src2 else {
            return Err(String::from("src2 must be a GPR for R-format instruction."));
        };
        Ok(Inst {
            opcode,
            addr: 0,
            dest,
            src1,
            src2,
            label: Label::None,
            offset: None          
        })
    }
}

fn parse_l_s_format_inst(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
    let mut remaining = remaining_line.split(",");
    let operand_cnt = remaining.clone().count();
    if operand_cnt != 2 {
        return Err(format!("Incorrect number of operands, expected 2, got {}", operand_cnt));
    }
    let reg = Operand::from_str(remaining.next().unwrap())?;
    let Operand::Gpr(reg) = reg else { return Err(format!("dest must be a register of the form xN."))};
    
    let mem_loc = String::from(remaining.next().unwrap().split_once(")").unwrap().0);
    let mut mem_loc = mem_loc.split("(");

    let ofs = Operand::from_str(mem_loc.next().unwrap())?;
    let Operand::Immediate(ofs) = ofs else { return Err(format!("Offset must be an immediate."))};

    let base = Operand::from_str(mem_loc.next().unwrap())?;
    let Operand::Gpr(base) = base else { return Err(format!("base must be a register of the form xN.."))};

    if opcode.parse_format() == InstParseFormat::S {
        Ok(Inst {
            opcode,
            addr: 0,
            dest: Operand::None,
            src1: Some(reg),
            src2: Operand::Gpr(base),
            label: Label::None,
            offset: Some(ofs)
        })
    } else {
        Ok(Inst {
            opcode,
            addr: 0,
            dest: Operand::Gpr(reg),
            src1: Some(base),
            src2: Operand::None,
            label: Label::None,
            offset: Some(ofs)
        })
    }
}

fn parse_j_format_inst(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
    let loop_label = Operand::from_str(&remaining_line)?;
    let Operand::Immediate(i) = loop_label else { return Err(format!("Loop label must be an immediate."))};
    Ok(Inst {
        opcode,
        addr: 0,
        // TODO handle a jump and link which is not using the link register
        dest: Operand::Gpr(if opcode == Opcode::JAL {1} else {0}),
        src1: None,
        src2: Operand::None,
        label: Label::SrcAddrSpace(i as usize),
        offset: None
    })
}

fn parse_mov_format_inst(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
    let mut remaining = remaining_line.split(",");
    let operand_cnt = remaining.clone().count();
    if operand_cnt != 2 {
        return Err(format!("Incorrect number of operands, expected 2, got {}", operand_cnt));
    }
    let dest = Operand::from_str(remaining.next().unwrap())?;
    let src = Operand::from_str(remaining.next().unwrap())?;
    match dest {
        Operand::Gpr(_) => {
            match src {
                Operand::Gpr(_) | Operand::Immediate(_) => {},
                _ => return Err(format!("mov src must be a register of the form xN or an immediate when dest is a GPR."))
            }
        },
        Operand::Predicate(_) => {
            let Operand::PredicateVal(_) = src else { return Err(format!("mov src must be a predicate value when dest is a predicate register."))};
        }
        Operand::Ec | Operand::Lc => {
            let Operand::Immediate(_) = src else { return Err(format!("mov src must be an immediate value when dest is LC/EC."))};
        }
        _ => return Err(format!("mov dest must be a register of the form xN, a predicate register, or LC/EC."))
    }
    Ok(Inst {
        opcode,
        addr: 0,
        dest,
        src1: Some(0),
        src2: src,
        label: Label::None,
        offset: None
    })
}

impl Inst {
    pub fn from_str(line: &str, addr: usize) -> Result<Self, String> {
        let mut line_split = line.split(" ");
        // TODO error message if no opcode
        let opcode: Opcode = Opcode::from_str(line_split.next().unwrap())?;
        let remaining = line_split.collect::<String>();
        let inst = match opcode.parse_format() {
            InstParseFormat::R | InstParseFormat::I | InstParseFormat::B => parse_i_r_b_format_inst(opcode, remaining),
            InstParseFormat::S | InstParseFormat::L => parse_l_s_format_inst(opcode, remaining),
            InstParseFormat::J => parse_j_format_inst(opcode, remaining),
            InstParseFormat::MOV => parse_mov_format_inst(opcode, remaining),
            InstParseFormat::NOP => {
                Ok(Inst {
                    opcode,
                    addr: 0,
                    dest: Operand::None,
                    src1: None,
                    src2: Operand::None,
                    label: Label::None,
                    offset: None
                })
            },
        };
        inst.and_then(|mut x| {x.addr = addr; Ok(x)}) 
    }

    pub fn print_fill(&self, output_str: &mut String, budget: usize) {
        let inst_str = format!("{}", self);
        let gap = std::cmp::max(budget-inst_str.len(), 0);
        output_str.push_str(inst_str.as_str());
        for _ in 0..gap {
            output_str.push(' ');
        }
        output_str.push_str(" | ");
    }

    pub fn nop() -> Self {
        Self {
            opcode: Opcode::NOP,
            addr: 0,
            dest: Operand::None,
            src1: None,
            src2: Operand::None,
            label: Label::None,
            offset: None
        }
    }

    /*pub fn gen_loop(pipelined: bool, addr: usize) -> Self {
        Self {
            opcode: if pipelined { Opcode::LOOP_PIP } else { Opcode::LOOP },
            addr: 0,
            dest: Operand::None,
            src1: None,
            src2: Operand::Immediate(addr as i64),
            offset: None
        }
    }*/

}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.to_str())?;
        match self.opcode.parse_format() {
            InstParseFormat::R | InstParseFormat::I => {
                write!(f, " {},", self.dest)?;
                write!(f, " x{},", self.src1.unwrap())?;
                write!(f, " {}", self.src2)?;
            },
            InstParseFormat::B => {
                write!(f, " {},", self.src2)?;
                write!(f, " x{},", self.src1.unwrap())?;
                write!(f, " {}", self.label)?;
            }
            InstParseFormat::L => {
                write!(f, " {},", self.dest)?;
                write!(f, " {}(x{})", self.offset.unwrap(), self.src1.unwrap())?;
            },
            InstParseFormat::S => {
                write!(f, " x{},", self.src1.unwrap())?;
                write!(f, " {}({})", self.offset.unwrap(), self.src2)?;
            },
            InstParseFormat::J => {
                write!(f, " {}", self.label)?;
            },
            InstParseFormat::NOP => {},
            InstParseFormat::MOV => {
                write!(f, " {},", self.dest)?;
                write!(f, " {}", self.src2)?;
            }
        }
        Ok(())
    }
}

impl Into<String> for Inst {
    fn into(self) -> String {
        format!("{}", self)
    }
}