use anyhow::{anyhow, Result};
use cairo_lang_compiler::diagnostics::DiagnosticsError;
use indicatif::HumanDuration;

use crate::compiler::db::build_scarb_root_database;
use crate::compiler::CompilationUnit;
use crate::core::workspace::Workspace;
use crate::ops;
use crate::ui::Status;

#[tracing::instrument(skip_all, level = "debug")]
pub fn compile(ws: &Workspace<'_>) -> Result<()> {
    let resolve = ops::resolve_workspace(ws)?;
    let compilation_units = ops::generate_compilation_units(&resolve, ws)?;

    for unit in compilation_units {
        compile_unit(unit, ws)?;
    }

    let elapsed_time = HumanDuration(ws.config().elapsed_time());
    ws.config().ui().print(Status::new(
        "Finished",
        &format!("release target(s) in {elapsed_time}"),
    ));

    Ok(())
}

fn compile_unit(unit: CompilationUnit, ws: &Workspace<'_>) -> Result<()> {
    let package_name = unit.main_package_id.name.clone();

    ws.config()
        .ui()
        .print(Status::new("Compiling", &unit.name()));

    let mut db = build_scarb_root_database(&unit, ws)?;

    ws.config()
        .compilers()
        .compile(unit, &mut db, ws)
        .map_err(|err| {
            if !suppress_error(&err) {
                ws.config().ui().anyhow(&err);
            }

            anyhow!("could not compile `{package_name}` due to previous error")
        })?;

    Ok(())
}

fn suppress_error(err: &anyhow::Error) -> bool {
    matches!(err.downcast_ref(), Some(&DiagnosticsError))
}
