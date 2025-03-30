use note::parse_note;
use std::fs;

fn main() {
    let file = std::fs::read_to_string("test.html").unwrap();

    let note = parse_note(&file);
    std::fs::write("test.json", serde_json::to_string(&note).unwrap());
}
