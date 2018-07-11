pub fn parse_line(line: &str) -> (Vec<&str>, bool) {
    let mut a_instruction: bool;
    let mut split_line: Vec<&str>;

    if line.starts_with('@') {
        a_instruction = true;
        let trimmed_line = line.trim_left_matches("@");
        split_line = trimmed_line.split(" ").collect();
        if split_line.len() > 1 {
            split_line.truncate(1);
        }
    } else {
        a_instruction = false;
        let mut max_c_fields = 3;
        if !line.contains('=') {
            max_c_fields -= 1;
        }
        if !line.contains(';') {
            max_c_fields -= 1;
        }
        split_line = line.split(|c| c == '=' || c == ';' || c == ' ').collect();
        split_line.truncate(max_c_fields);
    }
    (split_line, a_instruction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_a_instruction() {
        let (parsed_line, a_instruction) = parse_line("@100");
        println!("{:?}", parsed_line);
        assert_eq!(parsed_line, vec!["100"]);
        assert_eq!(a_instruction, true);
    }

    #[test]
    fn parse_a_instruction_with_comment() {

        let (parsed_line, a_instruction) = parse_line("@100 // set a register to 100");
        println!("{:?}", parsed_line);
        assert_eq!(parsed_line, vec!["100"]);
        assert_eq!(a_instruction, true);
    }

    #[test]
    fn parse_c_instruction() {
        let (parsed_line, a_instruction) = parse_line("D=D+M;JMP");
        assert_eq!(parsed_line, vec!["D", "D+M", "JMP"]);
        assert_eq!(a_instruction, false);
    }


    #[test]
    fn parse_c_instruction_with_comments() {
        let (parsed_line, a_instruction) = parse_line("D=D+M;JMP // unconditional jump");
        assert_eq!(parsed_line, vec!["D", "D+M", "JMP"]);
        assert_eq!(a_instruction, false);
    }

    #[test]
    fn parse_c_instruction_comp_only() {
        let (parsed_line, a_instruction) = parse_line("D+M");
        assert_eq!(parsed_line, vec!["D+M"]);
        assert_eq!(a_instruction, false);
    }

    #[test]
    fn parse_c_instruction_comp_and_dest_only() {
        let (parsed_line, a_instruction) = parse_line("D=D+M");
        assert_eq!(parsed_line, vec!["D", "D+M"]);
        assert_eq!(a_instruction, false);
    }

    #[test]
    fn parse_c_instruction_comp_and_jump_only() {
        let (parsed_line, a_instruction) = parse_line("D+M;JEQ");
        assert_eq!(parsed_line, vec!["D+M", "JEQ"]);
        assert_eq!(a_instruction, false);
    }
}