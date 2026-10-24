use std::{collections::BTreeSet, fs, io::{self, Read, Write}};

// Business behavior lives here. Neither the host nor Loom implements it.
fn transform(input: &str) -> String {
    let lines: BTreeSet<_> = input.lines().map(str::trim).filter(|s| !s.is_empty()).collect();
    if lines.is_empty() { String::new() } else { lines.into_iter().collect::<Vec<_>>().join("\n") + "\n" }
}

fn main() -> io::Result<()> {
    let mut input = String::new();
    fs::File::open("input.txt")?.take(8193).read_to_string(&mut input)?;
    if input.len() > 8192 { return Err(io::Error::other("input exceeds 8192 bytes")); }
    let result = transform(&input);
    let mut output = fs::OpenOptions::new().write(true).create(true).truncate(true).open("output.txt")?;
    output.write_all(result.as_bytes())?;
    output.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn trims_deduplicates_and_sorts() { assert_eq!(transform(" beta\nalpha\nbeta\n\n"), "alpha\nbeta\n"); }
    #[test]
    fn empty_input_stays_empty() { assert_eq!(transform(" \n\n"), ""); }
    #[test]
    fn unicode_is_preserved() { assert_eq!(transform(" café\ncafé\n"), "café\n"); }
}
