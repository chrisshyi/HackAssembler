extern crate hack_assembler;
use hack_assembler::*;
use std::fs::File;
use std::io::{Seek, SeekFrom, BufRead, BufReader, Write, BufWriter};


fn main() {
    // initialize objects
    let dest_file = File::open("dest_file.txt").unwrap();
    let comp_file = File::open("comp_file.txt").unwrap();
    let jump_file = File::open("jump_file.txt").unwrap();
    let mut predef_file = File::open("predefined_symbols.txt").unwrap();

    let a_decoder = ADecoder::new();
    let c_decoder = CDecoder::new(dest_file, comp_file, jump_file);

    let asm_file_root = "/home/chris/Dropbox/nand2tetris/nand2tetris/projects/06";
    let file_paths = vec!["add/Add.asm", "max/Max.asm", "pong/Pong.asm", "rect/Rect.asm"];
    for file_path in file_paths.iter() {
        predef_file.seek(SeekFrom::Start(0)); // rewind the file
        let mut symbol_table = SymbolTable::new(predef_file.try_clone().unwrap());
        if !symbol_table.symbol_map.contains_key(&"SCREEN".to_string()) {
            println!("Screen key doesn't exist.");
        }
        if !symbol_table.symbol_map.contains_key(&"KBD".to_string()) {
            println!("keyboard key doesn't exist");
        }
        let asm_file = File::open(format!("{}/{}", asm_file_root, file_path)).unwrap();
        let file_name = file_path.split(|c| c == '/' || c == '.').collect::<Vec<&str>>()[1];
        {
            let mut intm_file = File::create(format!("{}/{}.{}", asm_file_root, file_name, "intm")).unwrap();
            symbol_table.parse_file(asm_file, intm_file.try_clone().unwrap());
        }
        let mut intm_file = File::open(format!("{}/{}.intm", asm_file_root, file_name)).unwrap();
        let bin_file = File::create(format!("{}/{}.hack", asm_file_root, file_name)).unwrap();
        let reader = BufReader::new(intm_file.try_clone().unwrap());
        let mut writer = BufWriter::new(bin_file);
        for line in reader.lines() {
            let unwrapped_line = line.unwrap();
            let (parsed_line, info_map) = parse_line(unwrapped_line.as_str());
            let mut bin_line = String::new(); // the binary translation of the instruction line
            if *info_map.get("a_instruction").unwrap() {
                bin_line.push_str(a_decoder.decode(parsed_line, &info_map).as_str());
            } else {
                bin_line.push_str(c_decoder.decode(parsed_line, &info_map).as_str());
            }
            bin_line.push('\n');
            writer.write(bin_line.as_bytes());
        }

    }
    
}
