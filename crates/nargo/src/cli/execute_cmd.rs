use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use acvm::acir::native_types::Witness;
use acvm::{FieldElement, PartialWitnessGenerator};
use clap::ArgMatches;
use iter_extended::{try_btree_map, vecmap};
use noirc_abi::errors::AbiError;
use noirc_abi::input_parser::{Format, InputValue};
use noirc_abi::{decode_value, encode_value, AbiParameter, MAIN_RETURN_NAME};
use noirc_driver::CompiledProgram;

use super::{create_named_dir, read_inputs_from_file, write_to_file, InputMap, WitnessMap};
use crate::{
    cli::compile_cmd::compile_circuit,
    constants::{PROVER_INPUT_FILE, TARGET_DIR, WITNESS_EXT},
    errors::CliError,
};

pub(crate) fn run(args: ArgMatches) -> Result<(), CliError> {
    let args = args.subcommand_matches("execute").unwrap();
    let witness_name = args.value_of("witness_name");
    let show_ssa = args.is_present("show-ssa");
    let allow_warnings = args.is_present("allow-warnings");
    let program_dir =
        args.value_of("path").map_or_else(|| std::env::current_dir().unwrap(), PathBuf::from);

    let (return_value, solved_witness) = execute_with_path(&program_dir, show_ssa, allow_warnings)?;

    println!("Circuit witness successfully solved");
    if let Some(return_value) = return_value {
        println!("Circuit output: {return_value:?}");
    }
    if let Some(witness_name) = witness_name {
        let mut witness_dir = program_dir;
        witness_dir.push(TARGET_DIR);

        let witness_path = save_witness_to_dir(solved_witness, witness_name, witness_dir)?;

        println!("Witness saved to {}", witness_path.display());
    }
    Ok(())
}

fn execute_with_path<P: AsRef<Path>>(
    program_dir: P,
    show_ssa: bool,
    allow_warnings: bool,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
    let compiled_program = compile_circuit(&program_dir, show_ssa, allow_warnings)?;

    // Parse the initial witness values from Prover.toml
    let inputs_map = read_inputs_from_file(
        &program_dir,
        PROVER_INPUT_FILE,
        Format::Toml,
        compiled_program.abi.as_ref().unwrap().clone(),
    )?;

    execute_program(&compiled_program, &inputs_map)
}

pub(crate) fn execute_program(
    compiled_program: &CompiledProgram,
    inputs_map: &InputMap,
) -> Result<(Option<InputValue>, WitnessMap), CliError> {
    // Solve the remaining witnesses
    let solved_witness = solve_witness(compiled_program, inputs_map)?;

    let public_inputs = extract_public_inputs(compiled_program, &solved_witness)?;
    let return_value = public_inputs.get(MAIN_RETURN_NAME).cloned();

    Ok((return_value, solved_witness))
}

pub(crate) fn solve_witness(
    compiled_program: &CompiledProgram,
    input_map: &InputMap,
) -> Result<WitnessMap, CliError> {
    let mut solved_witness = input_map_to_witness_map(input_map, &compiled_program.param_witnesses)
        .map_err(|error| match error {
            AbiError::UndefinedInput(_) => {
                CliError::Generic(format!("{error} in the {PROVER_INPUT_FILE}.toml file."))
            }
            _ => CliError::from(error),
        })?;

    let backend = crate::backends::ConcreteBackend;
    backend.solve(&mut solved_witness, compiled_program.circuit.opcodes.clone())?;

    Ok(solved_witness)
}

/// Given an InputMap and an Abi, produce a WitnessMap
///
/// In particular, this method shows one how to associate values in a Toml/JSON
/// file with witness indices
fn input_map_to_witness_map(
    input_map: &InputMap,
    abi_witness_map: &BTreeMap<String, Vec<Witness>>,
) -> Result<WitnessMap, AbiError> {
    // First encode each input separately
    let encoded_input_map: BTreeMap<String, Vec<FieldElement>> =
        try_btree_map(input_map, |(key, value)| {
            encode_value(value.clone(), key).map(|v| (key.clone(), v))
        })?;

    // Write input field elements into witness indices specified in `abi_witness_map`.
    let witness_map = encoded_input_map
        .iter()
        .flat_map(|(param_name, encoded_param_fields)| {
            let param_witness_indices = &abi_witness_map[param_name];
            param_witness_indices
                .iter()
                .zip(encoded_param_fields.iter())
                .map(|(&witness, &field_element)| (witness, field_element))
        })
        .collect();

    Ok(witness_map)
}

pub(crate) fn extract_public_inputs(
    compiled_program: &CompiledProgram,
    solved_witness: &WitnessMap,
) -> Result<InputMap, AbiError> {
    let public_abi = compiled_program.abi.as_ref().unwrap().clone().public_abi();

    let public_inputs_map = public_abi
        .parameters
        .iter()
        .map(|AbiParameter { name, typ, .. }| {
            let param_witness_values =
                vecmap(compiled_program.param_witnesses[name].clone(), |witness_index| {
                    solved_witness[&witness_index]
                });

            decode_value(&mut param_witness_values.into_iter(), typ)
                .map(|input_value| (name.clone(), input_value))
                .unwrap()
        })
        .collect();

    Ok(public_inputs_map)
}

pub(crate) fn save_witness_to_dir<P: AsRef<Path>>(
    witness: WitnessMap,
    witness_name: &str,
    witness_dir: P,
) -> Result<PathBuf, CliError> {
    let mut witness_path = create_named_dir(witness_dir.as_ref(), "witness");
    witness_path.push(witness_name);
    witness_path.set_extension(WITNESS_EXT);

    let buf = Witness::to_bytes(&witness);

    write_to_file(buf.as_slice(), &witness_path);

    Ok(witness_path)
}
