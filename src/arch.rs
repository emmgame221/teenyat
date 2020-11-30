use std::fs::{self, File};
use std::io::{self, Read, Write};

const OP_CODE_MASK: u16 = !(!0u16 << 5) << 11;
const OP_CODE_SHIFT: u16 = 11;
const RA_MASK: u16 = !(!0u16 << 3) << 8;
const RA_SHIFT: u16 = 8;
const RB_MASK: u16 = !(!0u16 << 3) << 5;
const RB_SHIFT: u16 = 5;
const MEM_SIZE: u16 = 32768;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instruction {
    pub word_op_regs: u16,
    pub word_imm: u16,
}

impl Instruction {
    pub fn new(word_op_regs: u16, word_imm: u16) -> Self {
        Self {
            word_op_regs,
            word_imm,
        }
    }

    pub fn with_vals(op: OpCode, ra: RegMnem, rb: RegMnem, imm: u16) -> Self {
        let word1 =
            op.to_int() << OP_CODE_SHIFT | ra.to_int() << RA_SHIFT | rb.to_int() << RB_SHIFT;
        Self {
            word_op_regs: word1,
            word_imm: imm,
        }
    }

    pub fn get_op_code(&self) -> Result<OpCode, ArchError> {
        let code = (self.word_op_regs & OP_CODE_MASK) >> OP_CODE_SHIFT;
        OpCode::from_int(code)
    }

    pub fn get_ra(&self) -> Result<RegMnem, ArchError> {
        let code = (self.word_op_regs & RA_MASK) >> RA_SHIFT;
        RegMnem::from_int(code)
    }

    pub fn get_rb(&self) -> Result<RegMnem, ArchError> {
        let code = (self.word_op_regs & RB_MASK) >> RB_SHIFT;
        RegMnem::from_int(code)
    }
}

impl Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = self.get_op_code().unwrap();
        let regs = op.num_regs();
        let ra = if regs > 0 {
            format!("{:?}", self.get_ra().unwrap())
        } else {
            "".to_string()
        };
        let rb = if regs > 1 {
            format!("{:?}", self.get_rb().unwrap())
        } else {
            "".to_string()
        };
        let imm_type = match op {
            OpCode::Set | OpCode::Shl | OpCode::Shr => " imm",
            OpCode::Load
            | OpCode::Stor
            | OpCode::Call
            | OpCode::Jl
            | OpCode::Jle
            | OpCode::Je
            | OpCode::Jne
            | OpCode::Jge
            | OpCode::Jg => "addr",
            _ => "",
        };
        write!(f, "{:?} {} {} {} {}", op, ra, rb, imm_type, self.word_imm)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpCode {
    Set,
    Copy,
    Load,
    Stor,
    PLoad,
    PStor,
    Push,
    Pop,
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Neg,
    Inc,
    Dec,
    And,
    Or,
    Xor,
    Inv,
    Shl,
    Shr,
    Call,
    Jl,
    Jle,
    Je,
    Jne,
    Jge,
    Jg,
}

impl OpCode {
    pub fn to_int(&self) -> u16 {
        use OpCode::*;
        match self {
            Set => 0,
            Copy => 1,
            Load => 2,
            Stor => 3,
            PLoad => 4,
            PStor => 5,
            Push => 6,
            Pop => 7,
            Add => 8,
            Sub => 9,
            Mult => 10,
            Div => 11,
            Mod => 12,
            Neg => 13,
            Inc => 14,
            Dec => 15,
            And => 16,
            Or => 17,
            Xor => 18,
            Inv => 19,
            Shl => 20,
            Shr => 21,
            Call => 22,
            Jl => 23,
            Jle => 24,
            Je => 25,
            Jne => 26,
            Jge => 27,
            Jg => 28,
        }
    }

    pub fn from_int(code: u16) -> Result<OpCode, ArchError> {
        use OpCode::*;
        match code {
            0 => Ok(Set),
            1 => Ok(Copy),
            2 => Ok(Load),
            3 => Ok(Stor),
            4 => Ok(PLoad),
            5 => Ok(PStor),
            6 => Ok(Push),
            7 => Ok(Pop),
            8 => Ok(Add),
            9 => Ok(Sub),
            10 => Ok(Mult),
            11 => Ok(Div),
            12 => Ok(Mod),
            13 => Ok(Neg),
            14 => Ok(Inc),
            15 => Ok(Dec),
            16 => Ok(And),
            17 => Ok(Or),
            18 => Ok(Xor),
            19 => Ok(Inv),
            20 => Ok(Shl),
            21 => Ok(Shr),
            22 => Ok(Call),
            23 => Ok(Jl),
            24 => Ok(Jle),
            25 => Ok(Je),
            26 => Ok(Jne),
            27 => Ok(Jge),
            28 => Ok(Jg),
            _ => Err(ArchError::InvalidOpCode(code)),
        }
    }

