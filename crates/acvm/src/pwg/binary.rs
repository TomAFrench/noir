use std::collections::{BTreeMap, HashMap, HashSet};

use acir::{
    circuit::{gate::Directive, Gate},
    native_types::{Expression, Witness},
    FieldElement,
};
use num_bigint::BigUint;
use num_traits::{One, Zero};

use crate::{GateResolution, GateResolutionError};

// This structure tracks un-resolved witnesses which are constrained to be booleans or sum of booleans, so that gates which uses only those can be solved.
// This is the case for array equality constraints which are unsolved when the value of arrays returned from main are not supplied in the .toml
pub struct BinarySolver {
    binary_witness: HashSet<Witness>,
    invert_witness: HashMap<Witness, Witness>,
    positive_witness: HashMap<Witness, BigUint>,
}

impl Default for BinarySolver {
    fn default() -> Self {
        Self::new()
    }
}

impl BinarySolver {
    pub fn new() -> BinarySolver {
        BinarySolver {
            binary_witness: HashSet::new(),
            invert_witness: HashMap::new(),
            positive_witness: HashMap::new(),
        }
    }

    pub fn is_boolean(&self, w: &Witness) -> bool {
        self.binary_witness.contains(w)
    }

    pub fn are_inverse(&self, w1: &Witness, w2: &Witness) -> bool {
        self.invert_witness.get(w1) == Some(w2) || self.invert_witness.get(w2) == Some(w1)
    }

    pub fn are_boolean(&self, w1: &Witness, w2: &Witness) -> bool {
        self.are_inverse(w1, w2) || (self.is_boolean(w1) && self.is_boolean(w2))
    }

    pub fn get_max_value(&self, w: &Witness) -> Option<BigUint> {
        if self.is_boolean(w) {
            Some(BigUint::one())
        } else {
            self.positive_witness.get(w).cloned()
        }
    }

    pub fn solve(
        &mut self,
        gate: &Gate,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
    ) -> Result<GateResolution, GateResolutionError> {
        if let Gate::Arithmetic(arith) = gate {
            let partial_gate =
                super::arithmetic::ArithmeticSolver::evaluate(arith, initial_witness);
            let result = self.solve_booleans(initial_witness, &partial_gate);
            self.identify_booleans(&partial_gate);
            result
        } else {
            self.identify_binaries(gate);
            Ok(GateResolution::Skip)
        }
    }

    /// Solve (some) arithemtic expression which is only using booleans
    pub fn solve_booleans(
        &self,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &Expression,
    ) -> Result<GateResolution, GateResolutionError> {
        if let GateResolution::Resolved = self.solve_inverse(gate, initial_witness)? {
            return Ok(GateResolution::Resolved);
        }

        if let Some(max) = self.is_binary(gate) {
            if max < FieldElement::modulus() {
                if gate.q_c == FieldElement::zero() {
                    for (_, w) in &gate.linear_combinations {
                        initial_witness.insert(*w, FieldElement::zero());
                    }
                    Ok(GateResolution::Resolved)
                } else {
                    Err(GateResolutionError::UnsatisfiedConstrain)
                }
            } else {
                Ok(GateResolution::Skip)
            }
        } else {
            Ok(GateResolution::Skip)
        }
    }

    // checks whether the expression uses only booleans/positive witness and returns the max value of the expression in that case
    fn is_binary(&self, gate: &Expression) -> Option<BigUint> {
        let mut max = BigUint::zero();
        for (c, w1, w2) in &gate.mul_terms {
            if !self.are_boolean(w1, w2) {
                return None;
            }
            max += BigUint::from_bytes_be(&c.to_bytes());
        }
        for (c, w) in &gate.linear_combinations {
            if let Some(v) = self.get_max_value(w) {
                max += BigUint::from_bytes_be(&c.to_bytes()) * v;
            } else {
                return None;
            }
        }
        if max > FieldElement::modulus() {
            return None;
        }
        Some(max + BigUint::from_bytes_be(&gate.q_c.to_bytes()))
    }

