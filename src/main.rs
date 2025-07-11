use ark_bls12_377::{Bls12_377, Fr};
use ark_groth16::Groth16;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystem};
use ark_serialize::CanonicalSerialize;
use ark_snark::{CircuitSpecificSetupSNARK, SNARK};
use rand::thread_rng;
use std::time::Instant;

mod alloc;
mod cmp;
mod constraint;

#[derive(Clone)]
struct Circuit<const N: usize> {
    solution: [[u8; N]; N],
    puzzle: [[u8; N]; N],
}

fn serialize_input<const N: usize>(mat: [[u8; N]; N]) -> Vec<Fr> {
    let mut enc_input = Vec::new();
    for row in mat.iter() {
        for &value in row.iter() {
            for i in 0..8 {
                if (value >> i) & 1 == 1 {
                    enc_input.push(Fr::from(1u8));
                } else {
                    enc_input.push(Fr::from(0u8));
                }
            }
        }
    }
    enc_input
}

fn main() {
    let puzzle = [(); 9].map(|_| [(); 9].map(|_| 0u8));
    let solution = [(); 9].map(|_| [(); 9].map(|_| 0u8));

    let circuit_defining_cs = Circuit { puzzle, solution };
    println!("------------------------------------------------------------------------");
    println!("This test cheks if someone has a valid solution for a 9x9 sudoku puzzle");
    println!("------------------------------------------------------------------------");

    let rng = &mut thread_rng();

    println!("\n===Check circuit defining constraint system without proving===");
    let cs = ConstraintSystem::<Fr>::new_ref();
    // The function consumes the circuit and constraint system, which is why we clone them.
    circuit_defining_cs
        .clone()
        .generate_constraints(cs.clone())
        .unwrap();
    println!("Is satisfied: {}", cs.is_satisfied().unwrap());
    println!("Num constraints: {}", cs.num_constraints());

    // Setup Phase
    println!("\n===Entering Groth16 Setup Phase===");

    let start = Instant::now();
    let (pk, vk) = Groth16::<Bls12_377>::setup(circuit_defining_cs.clone(), rng).unwrap();
    let end = start.elapsed();

    println!("Setup time: {:?}", end);

    let puzzle = [
        [5, 3, 0, 0, 7, 0, 0, 0, 0],
        [6, 0, 0, 1, 9, 5, 0, 0, 0],
        [0, 9, 8, 0, 0, 0, 0, 6, 0],
        [8, 0, 0, 0, 6, 0, 0, 0, 3],
        [4, 0, 0, 8, 0, 3, 0, 0, 1],
        [7, 0, 0, 0, 2, 0, 0, 0, 6],
        [0, 6, 0, 0, 0, 0, 2, 8, 0],
        [0, 0, 0, 4, 1, 9, 0, 0, 5],
        [0, 0, 0, 0, 8, 0, 0, 7, 9],
    ];

    let solution = [
        [5, 3, 4, 6, 7, 8, 9, 1, 2],
        [6, 7, 2, 1, 9, 5, 3, 4, 8],
        [1, 9, 8, 3, 4, 2, 5, 6, 7],
        [8, 5, 9, 7, 6, 1, 4, 2, 3],
        [4, 2, 6, 8, 5, 3, 7, 9, 1],
        [7, 1, 3, 9, 2, 4, 8, 5, 6],
        [9, 6, 1, 5, 3, 7, 2, 8, 4],
        [2, 8, 7, 4, 1, 9, 6, 3, 5],
        [3, 4, 5, 2, 8, 6, 1, 7, 9],
    ];

    let circuit_to_verify = Circuit { puzzle, solution };

    // Proving
    println!("\n===Entering Proving Phase===");

    let mut start = Instant::now();
    let proof = Groth16::<Bls12_377>::prove(&pk, circuit_to_verify, rng).unwrap();

    println!("proof: {:#?}", proof);

    let end = start.elapsed();

    println!("Proving time: {:?}", end);

    println!("Proof size: {}", proof.compressed_size());

    // Verifying
    println!("\n===Verification Phase===");
    let enc_input = serialize_input(puzzle.clone());
    println!("Public Input Lenght {}", enc_input.len());
    start = Instant::now();
    let is_valid = Groth16::<Bls12_377>::verify(&vk, &enc_input, &proof).unwrap();
    let end = start.elapsed();

    println!("Verification time: {:?}", end);

    assert!(is_valid);

    println!("The solution is valid\n");
}
