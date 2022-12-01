#[cfg(test)]
mod utility_tests {
    use crate::u64_division_ceil;

    #[test]
    pub fn test_u64_division_ceil() {
        // Division by 0 cases is guarded against by u64 type and the source code

        let div1 = u64_division_ceil(10, 2);
        let div2 = u64_division_ceil(999, 66);
        let div3 = u64_division_ceil(15, 4);

        assert_eq!(div1, 5);
        assert_eq!(div2, 16);
        assert_eq!(div3, 4);
    }
}
