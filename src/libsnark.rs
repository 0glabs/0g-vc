use ark_bn254::Bn254;
use ark_circom::CircomCircuit;

use libsnark_rust::types::{ConstraintSystem, Input, LinearCombination};

pub fn make_constraints(circuit: &CircomCircuit<Bn254>) -> ConstraintSystem {
    let mut cs = ConstraintSystem::new(circuit.r1cs.num_inputs - 1, circuit.r1cs.num_aux);

    for (a, b, c) in &circuit.r1cs.constraints {
        let la = LinearCombination::new(a);
        let lb = LinearCombination::new(b);
        let lc = LinearCombination::new(c);
        cs.add_constraint(&la, &lb, &lc);
    }
    cs
}

pub fn make_input(circuit: &CircomCircuit<Bn254>) -> Option<(Input, Input)> {
    let witness = circuit.witness.as_ref()?;
    let wire_mapping = &circuit.r1cs.wire_mapping;

    let inputs: Vec<_> = (1..circuit.r1cs.num_variables)
        .map(|i| {
            if let Some(m) = wire_mapping {
                witness[m[i]]
            } else {
                witness[i]
            }
        })
        .collect();

    let input_num = circuit.r1cs.num_inputs - 1;

    let primary_inputs = Input::from_fr(&inputs[..input_num]);
    let auxiliary_inputs = Input::from_fr(&inputs[input_num..]);

    Some((primary_inputs, auxiliary_inputs))
}
