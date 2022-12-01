use create_type_spec_derive::CreateTypeSpec;
use read_write_rpc_derive::ReadWriteRPC;
use read_write_state_derive::ReadWriteState;

/// Represents fraction between 0 and 1 (both inclusive).
#[derive(ReadWriteRPC, ReadWriteState, CreateTypeSpec, Clone, Debug)]
pub struct Fraction {
    /// Numerator of ratio
    numerator: u32,
    /// Denominator of ratio
    denominator: u32,
}

impl Fraction {
    /**
     * Ensures that the `Fraction` instance is valid.
     */
    pub fn assert_valid(&self) {
        assert!(0 < self.denominator);
        assert!(self.numerator <= self.denominator);
    }

    /**
     * Constructor for `Fraction`. Throws if inputs are invalid.
     */
    pub fn new(numerator: u32, denominator: u32) -> Self {
        let value = unsafe { Self::new_unchecked(numerator, denominator) };
        value.assert_valid();
        value
    }

    /**
     * Constructor for `Fraction`s, without checking for validity.
     *
     * # Safety
     *
     * Caller should manually ensure that [`Self::assert_valid`] is called.
     */
    pub const unsafe fn new_unchecked(numerator: u32, denominator: u32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl std::cmp::PartialEq for Fraction {
    fn eq(&self, other: &Self) -> bool {
        (self.numerator * other.denominator) == (other.numerator * self.denominator)
    }
}

impl std::cmp::PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.numerator * other.denominator).partial_cmp(&(other.numerator * self.denominator))
    }
}

#[cfg(test)]
mod test {

    use super::Fraction;

    #[test]
    fn valid() {
        let _ = Fraction::new(0, 1);
        let _ = Fraction::new(1, 1);
        let _ = Fraction::new(5, 51);
    }

    #[test]
    #[should_panic]
    fn invalid_1() {
        Fraction::new(0, 0);
    }
    #[test]
    #[should_panic]
    fn invalid_2() {
        Fraction::new(1, 0);
    }
    #[test]
    #[should_panic]
    fn invalid_3() {
        Fraction::new(3123, 231);
    }

    #[test]
    fn eq() {
        assert_eq!(Fraction::new(0, 1), Fraction::new(0, 8));
        assert_eq!(Fraction::new(1, 1), Fraction::new(8, 8));
        assert_eq!(Fraction::new(1, 2), Fraction::new(4, 8));
        assert_eq!(Fraction::new(1, 7), Fraction::new(3, 21));
    }

    #[test]
    fn lt() {
        assert!(Fraction::new(0, 1) < Fraction::new(1, 8));
        assert!(Fraction::new(0, 1) < Fraction::new(1, 1));
        assert!(Fraction::new(0, 1001) < Fraction::new(1, 1));
        assert!(Fraction::new(500, 1001) < Fraction::new(1, 2));
    }
}
