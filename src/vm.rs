use crate::arch::OpCode::*;
use crate::arch::RegMnem::*;
use crate::arch::*;
use std::io::{self, Write};
use std::process;

const SP_INIT: u16 = 0x8000;
const CHAR_OUT_ADDR: u16 = 0x8000;
const CHAR_IN_ADDR: u16 = 0x8001;
const END_PROG_ADDR: u16 = 0xFFFF;

#[derive(Debug)]
pub struct TeenyAT {
    mem: Memory,
    ins: Instruction,
    pc: Register,
    r1: Register,
    r2: Register,
    r3: Register,
    r4: Register,
    r5: Register,
    r6: Register,
    sp: Register,
    op_code: OpCode,
    ra: RegMnem,
    rb: RegMnem,
    imm: u16,
    addr: u16,
    pub debug_mode: bool,
}

impl TeenyAT {
    pub fn new(program: Memory) -> Self {
        let ins = Instruction::new(0, 0);
        let pc = Register::new(RegMnem::Pc);
        let r1 = Register::new(RegMnem::R1);
        let r2 = Register::new(RegMnem::R2);
        let r3 = Register::new(RegMnem::R3);
        let r4 = Register::new(RegMnem::R4);
        let r5 = Register::new(RegMnem::R5);
        let r6 = Register::new(RegMnem::R6);
        let mut sp = Register::new(RegMnem::Sp);
        sp.val = SP_INIT;
        Self {
            mem: program,
            ins,
            pc,
            r1,
            r2,
            r3,
            r4,
            r5,
            r6,
            sp,
            op_code: OpCode::Set,
            ra: RegMnem::default(),
            rb: RegMnem::default(),
            imm: 0,
            addr: 0,
            debug_mode: false,
        }
    }

    pub fn run(&mut self) -> Result<(), ArchError> {
        if self.debug_mode {
            self.mem.print_program();
        }
        loop {
            self.fetch()?;
            self.decode()?;
            self.execute()?;
        }
    }

    fn fetch(&mut self) -> Result<(), ArchError> {
        let word1 = self.mem.read(self.pc.val)?;
        let word2 = self.mem.read(self.pc.val + 1)?;
        self.ins = Instruction::new(word1, word2);
        self.pc.val += 2;
        Ok(())
    }

    fn decode(&mut self) -> Result<(), ArchError> {
        self.op_code = self.ins.get_op_code()?;
        let num_regs = self.op_code.num_regs();
        if num_regs >= 1 {
            self.ra = self.ins.get_ra()?;
        }
        if num_regs == 2 {
            self.rb = self.ins.get_rb()?;
        }
        self.imm = self.ins.word_imm;
        self.addr = self.imm;
        Ok(())
    }

    fn execute(&mut self) -> Result<(), ArchError> {
        match self.op_code {
            Set => self.set(),
            Copy => self.copy(),
            Load => self.load()?,
            Stor => self.stor()?,
            PLoad => self.pload()?,
            PStor => self.pstor()?,
            Push => self.push()?,
            Pop => self.pop()?,
            Add => self.add(),
            Sub => self.sub(),
            Mult => self.mult(),
            Div => self.div(),
            Mod => self.divmod(),
            Neg => self.neg(),
            Inc => self.inc(),
            Dec => self.dec(),
            And => self.and(),
            Or => self.or(),
            Xor => self.xor(),
            Inv => self.inv(),
            Shl => self.shl(),
            Shr => self.shr(),
            Call => self.call()?,
            Jl => self.jl(),
            Jle => self.jle(),
            Je => self.je(),
            Jne => self.jne(),
            Jge => self.jge(),
            Jg => self.jg(),
        }
        Ok(())
    }

