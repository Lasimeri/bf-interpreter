/// Errors from BF interpretation.
#[derive(Debug)]
pub enum BfError {
    UnmatchedClose(usize),
    UnmatchedOpen(usize),
    EmptyProgram,
}

impl std::fmt::Display for BfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BfError::UnmatchedClose(pos) => write!(f, "unmatched closing bracket at position {}", pos),
            BfError::UnmatchedOpen(pos) => write!(f, "unmatched opening bracket at position {}", pos),
            BfError::EmptyProgram => write!(f, "empty program"),
        }
    }
}

const TAPE_SIZE: usize = 30_000;

/// Strip comments (`;` to EOL) and filter to BF operators.
pub fn parse_program(source: &str) -> Vec<char> {
    let mut filtered = String::new();
    for line in source.lines() {
        if let Some(pos) = line.find(';') {
            filtered.push_str(&line[..pos]);
        } else {
            filtered.push_str(line);
        }
        filtered.push('\n');
    }
    filtered
        .chars()
        .filter(|c| matches!(c, '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']'))
        .collect()
}

/// Build bracket jump table.
pub fn build_jumps(program: &[char]) -> Result<Vec<usize>, BfError> {
    let n = program.len();
    let mut jumps = vec![0; n];
    let mut stack = Vec::new();
    for (i, &c) in program.iter().enumerate() {
        match c {
            '[' => stack.push(i),
            ']' => {
                let start = stack.pop().ok_or(BfError::UnmatchedClose(i))?;
                jumps[start] = i;
                jumps[i] = start;
            }
            _ => {}
        }
    }
    if let Some(&pos) = stack.first() {
        return Err(BfError::UnmatchedOpen(pos));
    }
    Ok(jumps)
}

/// Run a BF program with provided input, returning output bytes.
pub fn run(source: &str, input: &[u8]) -> Result<Vec<u8>, BfError> {
    let program = parse_program(source);
    if program.is_empty() {
        return Err(BfError::EmptyProgram);
    }
    let jumps = build_jumps(&program)?;
    let n = program.len();

    let mut tape = [0u8; TAPE_SIZE];
    let mut ptr: usize = 0;
    let mut pc: usize = 0;
    let mut input_pos: usize = 0;
    let mut output = Vec::new();

    while pc < n {
        match program[pc] {
            '>' => {
                ptr += 1;
                if ptr >= TAPE_SIZE { ptr = 0; }
            }
            '<' => {
                if ptr == 0 { ptr = TAPE_SIZE - 1; } else { ptr -= 1; }
            }
            '+' => { tape[ptr] = tape[ptr].wrapping_add(1); }
            '-' => { tape[ptr] = tape[ptr].wrapping_sub(1); }
            '.' => { output.push(tape[ptr]); }
            ',' => {
                if input_pos < input.len() {
                    tape[ptr] = input[input_pos];
                    input_pos += 1;
                } else {
                    tape[ptr] = 0;
                }
            }
            '[' => {
                if tape[ptr] == 0 { pc = jumps[pc]; }
            }
            ']' => {
                if tape[ptr] != 0 { pc = jumps[pc]; }
            }
            _ => {}
        }
        pc += 1;
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        let program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let output = run(program, &[]).unwrap();
        assert_eq!(String::from_utf8_lossy(&output), "Hello World!\n");
    }

    #[test]
    fn test_cat() {
        let program = ",[.,]";
        let output = run(program, b"hi").unwrap();
        assert_eq!(&output, b"hi");
    }

    #[test]
    fn test_empty_program() {
        let result = run("", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unmatched_close() {
        let result = run("]", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_unmatched_open() {
        let result = run("[", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_comment_stripping() {
        let program = "+++ ; this is a comment\n---";
        let parsed = parse_program(program);
        assert_eq!(parsed, vec!['+', '+', '+', '-', '-', '-']);
    }
}
