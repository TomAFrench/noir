use std::path::PathBuf;

use acvm::ProofSystemCompiler;

use clap::Args;

use std::path::Path;

use crate::{
    cli::execute_cmd::save_witness_to_dir,
    constants::{ACIR_EXT, TARGET_DIR},
    errors::CliError,
    resolver::Resolver,
};

use super::{add_std_lib, create_named_dir, write_to_file, NargoConfig};

/// Compile the program and its secret execution trace into ACIR format
#[derive(Debug, Clone, Args)]
pub(crate) struct CompileCommand {
    /// The name of the ACIR file
    circuit_name: String,

    /// Solve the witness and write it to file along with the ACIR
    #[arg(short, long)]
    witness: bool,

    /// Issue a warning for each unused variable instead of an error
    #[arg(short, long)]
    allow_warnings: bool,
}

pub(crate) fn run(args: CompileCommand, config: NargoConfig) -> Result<(), CliError> {
    let mut circuit_path = config.program_dir.clone();
    circuit_path.push(TARGET_DIR);

    generate_circuit_and_witness_to_disk(
        &args.circuit_name,
        config.program_dir,
        circuit_path,
        args.witness,
        args.allow_warnings,
    )
    .map(|_| ())
}

#[allow(deprecated)]
pub fn generate_circuit_and_witness_to_disk<P: AsRef<Path>>(
    circuit_name: &str,
    program_dir: P,
    circuit_dir: P,
    generate_witness: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program = compile_circuit(program_dir.as_ref(), false, allow_warnings)?;
    let serialized = compiled_program.circuit.to_bytes();

    let mut circuit_path = create_named_dir(circuit_dir.as_ref(), "build");
    circuit_path.push(circuit_name);
    circuit_path.set_extension(ACIR_EXT);
    let path = write_to_file(serialized.as_slice(), &circuit_path);
    println!("Generated ACIR code into {path}");
    println!("{:?}", std::fs::canonicalize(&circuit_path));

    if generate_witness {
        let (_, solved_witness) =
            super::execute_cmd::execute_program(program_dir, &compiled_program)?;

        circuit_path.pop();
        save_witness_to_dir(solved_witness, circuit_name, &circuit_path)?;
    }

    Ok(circuit_path)
}

pub fn compile_circuit<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<noirc_driver::CompiledProgram, CliError> {
    let backend = crate::backends::ConcreteBackend;
    let mut driver = Resolver::resolve_root_config(program_dir.as_ref(), backend.np_language())?;
    add_std_lib(&mut driver);

    driver
        .into_compiled_program(backend.np_language(), show_ssa, allow_warnings)
        .map_err(|_| std::process::exit(1))
}
