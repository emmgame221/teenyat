mod arch;
mod assembler;
mod vm;

use std::env;

fn main() {
    let mut args = env::args();
    if args.len() >= 2 {
        args.next();
        let path = args.next().unwrap();
        let mut debug_mode = false;
        if let Some(s) = args.next() {
            debug_mode = s == "-d";
            if s == "-a" {
                assemble(path).unwrap();
                return;
            }
        }
        run(path, debug_mode).unwrap();
    } else {
        let path = console_input();
        run(path, false).unwrap();
    }
}

fn assemble(path: String) -> std::io::Result<()> {
    let mem = assembler::parse_file(&path)?;
    let out_path = path.replace(".tat", ".rom");
    mem.save_program(&out_path)?;
    mem.print_program();
    Ok(())
}

fn console_input() -> String {
    println!("Enter the name of the file to run: ");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).unwrap();
    buf
}

fn run(path: String, debug_mode: bool) -> std::io::Result<()> {
    let program = if path.ends_with(".tat") {
        assembler::parse_file(&path)?
    } else if path.ends_with(".rom") {
        arch::Memory::from_rom_file(&path)?
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input file must be either an assembly file (.tat) or a rom file (.rom)",
        ));
    };
    let mut vm = vm::TeenyAT::new(program);
    vm.debug_mode = debug_mode;
    vm.run()?;
    Ok(())
}
