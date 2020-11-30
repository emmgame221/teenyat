use crate::arch::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

const OUT: &'static str = "OUT";
const OUT_ADDR: &'static str = "0x8000";
const IN: &'static str = "IN";
const IN_ADDR: &'static str = "0x8001";
const END: &'static str = "END";
const END_ADDR: &'static str = "0xffff";

#[derive(Debug)]
struct UnresolvedIns {
    op: OpCode,
    ra: RegMnem,
    rb: RegMnem,
    imm: Token,
}

impl UnresolvedIns {
    fn new(op: OpCode, ra: RegMnem, rb: RegMnem, imm: Token) -> Self {
        Self { op, ra, rb, imm }
    }

    fn resolve(&self, labels: &HashMap<String, u16>) -> Result<Instruction, ArchError> {
        let imm: u16 = match &self.imm {
            Token::Imm(imm) => *imm,
            Token::Label(lbl, _) => labels[lbl],
            _ => {
                return Err(ArchError::InvalidOperand(
                    "Parse Error: operand in immediate/address position not immediate or label",
                ))
            }
        };
        let ins = Instruction::with_vals(self.op, self.ra, self.rb, imm);
        Ok(ins)
    }
}

pub fn parse_file(path: &str) -> io::Result<Memory> {
    let path = Path::new(path);
    let infile = File::open(path)?;
    let mut lines = read_file(infile)?;
    preprocess(&mut lines);
    let mut instructions: Vec<UnresolvedIns> = Vec::new();
    let mut labels: HashMap<String, u16> = HashMap::new();
    let mut next_ins_addr: u16 = 0;
    for (linenum, line) in lines.iter().enumerate() {
        let tokens = tokenize(&line, linenum as u16);
        let mut i = 0;
        while i < tokens.len() {
            let tok = tokens[i].clone();
            match tok {
                Token::Op(op) => {
                    handle_op(op, &tokens, &mut instructions, &mut i)?;
                    next_ins_addr += 2;
                }
                Token::Label(_, _) => {
                    handle_label(&tok, &mut labels, false, next_ins_addr, linenum)?;
                }
                _ => {
                    /*return Err(io::Error::new(
                        io::ErrorKind::Other,
                        format!("Parse error: Immediate or register mnemonic out of position at line {}: {:?}", linenum, tok),
                    ))*/
                }
            }
            i += 1;
        }
    }
    let mut mem = Memory::new();
    for ins in instructions.iter() {
        mem.add_ins(ins.resolve(&labels)?);
    }
    Ok(mem)
}

fn preprocess(lines: &mut Vec<String>) {
    for line in lines.iter_mut() {
        *line = line.trim().to_string();
        if let Some(i) = line.find(';') {
            line.truncate(i);
        }
        *line = line.replace(',', "");
        *line = line.replace("jmp", "set pc ");
        *line = line.replace("JMP", "set pc ");
        *line = line.replace("ret", "pop pc ");
        *line = line.replace("RET", "pop pc ");
        *line = line.replace(OUT, OUT_ADDR);
        *line = line.replace(IN, IN_ADDR);
        *line = line.replace(END, END_ADDR);
    }
}

fn handle_op(
    op: OpCode,
    tokens: &Vec<Token>,
    instructions: &mut Vec<UnresolvedIns>,
    i: &mut usize,
) -> io::Result<()> {
    let num_regs = op.num_regs();
    if num_regs == 0 {
        handle_op_0reg(op, instructions);
    } else if num_regs == 1 {
        handle_op_1reg(op, tokens, instructions, i);
    } else if num_regs == 2 {
        handle_op_2reg(op, tokens, instructions, i);
    }
    Ok(())
}

fn handle_op_0reg(op: OpCode, instructions: &mut Vec<UnresolvedIns>) {
    instructions.push(UnresolvedIns::new(
        op,
        RegMnem::default(),
        RegMnem::default(),
        Token::Imm(0),
    ));
}

fn handle_op_1reg(
    op: OpCode,
    tokens: &Vec<Token>,
    instructions: &mut Vec<UnresolvedIns>,
    i: &mut usize,
) {
    let mut i_ofs = 0usize;
    let ra = match op {
        OpCode::Stor => {
            if *i + 2 < tokens.len() {
                if let Token::Reg(reg) = tokens[*i + 2] {
                    i_ofs += 1;
                    reg
                } else {
                    RegMnem::default()
                }
            } else {
                RegMnem::default()
            }
        }
        _ => {
            if *i + 1 < tokens.len() {
                if let Token::Reg(reg) = tokens[*i + 1] {
                    i_ofs += 1;
                    reg
                } else {
                    RegMnem::default()
                }
            } else {
                RegMnem::default()
            }
        }
    };
    let rb = RegMnem::default();
    let mut imm = Token::Imm(0);
    match op {
        OpCode::Set | OpCode::Load | OpCode::Shl | OpCode::Shr => {
            imm = if *i + 2 < tokens.len() && tokens[*i + 2].is_imm() {
                i_ofs += 1;
                tokens[*i + 2].clone()
            } else {
                Token::Imm(0)
            };
        }
        OpCode::Stor => {
            imm = if *i + 1 < tokens.len() {
                i_ofs += 1;
                tokens[*i + 1].clone()
            } else {
                Token::Imm(0)
            }
        }
        _ => {}
    }
    *i += i_ofs;
    instructions.push(UnresolvedIns::new(op, ra, rb, imm));
}

