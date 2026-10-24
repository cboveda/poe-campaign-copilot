#[path="../../src-tauri/src/bug_report.rs"]
mod bug_report;
#[path="../../crates/input_log/src/assembler.rs"]
mod assembler;
fn main() {
    let url = bug_report::bug_report_url();
    assert_eq!(url, "https://github.com/cboveda/poe-campaign-copilot/issues/new?template=bug_report.yml");
    assert!(!url.contains("details="));
    println!("privacy fix: outgoing URL is a fixed form with no diagnostic inputs");
    let mut a = assembler::LineAssembler::new();
    let mut line = vec![b'x'; 128*1024]; line.push(b'\n');
    let lines=a.feed(&line);
    assert_eq!(lines[0].len(),128*1024);
    println!("line cap: newline-terminated 128KiB line accepted despite advertised 64KiB cap");
    let mut a = assembler::LineAssembler::new();
    a.feed(&vec![b'x'; 65*1024]);
    let tail=a.feed(b"2026/10/24 12:00:00 100 abc [INFO Client 1] [LOADING SCREEN] (Fake) Duration = 1 seconds\n");
    assert!(tail[0].starts_with("2026/10/24"));
    println!("resynchronization: oversized line remainder reinterpreted as a new log line without an intervening newline");
}
