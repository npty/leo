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
    expect_compiler_error,
    generate_main_input,
    generate_test_input_u32,
    get_output,
    parse_program,
    parse_program_with_input,
    EdwardsTestCompiler,
};
use leo_ast::InputValue;

#[test]
fn test_assert() {
    let program_string = include_str!("assert.leo");
    let mut program_1_pass = parse_program(program_string).unwrap();
    let mut program_0_pass = program_1_pass.clone();
    let mut program_2_fail = program_1_pass.clone();

    // Check that an input value of 1 satisfies the constraint system

    let main_input = generate_main_input(vec![("a", generate_test_input_u32(1))]);

    program_1_pass.set_main_input(main_input);

    assert_satisfied(program_1_pass);

    // Check that an input value of 0 satisfies the constraint system

    let main_input = generate_main_input(vec![("a", generate_test_input_u32(0))]);

    program_0_pass.set_main_input(main_input);

    assert_satisfied(program_0_pass);

    // Check that an input value of 2 does not satisfy the constraint system

    let main_input = generate_main_input(vec![("a", generate_test_input_u32(2))]);

    program_2_fail.set_main_input(main_input);

    expect_compiler_error(program_2_fail);
}

#[test]
fn test_mutate() {
    let program_string = include_str!("mutate.leo");
    let mut program_1_pass = parse_program(program_string).unwrap();
    let mut program_0_pass = program_1_pass.clone();

    // Check that an input value of 1 satisfies the constraint system

    let main_input = generate_main_input(vec![("a", generate_test_input_u32(1))]);

    program_1_pass.set_main_input(main_input);

    assert_satisfied(program_1_pass);

    // Check that an input value of 0 satisfies the constraint system

    let main_input = generate_main_input(vec![("a", generate_test_input_u32(0))]);

    program_0_pass.set_main_input(main_input);

    assert_satisfied(program_0_pass);
}

#[test]
fn test_for_loop() {
    let program_string = include_str!("for_loop.leo");
    let mut program_true_6 = parse_program(program_string).unwrap();
    let mut program_false_0 = program_true_6.clone();

    // Check that an input value of true satisfies the constraint system

    let main_input = generate_main_input(vec![("a", Some(InputValue::Boolean(true)))]);

    program_true_6.set_main_input(main_input);

    assert_satisfied(program_true_6);

    // Check that an input value of false satisfies the constraint system

    let main_input = generate_main_input(vec![("a", Some(InputValue::Boolean(false)))]);

    program_false_0.set_main_input(main_input);

    assert_satisfied(program_false_0);
}

#[test]
fn test_chain() {
    let program_string = include_str!("chain.leo");
    let mut program_1_1 = parse_program(program_string).unwrap();
    let mut program_2_2 = program_1_1.clone();
    let mut program_4_3 = program_1_1.clone();

    // Check that an input of 1 outputs 1

    let main_input = generate_main_input(vec![
        ("a", generate_test_input_u32(1)),
        ("b", generate_test_input_u32(1)),
    ]);

    program_1_1.set_main_input(main_input);

    assert_satisfied(program_1_1);

    // Check that an input of 2 outputs 2

    let main_input = generate_main_input(vec![
        ("a", generate_test_input_u32(2)),
        ("b", generate_test_input_u32(2)),
    ]);

    program_2_2.set_main_input(main_input);

    assert_satisfied(program_2_2);

    // Check that an input of 4 outputs 3

    let main_input = generate_main_input(vec![
        ("a", generate_test_input_u32(4)),
        ("b", generate_test_input_u32(3)),
    ]);

    program_4_3.set_main_input(main_input);

    assert_satisfied(program_4_3);
}

#[test]
fn test_nested() {
    let program_string = include_str!("nested.leo");
    let mut program_true_true_3 = parse_program(program_string).unwrap();
    let mut program_true_false_1 = program_true_true_3.clone();
    let mut program_false_false_0 = program_true_true_3.clone();

    // Check that an input value of true true outputs 3

    let main_input = generate_main_input(vec![
        ("a", Some(InputValue::Boolean(true))),
        ("b", Some(InputValue::Boolean(true))),
        ("c", generate_test_input_u32(3)),
    ]);

    program_true_true_3.set_main_input(main_input);

    assert_satisfied(program_true_true_3);

    // Check that an input value of true false outputs 1

    let main_input = generate_main_input(vec![
        ("a", Some(InputValue::Boolean(true))),
        ("b", Some(InputValue::Boolean(false))),
        ("c", generate_test_input_u32(1)),
    ]);

    program_true_false_1.set_main_input(main_input);

    assert_satisfied(program_true_false_1);

    // Check that an input value of false false outputs 0

    let main_input = generate_main_input(vec![
        ("a", Some(InputValue::Boolean(false))),
        ("b", Some(InputValue::Boolean(false))),
        ("c", generate_test_input_u32(0)),
    ]);

    program_false_false_0.set_main_input(main_input);

    assert_satisfied(program_false_false_0);
}

fn output_one(program: EdwardsTestCompiler) {
    let expected = include_bytes!("output/registers_one.out");
    let actual = get_output(program);

    assert_eq!(expected, actual.bytes().as_slice());
}

fn output_zero(program: EdwardsTestCompiler) {
    let expected = include_bytes!("output/registers_zero.out");
    let actual = get_output(program);

    assert_eq!(expected, actual.bytes().as_slice());
}

#[test]
fn test_multiple_returns() {
    let program_string = include_str!("multiple_returns.leo");

    // Check that an input value of 1 writes 1 to the output registers

    let registers_one_string = include_str!("input/registers_one.in");
    let program = parse_program_with_input(program_string, registers_one_string).unwrap();

    output_one(program);

    // Check that an input value of 0 writes 0 to the output registers

    let registers_zero_string = include_str!("input/registers_zero.in");
    let program = parse_program_with_input(program_string, registers_zero_string).unwrap();

    output_zero(program);
}

#[test]
fn test_cond_switch() {
    let input_string = include_str!("input/cond_switch.in");
    let program_string = include_str!("cond_switch.leo");
    let expect_output = include_bytes!("output/cond_switch.out");

    let program = parse_program_with_input(program_string, input_string).unwrap();

    let actual_output = get_output(program);

    assert_eq!(expect_output, actual_output.bytes().as_slice());
}