fn handle_op_2reg(
    op: OpCode,
    tokens: &Vec<Token>,
    instructions: &mut Vec<UnresolvedIns>,
    i: &mut usize,
) {
    let mut i_ofs = 0usize;
    let ra = if *i + 1 < tokens.len() {
        if let Token::Reg(reg) = tokens[*i + 1] {
            i_ofs += 1;
            reg
        } else {
            RegMnem::default()
        }
    } else {
        RegMnem::default()
    };
    let rb = if *i + 2 < tokens.len() {
        if let Token::Reg(reg) = tokens[*i + 2] {
            i_ofs += 1;
            reg
        } else {
            RegMnem::default()
        }
    } else {
        RegMnem::default()
    };
    let imm = match op {
        OpCode::Jl | OpCode::Jle | OpCode::Je | OpCode::Jne | OpCode::Jge | OpCode::Jg => {
            i_ofs += 1;
            if *i + 3 < tokens.len() {
                tokens[*i + 3].clone()
            } else {
                Token::Imm(0)
            }
        }
        _ => Token::Imm(0),
    };
    *i += i_ofs;
    instructions.push(UnresolvedIns::new(op, ra, rb, imm));
}

fn handle_label(
    tok: &Token,
    labels: &mut HashMap<String, u16>,
    do_eval: bool,
    addr: u16,
    line_num: usize,
) -> io::Result<Option<u16>> {
    let (lbl, line) = match tok {
        Token::Label(lbl, line) => (lbl, *line),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Parse Error: attempted to treat non-label token as label",
            ))
        }
    };
    if do_eval {
        if let Some(addr) = labels.get(lbl) {
            Ok(Some(*addr))
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Parse Error: unresolved label",
            ))
        }
    } else {
        if let Some(_) = labels.insert(lbl.to_string(), addr) {
            Err(ArchError::RepeatedLabel(lbl.to_string(), line, line_num as u16).into())
        } else {
            Ok(None)
        }
    }
}

fn read_file(file: File) -> io::Result<Vec<String>> {
    let infile = io::BufReader::new(file);
    let lines = infile.lines();
    let mut lines_vec: Vec<String> = Vec::new();
    for line in lines {
        match line {
            Ok(line) => {
                lines_vec.push(line);
            }
            Err(err) => return Err(err),
        }
    }
    Ok(lines_vec)
}

fn tokenize(line: &str, linenum: u16) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    for token in line.split_whitespace() {
        tokens.push(Token::parse_str(token, linenum));
    }
    tokens
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Op(OpCode),
    Reg(RegMnem),
    Label(String, u16),
    Imm(u16),
}

impl Token {
    fn parse_str(tok: &str, linenum: u16) -> Token {
        use Token::*;
        if tok.starts_with('!') {
            return Label(tok.to_string(), linenum);
        }
        if tok.starts_with(':') {
            return Label(tok.to_string(), linenum);
        }
        if tok.starts_with('\'') {
            if tok.len() == 3 {
                if let Some(chr) = tok.chars().nth(1) {
                    return Imm(chr as u8 as u16);
                }
            } else if tok.len() == 4 {
                let mut chars = tok.chars();
                if let Some(esc) = chars.nth(1) {
                    if esc == '\\' {
                        let chr = chars.next().unwrap();
                        return Imm(escape_char(chr));
                    }
                }
            }
        }
        if let Ok(op) = OpCode::from_str(tok) {
            return Op(op);
        }
        if let Ok(reg) = RegMnem::from_str(tok) {
            return Reg(reg);
        }
        if let Ok(imm) = tok.parse::<u16>() {
            return Imm(imm);
        }
        if let Ok(imm) = tok.parse::<i16>() {
            return Imm(imm as u16);
        }
        if let Ok(imm) = u16::from_str_radix(tok.trim_start_matches("0x"), 16) {
            return Imm(imm);
        }
        Token::Imm(0)
    }

    fn is_imm(&self) -> bool {
        match self {
            Token::Imm(_) => true,
            _ => false,
        }
    }
}

fn escape_char(chr: char) -> u16 {
    match chr {
        'a' => 0x07,
        'b' => 0x08,
        'n' => 0x0A,
        'r' => 0x0D,
        't' => 0x09,
        '\\' => 0x5C,
        '\'' => 0x27,
        '\"' => 0x22,
        '?' => 0x3F,
        _ => 0,
    }
}
