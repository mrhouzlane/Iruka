use pbc_zk::*;

/// Perform a zk computation on secret-shared data sum the secret variables.
///
/// ### Returns:
///
/// The sum of the secret variables.
pub fn sum_everything() -> Sbi32 {
    // Initialize state
    let mut sum: Sbi32 = sbi32_from(0);

    // Sum each variable
    for variable_id in 1..(num_secret_variables() + 1) {
        sum = sum + sbi32_input(variable_id);
    }

    sum
}
