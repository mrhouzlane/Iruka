/// Perform a zk computation on secret-shared data.
/// Finds the highest bidder and the amount of the second-highest bid
use pbc_zk::*;

pub fn zk_compute() -> (Sbi32, Sbi32) {
    // Initialize state
    let mut highest_bidder: Sbi32 = sbi32_from(sbi32_metadata(1));
    let mut highest_amount: Sbi32 = sbi32_from(0);
    let mut second_highest_amount: Sbi32 = sbi32_from(0);

    // Determine max
    for variable_id in 1..(num_secret_variables() + 1) {
        if sbi32_input(variable_id) > highest_amount {
            second_highest_amount = highest_amount;
            highest_amount = sbi32_input(variable_id);
            highest_bidder = sbi32_from(sbi32_metadata(variable_id));
        } else if sbi32_input(variable_id) > second_highest_amount {
            second_highest_amount = sbi32_input(variable_id);
        }
    }

    // Return highest bidder index, and second highest amount
    (highest_bidder, second_highest_amount)
}
