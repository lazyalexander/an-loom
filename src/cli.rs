//! Argument handling and presentation only; domain and execution logic live elsewhere.
use crate::{Result, fail};
use serde_json::{Value, json};
use std::path::Path;

const HELP: &str = "an-loom - inspectable spool development and delivery

  an-loom init DIRECTORY
  an-loom init-native DIRECTORY
  an-loom init-package DIRECTORY
  an-loom check PROJECT
  an-loom develop-native TASK HOST_CLI NEW_JOB NEW_DELIVERY [--approve SHA256]
  an-loom develop PROJECT HOST_CLI TOOLCHAIN_JSON NEW_JOB [--approve SHA256]
  an-loom receive-build PROJECT HOST_JOB
  an-loom declare PROJECT
  an-loom build PROJECT
  an-loom pack PROJECT
  an-loom verify ARCHIVE
  an-loom export-spool ARCHIVE NEW_DIRECTORY
  an-loom export-native-spool ARCHIVE NEW_DIRECTORY
  an-loom export-native-spool-v2 ARCHIVE NEW_DIRECTORY
  an-loom export-native-spool-v3 ARCHIVE NEW_DIRECTORY
  an-loom release PROJECT NEW_DIRECTORY
  an-loom verify-release DIRECTORY
  an-loom task PROJECT NEW_DIRECTORY
  an-loom receive-task TASK EDITABLE_PATH CANDIDATE
  an-loom check-task TASK
  an-loom receive-check TASK HOST_JOB
  an-loom pack-task TASK NEW_DIRECTORY
  an-loom revoke-task TASK

init-native copies the reviewed native text reference. check, pack and verify
read its acceptance records and dependency labels without running or downloading
them. develop previews a host build; --approve submits the exact reviewed plan once.
build snapshots package bytes. release creates a local delivery, not a remote
publication or runtime grant. The host owns registration and controlled execution.
task writes a kit assignment outside the project. receive-task copies one
candidate into that assignment and does not change the project or any release.
revoke-task rejects a later candidate. check-task freezes the whole candidate and
prepares a nonexecuting host profile. receive-check validates a retained host job
with confirmed termination and formal output acceptance. pack-task publishes the
same frozen archive after revalidation. None of these commands invokes a model or retries an unknown result.
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
        ("init-native", [root]) => {
            crate::project::scaffold_native(Path::new(root))?;
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
        ("export-native-spool-v2", [archive, destination]) => {
            crate::adapters::host::native_spool::export_v2(
                Path::new(archive),
                Path::new(destination),
            )?
        }
        ("export-native-spool-v3", [archive, destination]) => {
            crate::adapters::host::native_spool::export_v3(
                Path::new(archive),
                Path::new(destination),
            )?
        }
        ("export-native-spool", [archive, destination]) => {
            crate::adapters::host::native_spool::export(Path::new(archive), Path::new(destination))?
        }
        ("release", [root, destination]) => serde_json::to_value(crate::release::create(
            Path::new(root),
            Path::new(destination),
        )?)?,
        ("verify-release", [root]) => {
            serde_json::to_value(crate::release::verify(Path::new(root))?)?
        }
        ("task", [project, destination]) => serde_json::to_value(crate::task::prepare(
            Path::new(project),
            Path::new(destination),
        )?)?,
        ("receive-task", [task, editable, candidate]) => serde_json::to_value(
            crate::task::receive(Path::new(task), editable, Path::new(candidate))?,
        )?,
        ("check-task", [task]) => serde_json::to_value(crate::task::check(Path::new(task))?)?,
        ("receive-check", [task, evidence]) => serde_json::to_value(crate::task::receive_check(
            Path::new(task),
            Path::new(evidence),
        )?)?,
        ("pack-task", [task, destination]) => serde_json::to_value(crate::task::pack_candidate(
            Path::new(task),
            Path::new(destination),
        )?)?,
        ("revoke-task", [task]) => {
            crate::task::revoke(Path::new(task))?;
            json!({"revoked":true,"model_invoked":false})
        }
        ("develop-native", [task, host, job, delivery]) => crate::adapters::develop_native(
            Path::new(task),
            Path::new(host),
            Path::new(job),
            Path::new(delivery),
            None,
        )?,
        ("develop-native", [task, host, job, delivery, flag, hash]) if flag == "--approve" => {
            crate::adapters::develop_native(
                Path::new(task),
                Path::new(host),
                Path::new(job),
                Path::new(delivery),
                Some(hash),
            )?
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
