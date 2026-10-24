//! Argument handling and presentation only; domain and execution logic live elsewhere.
use crate::{Result, fail};
use serde_json::{Value, json};
use std::path::Path;

const HELP: &str = "an-loom - inspectable spool development and delivery

  an-loom init DIRECTORY
  an-loom init-package DIRECTORY
  an-loom check PROJECT
  an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB [--approve SHA256]
  an-loom receive-build PROJECT HOST_JOB
  an-loom declare PROJECT
  an-loom build PROJECT
  an-loom pack PROJECT
  an-loom verify ARCHIVE
  an-loom export-spool ARCHIVE NEW_DIRECTORY
  an-loom release PROJECT NEW_DIRECTORY
  an-loom verify-release DIRECTORY

develop previews a host build; --approve submits the exact reviewed plan once.
build snapshots package bytes. release creates a local delivery, not a remote
publication or runtime grant. The host owns registration and controlled execution.
";

pub(crate) fn run(args: Vec<String>) -> Result<()> {
    if args.is_empty() || matches!(args.as_slice(), [arg] if arg == "--help" || arg == "-h") {
        println!("{HELP}");
        return Ok(());
    }
    let (command, args) = args.split_first().ok_or_else(|| fail("missing command"))?;
    let result: Value = match (command.as_str(), args) {
        ("init", [root]) | ("init-package", [root]) => {
            crate::project::scaffold(Path::new(root), command == "init")?;
            json!({"created":root,"model_generated":false})
        }
        ("check", [root]) => {
            let checked = crate::package::inspect_project(Path::new(root))?;
            json!({"package":checked.manifest.package_identity(),"selected_files":checked.files.len(),"checked":true})
        }
        ("build", [root]) => {
            let build = crate::package::build_project(Path::new(root))?;
            json!({"bundle":build.manifest,"directory":build.output})
        }
        ("pack", [root]) => json!({"archive":crate::package::pack_project(Path::new(root))?}),
        ("verify", [archive]) => {
            serde_json::to_value(crate::package::verify_archive(Path::new(archive))?)?
        }
        ("receive-build", [root, job]) => {
            crate::build_artifacts::receive(Path::new(root), Path::new(job))?;
            json!({"received":true})
        }
        ("declare", [root]) => {
            crate::spool::declare(Path::new(root))?;
            json!({"declared":true,"granted":false})
        }
        ("export-spool", [archive, destination]) => {
            crate::spool::export(Path::new(archive), Path::new(destination))?;
            json!({"exported":true,"registered":false})
        }
        ("release", [root, destination]) => serde_json::to_value(crate::release::create(
            Path::new(root),
            Path::new(destination),
        )?)?,
        ("verify-release", [root]) => {
            serde_json::to_value(crate::release::verify(Path::new(root))?)?
        }
        ("develop", [root, host, tools, job]) => crate::adapters::develop(
            Path::new(root),
            Path::new(host),
            Path::new(tools),
            Path::new(job),
            None,
        )?,
        ("develop", [root, host, tools, job, flag, approval]) if flag == "--approve" => {
            crate::adapters::develop(
                Path::new(root),
                Path::new(host),
                Path::new(tools),
                Path::new(job),
                Some(approval),
            )?
        }
        _ => return Err(fail(format!("invalid command arguments\n{HELP}"))),
    };
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