    fn get_ra(&mut self) -> &mut Register {
        match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        }
    }

    fn rb_val(&self) -> u16 {
        match self.rb {
            _R0 | Pc => self.pc.val,
            R1 | Ax => self.r1.val,
            R2 | Bx => self.r2.val,
            R3 | Cx => self.r3.val,
            R4 | Dx => self.r4.val,
            R5 | Ex => self.r5.val,
            R6 | Fx => self.r6.val,
            R7 | Sp => self.sp.val,
        }
    }

    fn set(&mut self) {
        let imm = self.imm;
        let ra = self.get_ra();
        ra.val = imm;
    }

    fn copy(&mut self) {
        let rb_val = match self.rb {
            _R0 | Pc => self.pc.val,
            R1 | Ax => self.r1.val,
            R2 | Bx => self.r2.val,
            R3 | Cx => self.r3.val,
            R4 | Dx => self.r4.val,
            R5 | Ex => self.r5.val,
            R6 | Fx => self.r6.val,
            R7 | Sp => self.sp.val,
        };
        let ra = self.get_ra();
        ra.val = rb_val;
    }

    fn load(&mut self) -> Result<(), ArchError> {
        let addr = self.addr;
        let ra = match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        };
        if addr == CHAR_IN_ADDR {
            ra.val = input_char();
        } else if addr == END_PROG_ADDR {
            process::exit(ra.val as i32);
        } else {
            ra.val = self.mem.read(addr)?;
        }
        Ok(())
    }

    fn stor(&mut self) -> Result<(), ArchError> {
        let ra = match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        };
        if self.addr == CHAR_OUT_ADDR {
            output_char(ra.val);
        } else if self.addr == END_PROG_ADDR {
            process::exit(ra.val as i32);
        } else {
            self.mem.write(self.addr, ra.val)?;
        }
        Ok(())
    }

    fn pload(&mut self) -> Result<(), ArchError> {
        let rb = self.rb_val();
        let ra = match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        };
        if rb == CHAR_IN_ADDR {
            ra.val = input_char();
        } else if rb == END_PROG_ADDR {
            process::exit(ra.val as i32);
        } else {
            ra.val = self.mem.read(rb)?;
        }
        Ok(())
    }

    fn pstor(&mut self) -> Result<(), ArchError> {
        let rb = self.rb_val();
        let ra = match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        };
        if ra.val == CHAR_OUT_ADDR {
            output_char(rb);
        } else if ra.val == END_PROG_ADDR {
            process::exit(ra.val as i32);
        } else {
            self.mem.write(ra.val, rb)?;
        }
        Ok(())
    }

    fn push(&mut self) -> Result<(), ArchError> {
        let ra = self.get_ra();
        let temp = ra.val;
        self.sp.val -= 1;
        self.mem.write(self.sp.val, temp)?;
        Ok(())
    }

    fn pop(&mut self) -> Result<(), ArchError> {
        let addr = self.sp.val;
        let ra = match self.ra {
            _R0 | Pc => &mut self.pc,
            R1 | Ax => &mut self.r1,
            R2 | Bx => &mut self.r2,
            R3 | Cx => &mut self.r3,
            R4 | Dx => &mut self.r4,
            R5 | Ex => &mut self.r5,
            R6 | Fx => &mut self.r6,
            R7 | Sp => &mut self.sp,
        };
        ra.val = self.mem.read(addr)?;
        self.sp.val += 1;
        Ok(())
    }

    fn add(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val += rb;
    }

    fn sub(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val -= rb;
    }

    fn mult(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val *= rb;
    }

    fn div(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val /= rb;
    }

    fn divmod(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val %= rb;
    }

    fn neg(&mut self) {
        let ra = self.get_ra();
        ra.val = -(ra.val as i16) as u16;
    }

    fn inc(&mut self) {
        let ra = self.get_ra();
        ra.val += 1;
    }

    fn dec(&mut self) {
        let ra = self.get_ra();
        ra.val -= 1;
    }

    fn and(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val &= rb;
    }

    fn or(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val |= rb;
    }

    fn xor(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        ra.val ^= rb;
    }

    fn inv(&mut self) {
        let ra = self.get_ra();
        ra.val = !ra.val;
    }

    fn shl(&mut self) {
        let imm = self.imm;
        let ra = self.get_ra();
        ra.val <<= imm;
    }

    fn shr(&mut self) {
        let imm = self.imm;
        let ra = self.get_ra();
        ra.val >>= imm;
    }

    fn call(&mut self) -> Result<(), ArchError> {
        self.push()?;
        self.pc.val = self.addr;
        Ok(())
    }

    fn jl(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) < (rb as i16) {
            self.pc.val = self.addr;
        }
    }

    fn jle(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) <= (rb as i16) {
            self.pc.val = self.addr;
        }
    }

    fn je(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) == (rb as i16) {
            self.pc.val = self.addr;
        }
    }

    fn jne(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) != (rb as i16) {
            self.pc.val = self.addr;
        }
    }

    fn jge(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) >= (rb as i16) {
            self.pc.val = self.addr;
        }
    }

    fn jg(&mut self) {
        let rb = self.rb_val();
        let ra = self.get_ra();
        if (ra.val as i16) > (rb as i16) {
            self.pc.val = self.addr;
        }
    }
}

fn input_char() -> u16 {
    let input = io::stdin();
    let mut buf: String = String::new();
    let _line = input.read_line(&mut buf);
    buf.bytes().next().unwrap() as u16
}

fn output_char(chr: u16) {
    let mut output = io::stdout();
    let buf: Vec<u8> = vec![chr as u8];
    output.write(&buf).unwrap();
    output.flush().unwrap();
}
