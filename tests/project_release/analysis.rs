use super::support::Result;

#[test]
fn source_signals_reject_known_incompatible_operations_without_claiming_completeness() -> Result {
    let evidence = an_loom::spool::analyze(include_bytes!("../../examples/file-list/kit/main.rs"))?;
    assert!(!evidence.unknowns.is_empty());
    for source in [
        "fn main() { std::net::TcpStream::connect(\"server\"); }",
        "fn main() { unsafe {} }",
        "fn main( {",
    ] {
        assert!(an_loom::spool::analyze(source.as_bytes()).is_err());
    }
    Ok(())
}
