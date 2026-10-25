/// FRI (Fast Reed-Solomon Interactive Oracle Proof) — low-degree testing.

use crate::field::Fp;
use crate::merkle::{MerkleTree, MerkleProof};
use crate::channel::Channel;

/// Number of FRI queries (each provides ~3 bits of security with blowup 8).
pub const NUM_FRI_QUERIES: usize = 30;

/// A single FRI layer's commitment and metadata.
#[derive(Debug, Clone)]
pub struct FriLayer {
    /// Merkle root of this layer's evaluations.
    pub root: [u8; 32],
    /// The evaluations themselves (prover keeps these, verifier doesn't see them).
    pub evaluations: Vec<Fp>,
    /// The Merkle tree (prover keeps for generating proofs).
    pub tree: MerkleTree,
}

/// The complete FRI proof.
#[derive(Debug, Clone)]
pub struct FriProof {
    /// Merkle root of each committed folding layer.
    pub layer_roots: Vec<[u8; 32]>,
    /// The final layer, sent in the clear. Every value must be the same (a constant).
    pub final_layer: Vec<Fp>,
    /// Query proofs: for each query, the opened values and Merkle proofs at each layer.
    pub query_proofs: Vec<FriQueryProof>,
}

/// Proof data for a single FRI query across all layers.
#[derive(Debug, Clone)]
pub struct FriQueryProof {
    /// For each layer: (value_at_index, value_at_sibling, merkle_proof_index, merkle_proof_sibling)
    pub layers: Vec<FriQueryLayer>,
}

/// Query data for one layer.
#[derive(Debug, Clone)]
pub struct FriQueryLayer {
    pub value: Fp,
    pub sibling_value: Fp,
    pub proof: MerkleProof,
    pub sibling_proof: MerkleProof,
}

///   folded = f_even + beta * f_odd
fn fold_pair(f_pos: Fp, f_neg: Fp, d: Fp, beta: Fp) -> Fp {
    let two = Fp::new(2);
    let f_even = (f_pos + f_neg) * two.inv();
    let f_odd = (f_pos - f_neg) * (two * d).inv();
    f_even + beta * f_odd
}

pub fn fri_num_layers(domain_size: usize, final_size: usize) -> usize {
    assert!(domain_size.is_power_of_two() && final_size.is_power_of_two());
    assert!(domain_size > final_size);
    (domain_size / final_size).trailing_zeros() as usize
}

pub fn fri_commit(
    evaluations: &[Fp],
    domain_gen: Fp,
    domain_offset: Fp,
    final_size: usize,
    channel: &mut Channel,
) -> (Vec<FriLayer>, Vec<Fp>) {
    // Validates sizes (power of two, domain strictly larger than the final layer).
    let _ = fri_num_layers(evaluations.len(), final_size);

    let mut layers: Vec<FriLayer> = Vec::new();
    let mut current_evals = evaluations.to_vec();
    let mut current_gen = domain_gen;
    let mut current_offset = domain_offset;

    while current_evals.len() > final_size {
        // Commit to current layer
        let leaf_data: Vec<Vec<u8>> = current_evals
            .iter()
            .map(|v| v.value().to_le_bytes().to_vec())
            .collect();
        let tree = MerkleTree::new(&leaf_data);
        let root = tree.root();

        channel.absorb(&root);

        layers.push(FriLayer {
            root,
            evaluations: current_evals.clone(),
            tree,
        });

        // Derive folding challenge
        let beta = channel.squeeze_field();

        // Fold: pair evaluation i with evaluation i + half (the point -d_i on a coset).
        let half = current_evals.len() / 2;
        let mut folded = Vec::with_capacity(half);
        for i in 0..half {
            let d_i = current_offset * current_gen.pow(i as u64);
            folded.push(fold_pair(current_evals[i], current_evals[i + half], d_i, beta));
        }

        // New domain: generator squared, offset squared, half the size
        current_gen = current_gen * current_gen;
        current_offset = current_offset * current_offset;
        current_evals = folded;
    }

    for v in &current_evals {
        channel.absorb_field(*v);
    }

    (layers, current_evals)
}

