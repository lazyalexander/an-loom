use super::support::{Result, hash};
use an_loom::plan::{DevelopmentPlan, HostCall};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn approvals_bind_the_project_and_refuse_unsupported_execution() -> Result {
    let source = b"fn main() {}";
    let tools = b"{}";
    let preview = json!({"approval_sha256":"a".repeat(64),"plan":{
        "schema":"an-rust-build-plan-v1","source_sha256":hash(source),
        "toolchain_profile_sha256":hash(tools),"supported":true}});
    let command = HostCall {
        executable: "host".into(),
        args: vec!["build-kit".into()],
        cwd: "project".into(),
        environment: BTreeMap::new(),
    };
    let plan = DevelopmentPlan::assemble(
        b"project-one",
        source,
        tools,
        "b".repeat(64),
        preview.clone(),
        command.clone(),
    )?;
    let digest = plan.digest()?;
    plan.authorize(&digest)?;
    assert!(plan.authorize(&"0".repeat(64)).is_err());
    let changed = DevelopmentPlan::assemble(
        b"project-two",
        source,
        tools,
        "b".repeat(64),
        preview.clone(),
        command.clone(),
    )?;
    assert!(changed.authorize(&digest).is_err());
    assert!(
        DevelopmentPlan::assemble(
            b"project-one",
            b"changed source",
            tools,
            "b".repeat(64),
            preview.clone(),
            command.clone()
        )
        .is_err()
    );
    let mut unsupported = preview;
    unsupported["plan"]["supported"] = json!(false);
    let plan = DevelopmentPlan::assemble(
        b"project-one",
        source,
        tools,
        "b".repeat(64),
        unsupported,
        command,
    )?;
    assert!(plan.authorize(&plan.digest()?).is_err());
    Ok(())
}
