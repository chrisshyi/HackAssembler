use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write, BufRead, Seek, SeekFrom};

pub trait Decode {
    /// Generates the binary representation of an instruction using its fields
    /// 
    /// Arguments:
    /// 
    /// * instruct_fields - fields of the instruction
    /// * info_map - additional information for decoding, such as whether dest and jump were set
    fn decode(&self, instruct_fields: Vec<&str>, info_map: &HashMap<&str, bool>) -> String; 
} 

pub struct ADecoder {}

impl ADecoder {
    pub fn new() -> ADecoder {
        ADecoder{}
    }
}

impl Decode for ADecoder {
    fn decode(&self, instruct_fields: Vec<&str>, info_map: &HashMap<&str, bool>) -> String {
        let mut instruct_str = String::new();
        instruct_str.push('0'); // push the op code
        let address: i32 = (*(instruct_fields.get(0).unwrap())).parse::<i32>().unwrap();
        instruct_str.push_str(format!("{:015b}", address).as_str()); // pad with zeros to make a width of 15 bits
        instruct_str
    }
}

pub struct CDecoder {
    dest_map: HashMap<String, String>,
    comp_map: HashMap<String, String>,
    jump_map: HashMap<String, String>,
}

impl CDecoder {
    pub fn new(dest_file: File, comp_file: File, jump_file: File) -> CDecoder {
        let mut buf_reader = BufReader::new(dest_file);
        let mut dest_map: HashMap<String, String> = HashMap::new();
        let mut comp_map: HashMap<String, String> = HashMap::new();
        let mut jump_map: HashMap<String, String> = HashMap::new();

        for line in buf_reader.lines() {
            let unwrapped_line = line.unwrap(); // unwrapped_line is a String
            let split_line: Vec<String> = unwrapped_line.split(" ").map(|s| s.to_string()).collect();
            dest_map.insert((*split_line.get(0).unwrap()).clone(), (*split_line.get(1).unwrap()).clone());
        }

        buf_reader = BufReader::new(comp_file);
        for line in buf_reader.lines() {
            let unwrapped_line = line.unwrap(); // unwrapped_line is a String
            let split_line: Vec<String> = unwrapped_line.split(" ").map(|s| s.to_string()).collect();
            comp_map.insert((*split_line.get(0).unwrap()).clone(), (*split_line.get(1).unwrap()).clone());
        }

        buf_reader = BufReader::new(jump_file);
        for line in buf_reader.lines() {
            let unwrapped_line = line.unwrap(); // unwrapped_line is a String
            let split_line: Vec<String> = unwrapped_line.split(" ").map(|s| s.to_string()).collect();
            jump_map.insert((*split_line.get(0).unwrap()).clone(), (*split_line.get(1).unwrap()).clone());
        }
        CDecoder {
            dest_map: dest_map,
            comp_map: comp_map,
            jump_map: jump_map
        }
    }
}

impl Decode for CDecoder {
    fn decode(&self, instruct_fields: Vec<&str>, info_map: &HashMap<&str, bool>) -> String {
        let mut instruct_str = String::new();
        let mut comp_index = 0; // the index of the comp instruction in the vector
        // binary forms of the 3 fields
        let dest_bin: String;
        let comp_bin: String;
        let jump_bin: String;
        // if dest is specified, it would be the first field
        if *info_map.get("dest").unwrap() {
            let dest = instruct_fields.get(0).unwrap().to_string();
            dest_bin = self.dest_map.get(&dest).unwrap().to_string();
            comp_index = 1;
        } else {
            dest_bin = "000".to_string();
        }
        let comp = instruct_fields.get(comp_index).unwrap().to_string();
        comp_bin = self.comp_map.get(&comp).unwrap().clone().to_string();
        // if jump is specified, it would be the last field
        if *info_map.get("jump").unwrap() {
            let jump = instruct_fields.get(instruct_fields.len() - 1).unwrap().to_string();
            jump_bin = self.jump_map.get(&jump).unwrap().clone().to_string();
        } else {
            jump_bin = "000".to_string();
        }

        instruct_str.push_str("111"); // add the op code
        instruct_str.push_str(comp_bin.as_str());
        instruct_str.push_str(dest_bin.as_str());
        instruct_str.push_str(jump_bin.as_str());

        instruct_str
    }
}
/// Splits an instruction line into its fields
/// # Arguments
/// 
/// * `line` - The instruction line as a string slice
/// 
/// # Returns
/// 
/// * (split_line, info_map) - the split line along with a HashMap 
/// with additional information (whether dest and jump were set, A instruction or C instruction) 
/// 
pub fn parse_line<'a>(line: &'a str) -> (Vec<&'a str>, HashMap<&'static str, bool>) {
    let mut split_line: Vec<&str>;
    let mut dest = true; 
    let mut jump = true;
    let mut info_map = HashMap::new();

    if line.starts_with('@') {
        let trimmed_line = line.trim_left_matches("@");
        split_line = trimmed_line.split(" ").collect();
        if split_line.len() > 1 {
            split_line.truncate(1);
        }
        info_map.insert("a_instruction", true);
    } else {
        let mut max_c_fields = 3; // C instructions have a maximum of 3 fields, but dest and jump are optional
        if !line.contains('=') {
            max_c_fields -= 1;
            dest = false; 
        }
        if !line.contains(';') {
            max_c_fields -= 1;
            jump = false;
        }
        info_map.insert("a_instruction", false);
        info_map.insert("dest", dest);
        info_map.insert("jump", jump);
        split_line = line.split(|c| c == '=' || c == ';' || c == ' ').collect();
        split_line.truncate(max_c_fields);
    }
    (split_line, info_map)
}

