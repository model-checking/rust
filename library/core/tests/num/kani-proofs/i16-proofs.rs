#![feature(unchecked_shifts)]
#[cfg(kani)]
mod verification {
    // use super::*;

    #[kani::proof]
    fn verify_i16_unchecked_add() {
        let num1: i16 = kani::any::<i16>();
        let num2: i16 = kani::any::<i16>();

        // Safety preconditions:
        // - Positive number addition won't overflow
        // - Negative number addition won't underflow
        // Addition of two integers with different signs never overflows
        // Undefined behavior occurs when overflow or underflow happens
        kani::assume(
            (num1 > 0 && num2 > 0 && num1 < i16::MAX - num2)
                || (num1 < 0 && num2 < 0 && num1 > i16::MIN - num2),
        );

        unsafe {
            let result = num1.unchecked_add(num2);
            assert_eq!(Some(result), num1.checked_add(num2));
        }
    }

    #[kani::proof]
    fn verify_i16_unchecked_sub() {
        // TODO
    }

    #[kani::proof]
    fn verify_i16_unchecked_mul() {
        // TODO
    }

    #[kani::proof]
    fn verify_i16_unchecked_shl() {
        let num: i16 = kani::any::<i16>(); // Any value in type i16
        let shift_amount: u32 = kani::any::<u32>(); // Any shift amount in type u32
    
        // Assume the shift value is smaller than 16 because i16 only has 16 bits
        kani::assume(shift_amount < 16);
    
        unsafe {
            let result = num.unchecked_shl(shift_amount);
            assert_eq!(Some(result), num.checked_shl(shift_amount));
        }
    }

    #[kani::proof]
    fn verify_i16_unchecked_shr() {
        // TODO
    }

    #[kani::proof]
    fn verify_i16_unchecked_neg() {
        // TODO
    }
}
