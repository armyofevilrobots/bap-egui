use glob::glob;
use regex::Regex;

pub fn scan_ports() -> Vec<String> {
    let ttyregex =
        Regex::new(r"^/dev/tty(ACM\d{1}|USB\d{1}|BOTAPLOT\d{1})").expect("Invalid regex");
    let mut found_ports = vec![];
    for blob_entry in glob("/dev/tty*").expect("Invalid glob pattern") {
        if let Ok(entry) = blob_entry {
            let entry = entry
                .as_path()
                .to_str()
                .expect("Cannot convert path to utf8");
            if ttyregex.is_match(entry) {
                let port_path = &format!("serial://{}", entry);
                found_ports.push(port_path.clone());
            }
        }
    }
    found_ports
}