pub struct SymbolTable {
    pub symbol_map: HashMap<String, i32>
}

impl SymbolTable {
    /// Initializes a new SymbolTable using the file of
    /// predefined symbols
    pub fn new(predef_file: File) -> SymbolTable {
        let buf_reader = BufReader::new(predef_file);
        let mut symbol_map = HashMap::new();
        for line in buf_reader.lines() {
            let split_line: Vec<String> = line.unwrap().split(" ").map(|s| s.to_string()).collect();
            let symbol = (*(split_line.get(0).unwrap())).clone();
            let num = split_line.get(1).unwrap().parse::<i32>().unwrap();
            symbol_map.insert(symbol, num);
        }
        // loading symbols from file doesn't work for some reason...need to investigate
        symbol_map.insert("SP".to_string(), 0);
        symbol_map.insert("LCL".to_string(), 1);
        symbol_map.insert("ARG".to_string(), 2);
        symbol_map.insert("THIS".to_string(), 3);
        symbol_map.insert("THAT".to_string(), 4);
        symbol_map.insert("SCREEN".to_string(), 16384);
        symbol_map.insert("KBD".to_string(), 24576);
        for num in 0..16 {
            let r_symbol_str = format!("R{}", num);
            symbol_map.insert(r_symbol_str, num);
        }
        SymbolTable {
            symbol_map: symbol_map
        }
    }

