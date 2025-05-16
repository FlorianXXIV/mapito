use std::io;

pub fn confirm_input() -> bool {
    println!("proceed? [Y,n]");
    let stdin = io::stdin();
    let buf = &mut String::new();
    let _ = stdin.read_line(buf);
    let chars: Vec<char> = buf.chars().collect();

    let request: char = match chars.first() {
        Some(c) => *c,
        None => 'y',
    };

    match request {
        'y' | 'Y' | '\n' => true,
        'n' | 'N' => false,
        _ => panic!("invalid option"),
    }
}
/// Reads one line from stdin and returns it as sanitized string
pub fn read_line_to_string() -> String {
    let buf = &mut String::new();
    io::stdin().read_line(buf).expect("read_line");
    buf.to_string().replace("\n", "").replace("\"", "")
}