/// FRI query phase: open values at queried indices across all layers.
pub fn fri_query(
    layers: &[FriLayer],
    query_indices: &[usize],
) -> Vec<FriQueryProof> {
    query_indices
        .iter()
        .map(|&initial_idx| {
            let mut layer_proofs = Vec::new();
            let mut idx = initial_idx;

            for layer in layers {
                let half = layer.evaluations.len() / 2;
                // Normalize index to be in the first half
                let lo = idx % half;
                let hi = lo + half;

                layer_proofs.push(FriQueryLayer {
                    value: layer.evaluations[lo],
                    sibling_value: layer.evaluations[hi],
                    proof: layer.tree.prove(lo),
                    sibling_proof: layer.tree.prove(hi),
                });

                // Index for next layer is lo (halved domain)
                idx = lo;
            }

            FriQueryProof { layers: layer_proofs }
        })
        .collect()
}

pub fn fri_derive_betas(
    proof: &FriProof,
    channel: &mut Channel,
) -> Vec<Fp> {
    let mut betas = Vec::new();
    for root in &proof.layer_roots {
        channel.absorb(root);
        betas.push(channel.squeeze_field());
    }
    for v in &proof.final_layer {
        channel.absorb_field(*v);
    }
    betas
}

pub fn fri_verify_queries(
    proof: &FriProof,
    betas: &[Fp],
    initial_domain_gen: Fp,
    initial_domain_offset: Fp,
    initial_domain_size: usize,
    final_size: usize,
    query_indices: &[usize],
) -> Result<(), String> {
    if query_indices.is_empty() {
        return Err("no query indices — proof has no security".to_string());
    }
    let num_layers = fri_num_layers(initial_domain_size, final_size);
    if proof.layer_roots.len() != num_layers || betas.len() != num_layers {
        return Err(format!(
            "expected {} FRI layers, got {}", num_layers, proof.layer_roots.len()
        ));
    }
    if proof.final_layer.len() != final_size {
        return Err(format!(
            "expected final layer of {} values, got {}", final_size, proof.final_layer.len()
        ));
    }
    // Degree-bound check: a polynomial of degree < 1 is constant.
    let constant = proof.final_layer[0];
    if proof.final_layer.iter().any(|v| *v != constant) {
        return Err("FRI final layer is not constant: quotient is not low-degree".to_string());
    }
    if proof.query_proofs.len() != query_indices.len() {
        return Err(format!(
            "expected {} FRI query proofs, got {}",
            query_indices.len(), proof.query_proofs.len()
        ));
    }

    for (q_idx, query_proof) in proof.query_proofs.iter().enumerate() {
        if query_proof.layers.len() != num_layers {
            return Err(format!(
                "query {} has {} layers, expected {}",
                q_idx, query_proof.layers.len(), num_layers
            ));
        }

        let mut idx = query_indices[q_idx];
        let mut gen = initial_domain_gen;
        let mut offset = initial_domain_offset;
        let mut size = initial_domain_size;

        for (layer_idx, ql) in query_proof.layers.iter().enumerate() {
            let half = size / 2;
            let lo = idx % half;
            let hi = lo + half;

            // Verify Merkle proofs
            let lo_data = ql.value.value().to_le_bytes().to_vec();
            let hi_data = ql.sibling_value.value().to_le_bytes().to_vec();

            if !MerkleTree::verify(&proof.layer_roots[layer_idx], lo, &lo_data, &ql.proof, size) {
                return Err(format!(
                    "FRI Merkle proof failed: query {}, layer {}, index {}",
                    q_idx, layer_idx, lo
                ));
            }
            if !MerkleTree::verify(&proof.layer_roots[layer_idx], hi, &hi_data, &ql.sibling_proof, size) {
                return Err(format!(
                    "FRI Merkle proof failed: query {}, layer {}, sibling index {}",
                    q_idx, layer_idx, hi
                ));
            }

            // Fold this layer's pair; the result is the value at position `lo` of the next layer.
            let d_i = offset * gen.pow(lo as u64);
            let expected = fold_pair(ql.value, ql.sibling_value, d_i, betas[layer_idx]);

            let actual = if layer_idx + 1 < num_layers {
                // The next layer (size `half`) opens positions next_lo and next_lo + half/2.
                let next_lo = lo % (half / 2);
                let next = &query_proof.layers[layer_idx + 1];
                if lo == next_lo { next.value } else { next.sibling_value }
            } else {
                // Last committed layer folds into the final layer (size `half` == final_size).
                proof.final_layer[lo]
            };

            if expected != actual {
                return Err(format!(
                    "FRI folding mismatch: query {}, layer {}, expected {:?}, got {:?}",
                    q_idx, layer_idx, expected, actual
                ));
            }

            // Move to next layer's domain
            gen = gen * gen;
            offset = offset * offset;
            size = half;
            idx = lo;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::polynomial::Polynomial;
    use crate::domain::Domain;

    const TRACE_SIZE: usize = 8;
    const BLOWUP: usize = 8;

    /// Run FRI on `evals` claiming degree < `degree_bound`.
    /// `tamper` can modify the proof before the verifier sees it.
    fn run_fri(
        evals: &[Fp],
        degree_bound: usize,
        tamper: impl FnOnce(&mut FriProof),
    ) -> Result<(), String> {
        let dom = Domain::lde_domain(TRACE_SIZE, BLOWUP);
        let final_size = dom.size / degree_bound;

        let mut prover_channel = Channel::new();
        prover_channel.absorb(b"test_fri");
        let (layers, final_layer) =
            fri_commit(evals, dom.generator, dom.offset, final_size, &mut prover_channel);

        let max_queries = (dom.size / 4).min(NUM_FRI_QUERIES);
        let query_indices = prover_channel.squeeze_indices(max_queries, dom.size / 2);
        let query_proofs = fri_query(&layers, &query_indices);

        let mut proof = FriProof {
            layer_roots: layers.iter().map(|l| l.root).collect(),
            final_layer,
            query_proofs,
        };
        tamper(&mut proof);

        // Verifier replays the transcript.
        let mut verifier_channel = Channel::new();
        verifier_channel.absorb(b"test_fri");
        let betas = fri_derive_betas(&proof, &mut verifier_channel);
        let verifier_indices = verifier_channel.squeeze_indices(max_queries, dom.size / 2);

        fri_verify_queries(
            &proof, &betas, dom.generator, dom.offset, dom.size, final_size, &verifier_indices,
        )
    }

    fn evals_of_degree(degree: usize) -> Vec<Fp> {
        let dom = Domain::lde_domain(TRACE_SIZE, BLOWUP);
        let poly = Polynomial {
            coeffs: (0..=degree as u64).map(|i| Fp::new(i * 7 + 1)).collect(),
        };
        dom.elements().iter().map(|x| poly.evaluate(*x)).collect()
    }

    #[test]
    fn test_fri_honest_polynomial() {
        // Degree 7 < 8.
        let result = run_fri(&evals_of_degree(7), 8, |_| {});
        assert!(result.is_ok(), "FRI verification failed: {:?}", result.err());
    }

    #[test]
    fn test_fri_rejects_random_values() {
        // This is the bug the old FRI had: it accepted ANY 64 values.
        let random: Vec<Fp> = (0..(TRACE_SIZE * BLOWUP) as u64)
            .map(|i| Fp::new(i.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xABC_DEF1_23))
            .collect();
        assert!(run_fri(&random, 8, |_| {}).is_err());
    }

    #[test]
    fn test_fri_rejects_polynomial_above_degree_bound() {
        // Degree 40 is a perfectly good polynomial, but it exceeds the claimed bound of 8.
        assert!(run_fri(&evals_of_degree(40), 8, |_| {}).is_err());
    }

    #[test]
    fn test_fri_rejects_forged_constant_final_layer() {
        // A cheating prover replaces the final layer by a constant. The fold checks at the
        // queried positions no longer match.
        let result = run_fri(&evals_of_degree(40), 8, |proof| {
            let c = proof.final_layer[0];
            for v in proof.final_layer.iter_mut() {
                *v = c;
            }
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_fri_rejects_wrong_layer_count() {
        // A proof with a layer removed must not be accepted.
        let result = run_fri(&evals_of_degree(7), 8, |proof| {
            proof.layer_roots.pop();
        });
        assert!(result.is_err());
    }
}
