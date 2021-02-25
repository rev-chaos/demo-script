use super::{
    build_always_success_script, build_dynamic_load_script, build_resolved_tx, DummyDataLoader,
    SetField, ALWAYS_SUCCESS, DEMO, DYNAMIC_LOAD, ERROR_OUTPUT_NUMBER, MAX_CYCLES,
};
use ckb_error::assert_error_eq;
use ckb_script::{ScriptError, TransactionScriptsVerifier};
use ckb_types::core::{Capacity, TransactionBuilder};

#[test]
fn test_dynamic_load_only_one_output() {
    let type_script = build_dynamic_load_script();
    let lock_scipt = build_always_success_script();
    let capacity = Capacity::shannons(42);

    let mut data_loader = DummyDataLoader::new();
    let tx = TransactionBuilder::default()
        .build()
        .set_cell_dep(&mut data_loader, &ALWAYS_SUCCESS)
        .set_cell_dep(&mut data_loader, &DEMO)
        .set_cell_dep(&mut data_loader, &DYNAMIC_LOAD)
        .set_input(
            &mut data_loader,
            capacity.clone(),
            lock_scipt.clone(),
            type_script.clone(),
        )
        .set_output(capacity.clone(), lock_scipt.clone(), type_script.clone());

    let resolved_tx = build_resolved_tx(&data_loader, &tx);
    let mut verifier = TransactionScriptsVerifier::new(&resolved_tx, &data_loader);
    verifier.set_debug_printer(|_, msg| println!("debug: {}", msg));
    let verify_result = verifier.verify(MAX_CYCLES);
    verify_result.expect("pass");
}

#[test]
fn test_dynamic_load_two_outputs() {
    let type_script = build_dynamic_load_script();
    let lock_scipt = build_always_success_script();
    let capacity = Capacity::shannons(42);

    let mut data_loader = DummyDataLoader::new();
    let tx = TransactionBuilder::default()
        .build()
        .set_cell_dep(&mut data_loader, &ALWAYS_SUCCESS)
        .set_cell_dep(&mut data_loader, &DEMO)
        .set_cell_dep(&mut data_loader, &DYNAMIC_LOAD)
        .set_input(
            &mut data_loader,
            capacity.clone(),
            lock_scipt.clone(),
            type_script.clone(),
        )
        .set_output(capacity.clone(), lock_scipt.clone(), type_script.clone())
        .set_output(capacity.clone(), lock_scipt.clone(), type_script.clone());

    let resolved_tx = build_resolved_tx(&data_loader, &tx);
    let verifier = TransactionScriptsVerifier::new(&resolved_tx, &data_loader);
    let verify_result = verifier.verify(MAX_CYCLES);
    assert_error_eq!(
        verify_result.unwrap_err(),
        ScriptError::ValidationFailure(ERROR_OUTPUT_NUMBER).input_type_script(0),
    );
}