    fn solve_inverse(
        &self,
        gate: &Expression,
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
    ) -> Result<GateResolution, GateResolutionError> {
        if gate.mul_terms.len() == 1
            && self.are_inverse(&gate.mul_terms[0].1, &gate.mul_terms[0].2)
            && gate.linear_combinations.is_empty()
        {
            if gate.q_c.is_zero() {
                initial_witness.insert(gate.mul_terms[0].1, FieldElement::zero());
                initial_witness.insert(gate.mul_terms[0].2, FieldElement::zero());
                return Ok(GateResolution::Resolved);
            } else if !gate.q_c.is_one() {
                return Err(GateResolutionError::UnsatisfiedConstrain);
            }
        }

        Ok(GateResolution::Skip)
    }

    // look for boolean constraint and add boolean witness to a map
    pub fn identify_booleans(&mut self, arith: &Expression) {
        let mut x = None;
        if arith.mul_terms.len() == 1 && arith.linear_combinations.len() == 1 {
            // x*x = x
            if arith.mul_terms[0].1 == arith.mul_terms[0].2
                && arith.mul_terms[0].1 == arith.linear_combinations[0].1
                && arith.q_c.is_zero()
                && (arith.mul_terms[0].0 + arith.linear_combinations[0].0).is_zero()
            {
                x = Some(0);
            } else {
                // x = a*b, a,b booleans or inverse
                if self.are_boolean(&arith.mul_terms[0].1, &arith.mul_terms[0].2) {
                    if arith.q_c.is_zero() {
                        if (arith.mul_terms[0].0 + arith.linear_combinations[0].0).is_zero() {
                            x = Some(0);
                        }
                    } else if (arith.mul_terms[0].0 + arith.q_c).is_zero()
                        && arith.mul_terms[0].0 == arith.linear_combinations[0].0
                    {
                        x = Some(0);
                    }
                }
            }
        } else if arith.mul_terms.is_empty() && arith.linear_combinations.len() == 2 {
            //x=y
            let z = if self.is_boolean(&arith.linear_combinations[0].1)
                && !self.is_boolean(&arith.linear_combinations[1].1)
            {
                Some(1)
            } else if self.is_boolean(&arith.linear_combinations[1].1)
                && !self.is_boolean(&arith.linear_combinations[0].1)
            {
                Some(0)
            } else {
                None
            };
            if z.is_some() {
                if arith.q_c.is_zero() {
                    if (arith.linear_combinations[0].0 + arith.linear_combinations[1].0).is_zero() {
                        x = z;
                    }
                } else if (arith.q_c + arith.linear_combinations[1].0).is_zero()
                    && arith.linear_combinations[0].0 == arith.linear_combinations[1].0
                {
                    x = Some(0);
                }
            }
        } else if arith.mul_terms.is_empty() && arith.linear_combinations.len() > 2 {
            //"binary" gates 'optimised' by the optimiser should have an intermediate variable and a bunch of booleans
            let mut max = BigUint::from_bytes_be(&arith.q_c.to_bytes());
            for (i, lin) in arith.linear_combinations.iter().enumerate() {
                if let Some(v) = self.get_max_value(&lin.1) {
                    max += v * BigUint::from_bytes_be(&lin.0.to_bytes());
                } else if x.is_some() {
                    x = None;
                    break;
                } else {
                    x = Some(i);
                }
            }
            if max < FieldElement::modulus()
                && x.is_some()
                && arith.linear_combinations[x.unwrap()].0 == -FieldElement::one()
            {
                self.positive_witness.insert(arith.linear_combinations[x.unwrap()].1, max);
                x = None;
            }
        }
        if let Some(x) = x {
            self.binary_witness.insert(arith.linear_combinations[x].1);
        }
    }

    // identify boolean and inverse constraints in the gate
    pub fn identify_binaries(&mut self, gate: &Gate) {
        match gate {
            Gate::Directive(Directive::Invert { x, result }) => {
                self.invert_witness.insert(*x, *result);
            }
            Gate::Arithmetic(a) => {
                self.identify_booleans(a);
            }
            _ => (),
        }
    }
}
