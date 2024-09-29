use std::{collections::HashMap, fmt};

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Opcode {
    ADD,
    ADDI,
    SUB,
    MULU,
    LD,
    ST,
    LOOP,
    #[serde(rename = "loop.pip")]
    LOOP_PIP,
    NOP,
    MOV,
}

impl Opcode {
    fn from_str(op: &str) -> Result<Self, String> {
        match op {
            "add" => Ok(Self::ADD),
            "addi" => Ok(Self::ADDI),
            "sub" => Ok(Self::SUB),
            "mulu" => Ok(Self::MULU),
            "ld" => Ok(Self::LD),
            "st" => Ok(Self::ST),
            "loop" => Ok(Self::LOOP),
            "loop.pip" => Ok(Self::LOOP_PIP),
            "nop" => Ok(Self::NOP),
            "mov" => Ok(Self::MOV),
            _ => Err(format!("Unrecognized opcode: {}", op))
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            Self::ADD => "add",
            Self::ADDI => "addi",
            Self::SUB => "sub",
            Self::MULU => "mulu",
            Self::LD => "ld",
            Self::ST => "st",
            Self::LOOP => "loop",
            Self::LOOP_PIP => "loop.pip",
            Self::NOP => "nop",
            Self::MOV => "mov",
        }
    }

    pub fn eu_type(&self) -> ExecutionUnit {
        match self {
            Self::ADD | Self::ADDI | Self::SUB | Self::MOV => ExecutionUnit::ALU,
            Self::MULU => ExecutionUnit::Mult,
            Self::LD | Self::ST => ExecutionUnit::Mem,
            Self::LOOP | Self::LOOP_PIP => ExecutionUnit::Branch,
            Self::NOP => ExecutionUnit::ALU, // but why? 
        }
    }
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
#[serde(into="String")]
pub struct Inst {
    pub opcode: Opcode,
    pub addr: usize,
    pub dest: Operand,
    pub src1: Option<u32>,  // can only be a register or nothing
    pub src2: Operand,
    pub offset: Option<i64>
}

fn parse_aluop_insn(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
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
    if let Opcode::ADDI = opcode {
        let Operand::Immediate(_) = src2 else { 
            return Err(String::from("src2 must be an immediate for ADDI opcode."));
        };
    } else {
        let Operand::Gpr(_) = src2 else {
            return Err(String::from("src2 must be a GPR for non-ADDI ALU opcode."));
        };
    }
    Ok(Inst {
        opcode,
        addr: 0,
        dest,
        src1,
        src2,
        offset: None          
    })
}

fn parse_ldst_insn(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
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

    if opcode == Opcode::ST {
        Ok(Inst {
            opcode,
            addr: 0,
            dest: Operand::None,
            src1: Some(reg),
            src2: Operand::Gpr(base),
            offset: Some(ofs)
        })
    } else {
        Ok(Inst {
            opcode,
            addr: 0,
            dest: Operand::Gpr(reg),
            src1: Some(base),
            src2: Operand::None,
            offset: Some(ofs)
        })
    }
}

fn parse_loop_insn(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
    let loop_label = Operand::from_str(&remaining_line)?;
    let Operand::Immediate(_) = loop_label else { return Err(format!("Loop label must be an immediate."))};
    Ok(Inst {
        opcode,
        addr: 0,
        dest: Operand::None,
        src1: None,
        src2: loop_label,
        offset: None
    })
}

fn parse_mov_insn(opcode: Opcode, remaining_line: String) -> Result<Inst, String> {
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
        src1: None,
        src2: src,
        offset: None
    })
}

impl Inst {
    pub fn from_str(line: &str, addr: usize) -> Result<Self, String> {
        let mut line_split = line.split(" ");
        // TODO error message if no opcode
        let opcode: Opcode = Opcode::from_str(line_split.next().unwrap())?;
        let remaining = line_split.collect::<String>();
        let inst = match opcode {
            Opcode::ADD | Opcode::ADDI | Opcode::SUB | Opcode::MULU => parse_aluop_insn(opcode, remaining),
            Opcode::LD | Opcode::ST => parse_ldst_insn(opcode, remaining),
            Opcode::NOP => {
                Ok(Inst {
                    opcode,
                    addr: 0,
                    dest: Operand::None,
                    src1: None,
                    src2: Operand::None,
                    offset: None
                })
            },
            Opcode::LOOP | Opcode::LOOP_PIP => parse_loop_insn(opcode, remaining),
            Opcode::MOV => parse_mov_insn(opcode, remaining)
        };
        inst.and_then(|mut x| {x.addr = addr; Ok(x)}) 
    }

    pub fn nop() -> Self {
        Self {
            opcode: Opcode::NOP,
            addr: 0,
            dest: Operand::None,
            src1: None,
            src2: Operand::None,
            offset: None
        }
    }

    pub fn gen_loop(pipelined: bool, addr: usize) -> Self {
        Self {
            opcode: if pipelined { Opcode::LOOP_PIP } else { Opcode::LOOP },
            addr: 0,
            dest: Operand::None,
            src1: None,
            src2: Operand::Immediate(addr as i64),
            offset: None
        }
    }

}

impl fmt::Display for Inst {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.opcode.to_str())?;
        match self.opcode {
            Opcode::ADD | Opcode::SUB | Opcode::MULU | Opcode::ADDI => {
                write!(f, " {},", self.dest)?;
                write!(f, " x{},", self.src1.unwrap())?;
                write!(f, " {}", self.src2)?;
            },
            Opcode::LD => {
                write!(f, " {},", self.dest)?;
                write!(f, " {}(x{})", self.offset.unwrap(), self.src1.unwrap())?;
            },
            Opcode::ST => {
                write!(f, " x{},", self.src1.unwrap())?;
                write!(f, " {}({})", self.offset.unwrap(), self.src2)?;
            },
            Opcode::LOOP | Opcode::LOOP_PIP => {
                write!(f, " {}", self.src2)?;
            },
            Opcode::NOP => {},
            Opcode::MOV => {
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