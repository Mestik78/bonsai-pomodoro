use std::process::Command;

fn main() {
    let output = Command::new("script")
        .args(["-q", "-c", "cbonsai -p", "/dev/null"])
        .output()
        .unwrap();
    
    let raw = String::from_utf8_lossy(&output.stdout).to_string();
    let cleaned = raw
        .replace("\x1b(B", "")
        .replace("\x1b)0", "")
        .replace("\x1b[?25h", "")
        .replace("\x1b[?25l", "")
        .replace("\x1b[H", "")
        .replace("\x1b[2J", "")
        .replace("\r\n", "\n")
        .replace("\r", "");
    
    println!("Lines: {}", cleaned.lines().count());
    for (i, line) in cleaned.lines().enumerate().take(5) {
        println!("{}: {:?}", i, line);
    }
}
