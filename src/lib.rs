use std::fmt;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, BufRead};

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
        instruct_str.push_str(format!("{:b}", address).as_str());
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
        let mut dest_bin: String;
        let mut comp_bin: String;
        let mut jump_bin: String;

        if *info_map.get("dest").unwrap() {
            let dest = instruct_fields.get(0).unwrap().to_string();
            dest_bin = self.dest_map.get(&dest).unwrap().to_string();
            comp_index = 1;
        } else {
            dest_bin = "000".to_string();
        }
        let comp = instruct_fields.get(comp_index).unwrap().to_string();
        let comp_bin = self.comp_map.get(&comp).unwrap().clone().to_string();
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
pub fn parse_line(line: &str) -> (Vec<&str>, HashMap<&str, bool>) {
    let mut a_instruction: bool; // true if line is an A instruction
    let mut split_line: Vec<&str>;
    let mut dest = true; 
    let mut jump = true;
    let mut info_map = HashMap::new();

    if line.starts_with('@') {
        a_instruction = true;
        let trimmed_line = line.trim_left_matches("@");
        split_line = trimmed_line.split(" ").collect();
        if split_line.len() > 1 {
            split_line.truncate(1);
        }
        info_map.insert("a_instruction", true);
    } else {
        a_instruction = false;
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
        assert_eq!(&decoder.decode(vec!["4"], &HashMap::new()), "0100");
    }

    #[test]
    fn a_decode_test_2() {
        let decoder = ADecoder::new();
        assert_eq!(&decoder.decode(vec!["100"], &HashMap::new()), "01100100");
    }
}