    /// Makes two passes through an assembly code file
    /// and processes symbols
    /// 
    /// Arguments:
    /// 
    /// asm_file: the original assembly file before any processing
    /// intm_file: the intermediate file with all symbols replaced, and white/comments lines removed
    pub fn parse_file(&mut self, mut asm_file: File, mut intm_file: File) {
        let buf_reader = BufReader::new(asm_file.try_clone().unwrap());
        let mut line_num = 0;
        let mut next_mem = 16;
        // parse label symbols first
        for line in buf_reader.lines() {
            let unwrapped_line = line.unwrap();
            if unwrapped_line.is_empty() {
                continue;
            }
            line_num = self.parse_label_in_line(unwrapped_line.as_str(), line_num);
        }
        asm_file.seek(SeekFrom::Start(0)); // seek back to the beginning of the file
        let buf_reader = BufReader::new(asm_file.try_clone().unwrap());
        for line in buf_reader.lines() {
            let unwrapped_line = line.unwrap();
            if unwrapped_line.is_empty() {
                continue;
            }
            next_mem = self.parse_variable_in_line(unwrapped_line.as_str(), next_mem, intm_file.try_clone().unwrap())
        }
    }
    ///
    /// Parses the label symbols in a line of instruction 
    /// 
    /// Arguments:
    /// 
    /// line: the line string 
    /// line_num: the current line number, used for processing label symbols
    /// 
    /// Returns: the mutated line_num 
    fn parse_label_in_line(&mut self, line: &str, mut line_num: i32) -> i32 {
        // Assume that instruction lines would not start with an empty space
        if line.starts_with(|c| c == ' ' || c == '/') {
            return line_num;
        }
        if line.starts_with('(') {
            let split_line: Vec<&str> = line.split(|c| c == '(' || c ==')' || c == ' ').collect(); 
            let label = split_line[1].to_string(); // The second token contains the symbol 
            if !self.symbol_map.contains_key(&label) {
                self.symbol_map.insert(label, line_num); // consume the label
            }
            return line_num;
        } 
        line_num += 1;
        line_num 
    }
    ///
    /// Parses variable symbols in a line of instruction
    /// 
    /// Arguments:
    /// 
    /// line: the line literal
    /// next_mem: the next available memory location
    /// intm_file: an intermediate file with symbols replaced and blank/comment lines removed
    /// 
    /// Returns: the mutated next available memory location
    fn parse_variable_in_line(&mut self, line: &str, mut next_mem: i32, mut intm_file: File) -> i32 {
        // Assume that instruction lines would not start with an empty space
        if line.starts_with(|c: char| c == ' ' || c == '/' || c == '(') {
            return next_mem;
        }
        let mut writer = BufWriter::new(intm_file);
        if line.starts_with('@') {
            let split_line: Vec<&str> = line.split(|c| c == '@' || c == ' ').collect();
            let variable = split_line[1].to_string(); // second token contains the variable
            let var_clone = variable.clone();
            if !variable.parse::<i32>().is_ok() { // if the variable isn't a number (i.e. setting an address)
                if !self.symbol_map.contains_key(&variable) {
                    self.symbol_map.insert(variable, next_mem); // consume the variable
                    next_mem += 1;
                    // write to the intermediate file with the symbol replced
                }
                writer.write(format!("@{}\n", self.symbol_map.get(&var_clone).unwrap()).as_bytes());
            } else {
                let mut line_str = line.to_string();
                line_str.push('\n');
                writer.write(line_str.as_bytes());
            }
        } else {
            // write the C instruction as is to the intermediate file
            let mut line_str = line.to_string();
            line_str.push('\n');
            writer.write(line_str.as_bytes());
        }
        next_mem
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_a_instruction() {
        let (parsed_line, info_map) = parse_line("@100");
        println!("{:?}", parsed_line);
        assert_eq!(parsed_line, vec!["100"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), true);
    }

    #[test]
    fn parse_a_instruction_with_comment() {

        let (parsed_line, info_map) = parse_line("@100 // set a register to 100");
        println!("{:?}", parsed_line);
        assert_eq!(parsed_line, vec!["100"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), true);
    }

    #[test]
    fn parse_c_instruction() {
        let (parsed_line, info_map) = parse_line("D=D+M;JMP");
        assert_eq!(parsed_line, vec!["D", "D+M", "JMP"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), false);
        assert_eq!(*info_map.get("dest").unwrap(), true);
        assert_eq!(*info_map.get("jump").unwrap(), true);
    }


    #[test]
    fn parse_c_instruction_with_comments() {
        let (parsed_line, info_map) = parse_line("D=D+M;JMP // unconditional jump");
        assert_eq!(parsed_line, vec!["D", "D+M", "JMP"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), false);
        assert_eq!(*info_map.get("dest").unwrap(), true);
        assert_eq!(*info_map.get("jump").unwrap(), true);
    }

    #[test]
    fn parse_c_instruction_comp_only() {
        let (parsed_line, info_map) = parse_line("D+M");
        assert_eq!(parsed_line, vec!["D+M"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), false);
        assert_eq!(*info_map.get("dest").unwrap(), false);
        assert_eq!(*info_map.get("jump").unwrap(), false);
    }

    #[test]
    fn parse_c_instruction_comp_and_dest_only() {
        let (parsed_line, info_map) = parse_line("D=D+M");
        assert_eq!(parsed_line, vec!["D", "D+M"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), false);
        assert_eq!(*info_map.get("dest").unwrap(), true);
        assert_eq!(*info_map.get("jump").unwrap(), false);
    }

    #[test]
    fn parse_c_instruction_comp_and_jump_only() {
        let (parsed_line, info_map) = parse_line("D+M;JEQ");
        assert_eq!(parsed_line, vec!["D+M", "JEQ"]);
        assert_eq!(*info_map.get("a_instruction").unwrap(), false);
        assert_eq!(*info_map.get("dest").unwrap(), false);
        assert_eq!(*info_map.get("jump").unwrap(), true);
    }

    #[test]
    fn a_decode_test() {
        let decoder = ADecoder::new();
        assert_eq!(&decoder.decode(vec!["4"], &HashMap::new()), "0000000000000100");
    }

    #[test]
    fn a_decode_test_2() {
        let decoder = ADecoder::new();
        assert_eq!(&decoder.decode(vec!["100"], &HashMap::new()), "0000000001100100");
    }

    /// setup function for CDecoder
    fn c_decoder_setup() -> CDecoder {
        let dest_file = File::open("dest_file.txt").unwrap();
        let comp_file = File::open("comp_file.txt").unwrap();
        let jump_file = File::open("jump_file.txt").unwrap();

        CDecoder::new(dest_file, comp_file, jump_file)
    }

    #[test]
    fn c_decode_no_jump() {
        let decoder = c_decoder_setup();
        let mut info_map = HashMap::new();
        info_map.insert("dest", true);
        info_map.insert("jump", false);
        assert_eq!(&decoder.decode(vec!["MD", "D+1"], &info_map), "1110011111011000");
    }

    #[test]
    fn c_decode_no_jump_and_no_dest() {
        let decoder = c_decoder_setup();
        let mut info_map = HashMap::new();
        info_map.insert("dest", false);
        info_map.insert("jump", false);
        assert_eq!(&decoder.decode(vec!["D+1"], &info_map), "1110011111000000");
    }

    #[test]
    fn c_decode_no_dest() {
        let decoder = c_decoder_setup();
        let mut info_map = HashMap::new();
        info_map.insert("dest", false);
        info_map.insert("jump", true);
        assert_eq!(&decoder.decode(vec!["D+1", "JLE"], &info_map), "1110011111000110");
    }

    #[test]
    fn c_decode_m_not_a() {
        let decoder = c_decoder_setup();
        let mut info_map = HashMap::new();
        info_map.insert("dest", true);
        info_map.insert("jump", true);
        assert_eq!(&decoder.decode(vec!["M", "M+1", "JEQ"], &info_map), "1111110111001010");
    }

    #[test]
    fn c_decode_unconditional_jump() {
        let decoder = c_decoder_setup();
        let mut info_map = HashMap::new();
        info_map.insert("dest", false);
        info_map.insert("jump", true);
        assert_eq!(&decoder.decode(vec!["0", "JMP"], &info_map), "1110101010000111");
    }

    fn symbol_table_setup() -> SymbolTable {
        let file = File::open("predefined_symbols.txt").unwrap();
        SymbolTable::new(file)
    }

    #[test]
    fn test_label_parsing() {
        let mut symbol_table = symbol_table_setup();
        symbol_table.parse_label_in_line("(END)", 10);
        assert_eq!(*symbol_table.symbol_map.get(&"END".to_string()).unwrap(), 10);
    }

    #[test]
    fn test_variable_parsing() {
        let mut symbol_table = symbol_table_setup();
        symbol_table.parse_variable_in_line("@start // start var", 10, File::create("blah").unwrap());
        assert_eq!(*symbol_table.symbol_map.get(&"start".to_string()).unwrap(), 10);
    }

    #[test]
    fn test_non_variable_parsing() {
        let mut symbol_table = symbol_table_setup();
        symbol_table.parse_variable_in_line("@10 // start var", 10, File::create("blah").unwrap());
        assert_eq!(symbol_table.symbol_map.contains_key(&"10".to_string()), false);
    }

    #[test]
    fn test_predefined_symbol() {
        let mut symbol_table = symbol_table_setup();
        assert_eq!(*symbol_table.symbol_map.get(&"SCREEN".to_string()).unwrap(), 16384);
        assert_eq!(*symbol_table.symbol_map.get(&"KBD".to_string()).unwrap(), 24576);
        assert_eq!(*symbol_table.symbol_map.get(&"SP".to_string()).unwrap(), 0);
    }
    #[test]
    fn test_file_parsing() {
        let mut symbol_table = symbol_table_setup();
        let mut asm_file = File::open("symbol_test.txt").unwrap();
        let mut intm_file = File::create("intm1.txt").unwrap();
        symbol_table.parse_file(asm_file, intm_file);
        assert_eq!(*symbol_table.symbol_map.get(&"sum".to_string()).unwrap(), 16);
        assert_eq!(*symbol_table.symbol_map.get(&"HELLO".to_string()).unwrap(), 1);
        assert_eq!(*symbol_table.symbol_map.get(&"i".to_string()).unwrap(), 17);
        assert_eq!(*symbol_table.symbol_map.get(&"END".to_string()).unwrap(), 2);
        assert_eq!(*symbol_table.symbol_map.get(&"blah".to_string()).unwrap(), 18);
    }

    
    #[test]
    fn test_file_parsing_2() {
        let mut symbol_table = symbol_table_setup();
        let mut asm_file = File::open("symbol_test_2.txt").unwrap();
        let mut intm_file = File::create("intm2.txt").unwrap();
        symbol_table.parse_file(asm_file, intm_file);
        assert_eq!(*symbol_table.symbol_map.get(&"sum".to_string()).unwrap(), 17);
        assert_eq!(*symbol_table.symbol_map.get(&"LOOP".to_string()).unwrap(), 4);
        assert_eq!(*symbol_table.symbol_map.get(&"i".to_string()).unwrap(), 16);
        assert_eq!(*symbol_table.symbol_map.get(&"STOP".to_string()).unwrap(), 8);
        assert_eq!(*symbol_table.symbol_map.get(&"R0".to_string()).unwrap(), 0);
        assert_eq!(*symbol_table.symbol_map.get(&"END".to_string()).unwrap(), 11);
    }

    #[test]
    fn test_file_parsing_with_predefined() {
        let mut symbol_table = symbol_table_setup();
        let mut asm_file = File::open("symbol_test_3.txt").unwrap();
        let mut intm_file = File::create("intm3.txt").unwrap();
        symbol_table.parse_file(asm_file, intm_file);
        assert_eq!(*symbol_table.symbol_map.get(&"i".to_string()).unwrap(), 16);
    }

}
