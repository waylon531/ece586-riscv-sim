pub enum Register {
    Zero,
    RA,
    SP,
    GP,
    TP,
    T0,T1,T2,T3,T4,T5,T6,
    // These are two names for the same reg
    S0,FP,
    S1,S2,S3,S4,S5,S6,S7,S8,S9,S10,S11,
    A0,A1,A2,A3,A4,A5,A6,A7,
}
impl Register {
    pub fn to_num(&self) -> usize {
        use Register::*;
        match *self {
            Zero => 0,
            RA => 1,
            SP => 2,
            GP => 3,
            TP => 4,
            T0 => 5,
            T1 => 6,
            T2 => 7,
            S0 => 8,
            FP => 8,
            S1 => 9,
            A0 => 10,
            A1 => 11,
            A2 => 12,
            A3 => 13,
            A4 => 14,
            A5 => 15,
            A6 => 16,
            A7 => 17,
            S2 => 18,
            S3 => 19,
            S4 => 20,
            S5 => 21,
            S6 => 22,
            S7 => 23,
            S8 => 24,
            S9 => 25,
            S10 => 26,
            S11 => 27,
            T3 => 28,
            T4 => 29,
            T5 => 30,
            T6 => 31,
        }
    }
    pub fn from_num(num: u32) -> Option<Register> {
        use Register::*;
        match num {
            0 => Some(Zero),
            1 => Some(RA),
            2 => Some(SP),
            3 => Some(GP),
            4 => Some(TP),
            5 => Some(T0),
            6 => Some(T1),
            7 => Some(T2),
            8 => Some(S0),
            9 => Some(S1),
            10 => Some(A0),
            11 => Some(A1),
            12 => Some(A2),
            13 => Some(A3),
            14 => Some(A4),
            15 => Some(A5),
            16 => Some(A6),
            17 => Some(A7),
            18 => Some(S2),
            19 => Some(S3),
            20 => Some(S4),
            21 => Some(S5),
            22 => Some(S6),
            23 => Some(S7),
            24 => Some(S8),
            25 => Some(S9),
            26 => Some(S10),
            27 => Some(S11),
            28 => Some(T3),
            29 => Some(T4),
            30 => Some(T5),
            31 => Some(T6),
            _ => None
        }
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_reg_num_conversion() {
        for i in 0..32 {
            assert_eq!(i, Register::from_num(i as u32).unwrap().to_num());
        }
    }
}
