mod demo;
mod dynamic_load;

use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::{
    bytes::Bytes,
    core::{
        cell::{CellMetaBuilder, ResolvedTransaction},
        Capacity, DepType, EpochExt, HeaderView, ScriptHashType, TransactionView,
    },
    packed::{Byte32, CellDep, CellInput, CellOutput, OutPoint, Script, WitnessArgsBuilder},
    prelude::*,
};
use lazy_static::lazy_static;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

pub const MAX_CYCLES: u64 = std::u64::MAX;

// errors
pub const ERROR_OUTPUT_NUMBER: i8 = -99;

lazy_static! {
    pub static ref ALWAYS_SUCCESS: Bytes =
        Bytes::from(&include_bytes!("../../build/always_success")[..]);
    pub static ref DEMO: Bytes = Bytes::from(&include_bytes!("../../build/demo")[..]);
    pub static ref DYNAMIC_LOAD: Bytes =
        Bytes::from(&include_bytes!("../../build/dynamic_load")[..]);
}

#[derive(Default)]
pub struct DummyDataLoader {
    pub cells: HashMap<OutPoint, (CellOutput, Bytes)>,
    pub headers: HashMap<Byte32, HeaderView>,
    pub epoches: HashMap<Byte32, EpochExt>,
}

impl DummyDataLoader {
    fn new() -> Self {
        Self::default()
    }
}

impl CellDataProvider for DummyDataLoader {
    fn get_cell_data(&self, out_point: &OutPoint) -> Option<Bytes> {
        self.cells.get(out_point).map(|(_, data)| data.clone())
    }

    fn get_cell_data_hash(&self, out_point: &OutPoint) -> Option<Byte32> {
        self.cells
            .get(out_point)
            .map(|(_, data)| CellOutput::calc_data_hash(&data))
    }
}

impl HeaderProvider for DummyDataLoader {
    fn get_header(&self, hash: &Byte32) -> Option<HeaderView> {
        self.headers.get(hash).cloned()
    }
}

fn build_demo_script() -> Script {
    let data_hash = CellOutput::calc_data_hash(&DEMO);
    Script::new_builder()
        .code_hash(data_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .build()
}

fn build_dynamic_load_script() -> Script {
    let args = CellOutput::calc_data_hash(&DEMO).as_bytes();
    let data_hash = CellOutput::calc_data_hash(&DYNAMIC_LOAD);
    Script::new_builder()
        .args(args.pack())
        .code_hash(data_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .build()
}

fn build_always_success_script() -> Script {
    let data_hash = CellOutput::calc_data_hash(&ALWAYS_SUCCESS);
    Script::new_builder()
        .code_hash(data_hash.clone())
        .hash_type(ScriptHashType::Data.into())
        .build()
}

pub fn build_resolved_tx(
    data_loader: &DummyDataLoader,
    tx: &TransactionView,
) -> ResolvedTransaction {
    let resolved_cell_deps = tx
        .cell_deps()
        .into_iter()
        .map(|dep| {
            let deps_out_point = dep.clone();
            let (dep_output, dep_data) =
                data_loader.cells.get(&deps_out_point.out_point()).unwrap();
            CellMetaBuilder::from_cell_output(dep_output.to_owned(), dep_data.to_owned())
                .out_point(deps_out_point.out_point().clone())
                .build()
        })
        .collect();

    let mut resolved_inputs = Vec::new();
    for i in 0..tx.inputs().len() {
        let previous_out_point = tx.inputs().get(i).unwrap().previous_output();
        let (input_output, input_data) = data_loader.cells.get(&previous_out_point).unwrap();
        resolved_inputs.push(
            CellMetaBuilder::from_cell_output(input_output.to_owned(), input_data.to_owned())
                .out_point(previous_out_point)
                .build(),
        );
    }

    ResolvedTransaction {
        transaction: tx.clone(),
        resolved_cell_deps,
        resolved_inputs,
        resolved_dep_groups: vec![],
    }
}

trait SetField {
    fn set_cell_dep(self: &Self, dummy: &mut DummyDataLoader, code_bytes: &Bytes) -> Self;
    fn set_input(
        self: &Self,
        dummy: &mut DummyDataLoader,
        capacity: Capacity,
        lock_script: Script,
        type_script: Script,
    ) -> Self;
    fn set_output(
        self: &Self,
        capacity: Capacity,
        lock_script: Script,
        type_script: Script,
    ) -> Self;
}

impl SetField for TransactionView {
    fn set_cell_dep(self: &Self, dummy: &mut DummyDataLoader, code_bytes: &Bytes) -> Self {
        let mut rng = thread_rng();
        let out_point = {
            let contract_tx_hash = {
                let mut buf = [0u8; 32];
                rng.fill(&mut buf);
                buf.pack()
            };
            OutPoint::new(contract_tx_hash.clone(), 0)
        };
        let cell = CellOutput::new_builder()
            .capacity(
                Capacity::bytes(code_bytes.len())
                    .expect("script capacity")
                    .pack(),
            )
            .build();

        dummy
            .cells
            .insert(out_point.clone(), (cell, code_bytes.clone()));

        let tx_builder = self.as_advanced_builder();
        tx_builder
            .cell_dep(
                CellDep::new_builder()
                    .out_point(out_point)
                    .dep_type(DepType::Code.into())
                    .build(),
            )
            .build()
    }

    fn set_input(
        self: &Self,
        dummy: &mut DummyDataLoader,
        capacity: Capacity,
        lock_script: Script,
        type_script: Script,
    ) -> Self {
        let mut rng = thread_rng();
        let previous_tx_hash = {
            let mut buf = [0u8; 32];
            rng.fill(&mut buf);
            buf.pack()
        };
        let previous_out_point = OutPoint::new(previous_tx_hash, 0);
        let previous_output_cell = CellOutput::new_builder()
            .capacity(capacity.pack())
            .type_(Some(type_script).pack())
            .lock(lock_script)
            .build();

        dummy.cells.insert(
            previous_out_point.clone(),
            (previous_output_cell.clone(), Bytes::new()),
        );
        let witness_args = WitnessArgsBuilder::default().build();

        let tx_builder = self.as_advanced_builder();
        tx_builder
            .input(CellInput::new(previous_out_point, 0))
            .witness(witness_args.as_bytes().pack())
            .build()
    }

    fn set_output(
        self: &Self,
        capacity: Capacity,
        lock_script: Script,
        type_script: Script,
    ) -> Self {
        let tx_builder = self.as_advanced_builder();
        tx_builder
            .output(
                CellOutput::new_builder()
                    .capacity(capacity.pack())
                    .type_(Some(type_script).pack())
                    .lock(lock_script)
                    .build(),
            )
            .output_data(Bytes::new().pack())
            .build()
    }
}