    pub fn num_regs(&self) -> u16 {
        use OpCode::*;
        match self {
            Call => 0,
            Set | Load | Stor | Push | Pop | Neg | Inc | Dec | Inv | Shl | Shr => 1,
            _ => 2,
        }
    }

    pub fn from_str(op: &str) -> Result<OpCode, ArchError> {
        use OpCode::*;
        let op = op.to_ascii_lowercase();
        match op.as_str() {
            "set" => Ok(Set),
            "copy" => Ok(Copy),
            "load" => Ok(Load),
            "stor" => Ok(Stor),
            "pload" => Ok(PLoad),
            "pstor" => Ok(PStor),
            "push" => Ok(Push),
            "pop" => Ok(Pop),
            "add" => Ok(Add),
            "sub" => Ok(Sub),
            "mult" => Ok(Mult),
            "div" => Ok(Div),
            "mod" => Ok(Mod),
            "neg" => Ok(Neg),
            "inc" => Ok(Inc),
            "dec" => Ok(Dec),
            "and" => Ok(And),
            "or" => Ok(Or),
            "xor" => Ok(Xor),
            "inv" => Ok(Inv),
            "shl" => Ok(Shl),
            "shr" => Ok(Shr),
            "call" => Ok(Call),
            "jl" => Ok(Jl),
            "jle" => Ok(Jle),
            "je" => Ok(Je),
            "jne" => Ok(Jne),
            "jge" => Ok(Jge),
            "jg" => Ok(Jg),
            _ => Err(ArchError::InvalidOpMnem(op)),
        }
    }
}

impl Default for OpCode {
    fn default() -> Self {
        OpCode::Set
    }
}

#[derive(Clone, Debug)]
pub struct Register {
    pub mnem: RegMnem,
    pub val: u16,
}

impl Register {
    pub fn new(mnem: RegMnem) -> Self {
        Self { mnem, val: 0 }
    }
}

impl Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.mnem, self.val as i16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegMnem {
    Pc,
    _R0,
    R1,
    Ax,
    R2,
    Bx,
    R3,
    Cx,
    R4,
    Dx,
    R5,
    Ex,
    R6,
    Fx,
    Sp,
    R7,
}

impl RegMnem {
    pub fn to_int(&self) -> u16 {
        use RegMnem::*;
        match self {
            Pc => 0,
            _R0 => 0,
            R1 => 1,
            Ax => 1,
            R2 => 2,
            Bx => 2,
            R3 => 3,
            Cx => 3,
            R4 => 4,
            Dx => 4,
            R5 => 5,
            Ex => 5,
            R6 => 6,
            Fx => 6,
            Sp => 7,
            R7 => 7,
        }
    }

    pub fn from_int(code: u16) -> Result<RegMnem, ArchError> {
        use RegMnem::*;
        match code {
            0 => Ok(Pc),
            1 => Ok(R1),
            2 => Ok(R2),
            3 => Ok(R3),
            4 => Ok(R4),
            5 => Ok(R5),
            6 => Ok(R6),
            7 => Ok(Sp),
            _ => Err(ArchError::InvalidRegister(code)),
        }
    }

    pub fn from_str(mnem: &str) -> Result<RegMnem, ArchError> {
        use RegMnem::*;
        let mnem = mnem.to_ascii_lowercase();
        match mnem.as_str() {
            "pc" | "r0" => Ok(Pc),
            "ax" | "r1" => Ok(R1),
            "bx" | "r2" => Ok(R2),
            "cx" | "r3" => Ok(R3),
            "dx" | "r4" => Ok(R4),
            "ex" | "r5" => Ok(R5),
            "fx" | "r6" => Ok(R6),
            "sp" | "r7" => Ok(R7),
            _ => Err(ArchError::InvalidRegMnem(mnem)),
        }
    }
}

