use std::path::PathBuf;

use acvm::ProofSystemCompiler;
use clap::ArgMatches;
use noirc_abi::input_parser::Format;
use std::path::Path;

use super::execute_cmd::extract_public_inputs;
use super::{create_named_dir, write_inputs_to_file, write_to_file};
use crate::{
    constants::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("prove").unwrap();
    let proof_name = args.value_of("proof_name").unwrap();
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    let proof_path = prove(proof_name, show_ssa, allow_warnings)?;

    println!("Proof successfully created and located at {}", proof_path.display());
    println!("{:?}", std::fs::canonicalize(&proof_path));
    Ok(())
}

fn prove(proof_name: &str, show_ssa: bool, allow_warnings: bool) -> Result<PathBuf, CliError> {
    let curr_dir = std::env::current_dir().unwrap();

    let mut proof_dir = PathBuf::new();
    proof_dir.push(PROOFS_DIR);

    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    prove_with_path(proof_name, curr_dir, proof_path, show_ssa, allow_warnings)
}

pub fn prove_with_path<P: AsRef<Path>>(
    proof_name: &str,
    program_dir: P,
    proof_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<PathBuf, CliError> {
    let compiled_program =
        super::compile_cmd::compile_circuit(program_dir.as_ref(), show_ssa, allow_warnings)?;
    let (_, solved_witness) = super::execute_cmd::execute_program(&program_dir, &compiled_program)?;

    // We allow the user to optionally not provide a value for the circuit's return value, so this may be missing from
    // `witness_map`. We must then decode these from the circuit's witness values.
    let public_inputs = extract_public_inputs(&compiled_program, &solved_witness)?;

    // Write public inputs into Verifier.toml
    write_inputs_to_file(&public_inputs, &program_dir, VERIFIER_INPUT_FILE, Format::Toml)?;

    let backend = crate::backends::ConcreteBackend;
    let proof = backend.prove_with_meta(compiled_program.circuit, solved_witness);

    let proof_path = write_proof_to_file(proof, proof_name, proof_dir);

    Ok(proof_path)
}

pub fn write_proof_to_file<P: AsRef<Path>>(
    proof: Vec<u8>,
    proof_name: &str,
    proof_dir: P,
) -> PathBuf {
    let mut proof_path = create_named_dir(proof_dir.as_ref(), "proof");
    proof_path.push(proof_name);
    proof_path.set_extension(PROOF_EXT);

    write_to_file(hex::encode(proof).as_bytes(), &proof_path);

    proof_path
}
