// Copyright (C) 2019-2021 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::{
    assert_satisfied,
    expect_asg_error,
    expect_compiler_error,
    generate_main_input,
    integers::{expect_computation_error, IntegerTester},
    parse_program,
};
use leo_ast::InputValue;
use leo_input::types::{I128Type, IntegerType, SignedIntegerType};

test_int!(
    TestI128,
    i128,
    IntegerType::Signed(SignedIntegerType::I128Type(I128Type {})),
    Int128
);

#[test]
fn test_i128_min() {
    TestI128::test_min();
}

#[test]
fn test_i128_min_fail() {
    TestI128::test_min_fail();
}

#[test]
fn test_i128_max() {
    TestI128::test_max();
}

#[test]
fn test_i128_max_fail() {
    TestI128::test_max_fail();
}

#[test]
fn test_i128_neg() {
    TestI128::test_negate();
}

#[test]
fn test_i128_neg_max_fail() {
    TestI128::test_negate_min_fail();
}

#[test]
fn test_i128_neg_zero() {
    TestI128::test_negate_zero();
}

#[test]
fn test_i128_add() {
    TestI128::test_add();
}

#[test]
fn test_i128_sub() {
    TestI128::test_sub();
}

#[test]
fn test_i128_mul() {
    TestI128::test_mul();
}

#[test]
#[ignore] // takes several minutes
fn test_i128_div() {
    TestI128::test_div();
}

#[test]
fn test_i128_pow() {
    TestI128::test_pow();
}

#[test]
fn test_i128_eq() {
    TestI128::test_eq();
}

#[test]
fn test_i128_ne() {
    TestI128::test_ne();
}

#[test]
fn test_i128_ge() {
    TestI128::test_ge();
}

#[test]
fn test_i128_gt() {
    TestI128::test_gt();
}

#[test]
fn test_i128_le() {
    TestI128::test_le();
}

#[test]
fn test_i128_lt() {
    TestI128::test_lt();
}

#[test]
fn test_i128_assert_eq() {
    TestI128::test_console_assert();
}

#[test]
fn test_i128_ternary() {
    TestI128::test_ternary();
}

#[test]
fn test_no_space_between_literal() {
    let program_string = include_str!("no_space_between_literal.leo");
    let program = parse_program(program_string);

    assert!(program.is_err());
}
