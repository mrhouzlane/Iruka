use pbc_zk::*;

/// Perform a zk computation on secret-shared data to count the number
/// of accepting votes (non-zero).
///
/// ### Returns:
///
/// The number of accepting votes.
pub fn zk_compute() -> Sbi32 {
    // Initialize votes
    let mut votes_for: Sbi32 = sbi32_from(0);

    // Count votes
    for variable_id in 1..(num_secret_variables() + 1) {
        if sbi32_input(variable_id) != sbi32_from(0) {
            votes_for = votes_for + sbi32_from(1);
        }
    }
    votes_for
}