impl Default for RegMnem {
    fn default() -> Self {
        RegMnem::Pc
    }
}

#[derive(Debug)]
pub struct Memory {
    ram: Vec<u16>,
    next_ins: usize,
}

impl Memory {
    pub fn new() -> Self {
        let ram: Vec<u16> = vec![0; MEM_SIZE as usize];
        Self { ram, next_ins: 0 }
    }

    pub fn from_rom_file(path: &str) -> io::Result<Self> {
        //println!("Reading from rom file.");
        let mut mem = Self::new();
        let file = File::open(path)?;
        let mut bytes = file.bytes();
        while let Some(Ok(byte)) = bytes.next() {
            let upper = byte as u16;
            let lower = (bytes.next().unwrap().unwrap() as u16) << 8;
            let word = upper | lower;
            //print!("{:04x} ", word);
            mem.ram[mem.next_ins] = word;
            mem.next_ins += 1;
        }
        //println!("");
        Ok(mem)
    }

    pub fn read(&self, addr: u16) -> Result<u16, ArchError> {
        if addr >= MEM_SIZE {
            Err(ArchError::MemAddrOutOfRange(addr))
        } else {
            Ok(self.ram[addr as usize])
        }
    }

    pub fn write(&mut self, addr: u16, val: u16) -> Result<(), ArchError> {
        if addr >= MEM_SIZE {
            Err(ArchError::MemAddrOutOfRange(addr))
        } else {
            self.ram[addr as usize] = val;
            Ok(())
        }
    }

    pub fn add_ins(&mut self, ins: Instruction) {
        self.ram[self.next_ins] = ins.word_op_regs;
        self.ram[self.next_ins + 1] = ins.word_imm;
        self.next_ins += 2;
    }

    pub fn print_program(&self) {
        let mut i = 0usize;
        while i < self.next_ins {
            //let ins = Instruction::new(self.ram[i], self.ram[i + 1]);
            //println!("{}", ins);
            println!("0x{:04x} 0x{:04x}", self.ram[i], self.ram[i + 1]);
            i += 2;
        }
    }

    pub fn save_program(&self, path: &str) -> io::Result<()> {
        let mut out_file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?;
        out_file.write_all(&self.bytes())?;
        Ok(())
    }

    fn bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        for (i, word) in self.ram.iter().enumerate() {
            let byte1 = ((*word & 0xFF00) >> 8) as u8;
            let byte2 = (*word & 0x00FF) as u8;
            bytes.push(byte2);
            bytes.push(byte1);
            if i > self.next_ins {
                break;
            }
        }
        bytes
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArchError {
    InvalidOpCode(u16),
    InvalidOpMnem(String),
    InvalidRegister(u16),
    InvalidRegMnem(String),
    MemAddrOutOfRange(u16),
    InvalidInstruction,
    UnresolvableLabel(&'static str),
    InvalidOperand(&'static str),
    RepeatedLabel(String, u16, u16),
}

use std::fmt::{self, Display};

impl Display for ArchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ArchError::*;
        match self {
            InvalidOpCode(code) => {
                writeln!(f, "Invalid OpCode: {}", code)?;
            }
            InvalidOpMnem(mnem) => {
                writeln!(f, "Invalid OpCode Mnemonic: {}", mnem)?;
            }
            InvalidRegister(code) => {
                writeln!(f, "Invalid Register Code: {}", code)?;
            }
            InvalidRegMnem(mnem) => {
                writeln!(f, "Invalid Register Mnemonic: {}", mnem)?;
            }
            MemAddrOutOfRange(addr) => {
                writeln!(
                    f,
                    "Memory address out of range: {}. Last memory address is at: {}",
                    addr,
                    MEM_SIZE - 1
                )?;
            }
            InvalidInstruction => {
                writeln!(f, "Invalid Instruction")?;
            }
            UnresolvableLabel(msg) => {
                writeln!(f, "{}", msg)?;
            }
            InvalidOperand(msg) => {
                writeln!(f, "{}", msg)?;
            }
            RepeatedLabel(lbl, prev, cur) => {
                writeln!(
                    f,
                    "Ambiguous label: {}. First appearance: {}, Second appearance: {}",
                    lbl, prev, cur
                )?;
            }
        }
        Ok(())
    }
}

impl From<ArchError> for std::io::Error {
    fn from(err: ArchError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, format!("{}", err))
    }
}
