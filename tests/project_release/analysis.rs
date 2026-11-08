use super::support::Result;

#[test]
fn source_signals_preserve_syntax_validation_and_leave_authority_to_the_runtime() -> Result {
    let evidence = an_loom::spool::analyze(include_bytes!("../../examples/file-list/kit/main.rs"))?;
    assert!(!evidence.unknowns.is_empty());
    for source in [
        "fn main() { std::net::TcpStream::connect(\"server\"); }",
        "fn main() { unsafe {} }",
        "unsafe extern \"C\" { fn external(); } fn main() {}",
        "fn main() { std::process::Command::new(\"command\"); }",
    ] {
        assert!(
            !an_loom::spool::analyze(source.as_bytes())?
                .unknowns
                .is_empty()
        );
    }
    assert!(an_loom::spool::analyze(b"fn main( {").is_err());
    Ok(())
}
