use std::ops::DerefMut;

use cao_lang::{
    compiler::{CompositeCard, Module, UnaryExpression},
    prelude::*,
};

#[test]
fn composite_card_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::CompositeCard(Box::new(CompositeCard {
                ty: "triplepog".to_string(),
                cards: vec![Card::set_global_var(
                    "result",
                    Card::StringLiteral("poggers".to_owned()),
                )],
            }))),
        )]
        .into(),
    };

    let program = compile(cu, None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let res = vm.read_var_by_name("result", &program.variables).unwrap();
    let ress = unsafe { res.as_str().expect("Failed to read string") };

    assert_eq!(ress, "poggers");
}

#[test]
fn test_trace_entry() {
    let ir = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default().with_card(Card::call_function("pooh", vec![])),
            ),
            (
                "pooh".into(),
                Function::default().with_card(Card::call_native("non-existent-function", vec![])),
            ),
        ]
        .into(),
    };
    let program = compile(ir.clone(), None).unwrap();

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    let err = vm.run(&program).expect_err("run");

    let trace = err.trace;

    let error_card = ir
        .get_card(&trace[0].index)
        .expect("Expected to find the errored card");

    assert!(matches!(error_card, Card::CallNative(_)));
}

#[test]
fn test_string_w_utf8() {
    let test_str = "winnie the pooh is 🔥🔥🔥 ";
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::set_global_var(
                "result",
                Card::StringLiteral(test_str.to_string()),
            )),
        )]
        .into(),
    };

    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let varid = program.variable_id("result").expect("varid");
    let ptr = vm.read_var(varid).expect("read var");
    let ress = unsafe { ptr.as_str().expect("Failed to read string") };

    assert_eq!(test_str, ress);
}

#[test]
fn test_string_param() {
    let name = "fooboi";
    let test_str = "tiggers boi";

    struct State {
        res: String,
    }

    let fun = move |vm: &mut Vm<State>, arg: &str| {
        vm.auxiliary_data.res = arg.to_string();
        Ok(Value::Nil)
    };

    let mut vm = Vm::new(State {
        res: "".to_string(),
    })
    .unwrap();
    vm.register_native_function(name, into_f1(fun)).unwrap();

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::StringLiteral(test_str.to_string()))
                .with_card(Card::call_native(name, vec![])),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    vm.run(&program).expect("run");
    let aux = vm.unwrap_aux();

    assert_eq!(aux.res, test_str);
}

#[test]
fn simple_if_statement() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default().with_card(Card::IfTrue(Box::new([
                    Card::ScalarInt(42),
                    Card::call_function("pooh", vec![]),
                ]))),
            ),
            (
                "pooh".into(),
                Function::default().with_card(Card::set_global_var("result", Card::ScalarInt(69))),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("run");

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), Value::Integer(69));
}

#[test]
fn simple_if_statement_skips_if_false() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::IfTrue(Box::new([
                    Card::ScalarInt(0),
                    Card::set_global_var("result", Card::ScalarInt(69)),
                ])))
                .with_card(Card::IfFalse(Box::new([
                    Card::ScalarInt(1),
                    Card::set_global_var("result", Card::ScalarInt(42)),
                ]))),
        )]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).unwrap();

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).unwrap();

    let varid = program.variable_id("result").unwrap();
    let value = vm.read_var(varid);
    assert!(
        value.is_none(),
        "expected value to be none, instead got: {:?}",
        value
    );
}

fn if_else_test(condition: Card, true_res: Card, false_res: Card, expected_result: Value) {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default()
                    .with_card(Card::IfElse(Box::new([
                        condition,
                        Card::call_function("pooh", vec![]),
                        Card::call_function("tiggers", vec![]),
                    ])))
                    .with_card(Card::set_global_var("result2", Card::ScalarInt(0xbeef))),
            ),
            (
                "pooh".into(),
                Function::default().with_cards(vec![Card::set_global_var("result", true_res)]),
            ),
            (
                "tiggers".into(),
                Function::default().with_cards(vec![Card::set_global_var("result", false_res)]),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(1000);
    vm.run(&program).expect("program run");

    let varid = program.variable_id("result").expect("varid");
    assert_eq!(vm.read_var(varid).expect("read var"), expected_result);

    // test if the cards after the if statement were executed
    let varid = program.variable_id("result2").expect("varid");
    assert_eq!(
        vm.read_var(varid).expect("read var"),
        Value::Integer(0xbeef)
    );
}

#[test]
fn simple_if_else_statement_test_then() {
    if_else_test(
        Card::ScalarInt(1),
        Card::ScalarInt(42),
        Card::ScalarInt(69),
        Value::Integer(42),
    );
}

#[test]
fn simple_if_else_statement_test_else() {
    if_else_test(
        Card::ScalarInt(0),
        Card::ScalarInt(42),
        Card::ScalarInt(69),
        Value::Integer(69),
    );
}

#[test]
fn test_local_variable() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                // init the global variable
                .with_card(Card::set_global_var("bar", Card::ScalarInt(420)))
                // set another value in local var
                .with_card(Card::set_var("foo", Card::ScalarInt(123)))
                // read the var and set the global variable
                .with_card(Card::set_global_var("bar", Card::read_var("foo"))),
        )]
        .into(),
    };

    let program = compile(program, None).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    vm.run(&program).unwrap();

    let res = vm
        .read_var_by_name("bar", &program.variables)
        .expect("Failed to read result variable");
    assert_eq!(res, Value::Integer(123));
}

#[test]
fn local_variable_doesnt_leak_out_of_scope() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default()
                    .with_card(Card::set_var("foo", Card::ScalarInt(123)))
                    .with_card(Card::call_function("bar", vec![])),
            ),
            (
                "bar".into(),
                Function::default().with_card(Card::read_var("foo")),
            ),
        ]
        .into(),
    };

    let program = compile(program, None).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    let res = vm.run(&program);
    let _name = "foo".to_string();
    assert!(matches!(
        res.map_err(|err| err.payload),
        Err(ExecutionErrorPayload::VarNotFound(_name))
    ));
}

#[test]
fn simple_for_loop() {
    let program = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default().with_cards(vec![
                    // init the result variable
                    Card::set_global_var("result", Card::ScalarInt(0)),
                    // loop
                    Card::repeat(
                        Card::ScalarInt(5),
                        Some("i".to_string()),
                        Card::call_function("Loop", vec![Card::read_var("i")]),
                    ),
                ]),
            ),
            (
                "Loop".into(),
                Function::default().with_arg("i").with_cards(vec![
                    // Add i to the global 'result' variable in each iteration
                    Card::set_global_var(
                        "result",
                        Card::Add(Box::new([Card::read_var("i"), Card::read_var("result")])),
                    ),
                ]),
            ),
        ]
        .into(),
    };
    let program = compile(program, Some(CompileOptions::new())).expect("compile");

    // Compilation was successful

    let mut vm = Vm::new(()).unwrap().with_max_iter(500);
    vm.run(&program).unwrap();

    let res = vm
        .read_var_by_name("result", &program.variables)
        .expect("Failed to read result variable");
    assert_eq!(res, Value::Integer((0..5).fold(0, std::ops::Add::add)));
}

#[test]
fn call_native_test() {
    let name = "foo";
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_cards(vec![Card::call_native(name, vec![])]),
        )]
        .into(),
    };

    let prog = compile(cu, CompileOptions::new()).unwrap();

    struct State {
        called: bool,
    }

    let fun = move |vm: &mut Vm<State>| {
        vm.auxiliary_data.called = true;
        Ok(Value::Nil)
    };

    let mut vm = Vm::new(State { called: false }).unwrap();
    vm.register_native_function(name, fun).unwrap();
    vm.run(&prog).expect("run failed");
    assert!(vm.unwrap_aux().called);
}

#[test]
#[cfg(feature = "serde")]
fn test_function_registry() {
    struct State {
        call_0: bool,
        call_1: bool,
        call_2: bool,
        call_3: bool,
    }

    fn myfunc0(vm: &mut Vm<State>) -> Result<Value, ExecutionErrorPayload> {
        vm.auxiliary_data.call_0 = true;
        Ok(Value::Nil)
    }

    fn myfunc1(vm: &mut Vm<State>, i: i64) -> Result<Value, ExecutionErrorPayload> {
        vm.auxiliary_data.call_1 = true;
        assert_eq!(i, 42);
        Ok(Value::Nil)
    }

    fn myfunc2(vm: &mut Vm<State>, i: i64, j: f64) -> Result<Value, ExecutionErrorPayload> {
        vm.auxiliary_data.call_2 = true;
        assert_eq!(i, 12);
        assert_eq!(j, 4.2);
        Ok(Value::Nil)
    }

    fn myfunc3(
        vm: &mut Vm<State>,
        i: i64,
        j: f64,
        b: bool,
    ) -> Result<Value, ExecutionErrorPayload> {
        vm.auxiliary_data.call_3 = true;
        assert_eq!(i, 33);
        assert_eq!(j, 2.88);
        assert_eq!(b, false);
        Ok(Value::Nil)
    }

    let mut vm = Vm::new(State {
        call_0: false,
        call_1: false,
        call_2: false,
        call_3: false,
    })
    .unwrap();

    // if this compiles we're good to go
    vm.register_native_function("func0", myfunc0).unwrap();
    vm.register_native_function("func1", into_f1(myfunc1))
        .unwrap();
    vm.register_native_function("func2", into_f2(myfunc2))
        .unwrap();
    vm.register_native_function("func3", into_f3(myfunc3))
        .unwrap();

    const PROG: &str = r#"
submodules: []
imports: []
functions:
    - - main
      - arguments: []
        cards:
            - !CallNative
                name: "func0"
                args: []
            - !CallNative 
                name: "func1"
                args:
                    - !ScalarInt 42
            - !CallNative
                name: "func2"
                args:
                    - !ScalarInt 12
                    - !ScalarFloat 4.2

            - !CallNative
                name: "func3"
                args:
                    - !ScalarInt 33
                    - !ScalarFloat 2.88
                    - !ScalarInt 0

"#;
    let cu = serde_yaml::from_str(PROG).unwrap();
    let prog = compile(cu, CompileOptions::new()).unwrap();

    vm.run(&prog).expect("run failed");

    let state = vm.unwrap_aux();
    assert!(state.call_0);
    assert!(state.call_1);
    assert!(state.call_2);
    assert!(state.call_3);
}

#[test]
fn jump_function_w_params_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default()
                    .with_card(Card::StringLiteral("winnie the pooh".to_owned()))
                    .with_card(Card::ScalarInt(42))
                    .with_card(Card::call_function("pooh", vec![])),
            ),
            (
                "pooh".into(),
                Function::default()
                    .with_arg("foo")
                    .with_arg("bar")
                    .with_card(Card::set_global_var("g_foo", Card::read_var("foo")))
                    .with_card(Card::set_global_var("g_bar", Card::read_var("bar"))),
            ),
        ]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
    let foo = vm
        .read_var_by_name("g_foo", &program.variables)
        .expect("Failed to read foo variable");
    let bar = vm
        .read_var_by_name("g_bar", &program.variables)
        .expect("Failed to read bar variable");
    dbg!(foo, bar);
    assert!(matches!(foo, Value::Integer(42)));
    match bar {
        Value::Object(s) => unsafe {
            let val = s.as_ref().as_str().unwrap();
            assert_eq!(val, "winnie the pooh");
        },
        _ => panic!("Unexpected value set for bar {:?}", bar),
    }
}

#[test]
fn len_test_empty() {
    // happy path
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::set_global_var(
                "g_result",
                Card::Len(UnaryExpression::new(Card::CreateTable)),
            )),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let len = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    assert_eq!(len, Value::Integer(0));
}

#[test]
fn len_test_happy() {
    // happy path
    let t = "t";
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::set_var(t, Card::CreateTable))
                // first property
                .with_card(Card::set_property(
                    Card::ScalarInt(42),
                    Card::read_var(t),
                    Card::StringLiteral("asd".to_string()),
                ))
                // same property as above
                .with_card(Card::set_property(
                    Card::ScalarInt(69),
                    Card::read_var(t),
                    Card::StringLiteral("asd".to_string()),
                ))
                // new property
                .with_card(Card::set_property(
                    Card::ScalarInt(89),
                    Card::read_var(t),
                    Card::StringLiteral("basdasd".to_string()),
                ))
                // len
                .with_card(Card::set_global_var(
                    "g_result",
                    Card::Len(UnaryExpression::new(Card::read_var(t))),
                )),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let len = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    assert_eq!(len, Value::Integer(2));
}

#[test]
fn nested_module_can_call_self_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                functions: [
                    (
                        "win".into(),
                        Function::default().with_card(Card::call_function("nie", vec![])),
                    ),
                    (
                        "nie".into(),
                        Function::default().with_card(Card::set_global_var(
                            "g_result",
                            Card::StringLiteral("poggers".to_owned()),
                        )),
                    ),
                ]
                .into(),
            },
        )]
        .into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("winnie.win", vec![])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn invalid_import_is_error_test() {
    let cu = CaoProgram {
        submodules: [].into(),
        imports: ["winnie..pooh".into()].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("pooh", vec![])),
        )]
        .into(),
    };

    compile(cu, CompileOptions::new()).expect_err("compile");
}

#[test]
fn non_existent_import_is_error_test() {
    let cu = CaoProgram {
        submodules: [].into(),
        imports: ["winnie..pooh".into()].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("pooh", vec![])),
        )]
        .into(),
    };

    let err = compile(cu, CompileOptions::new()).expect_err("compile");

    // TODO: the error itself should be more meaningful
    dbg!(&err);
    assert!(matches!(
        err.payload,
        CompilationErrorPayload::InvalidJump { .. }
    ));
}

#[test]
fn import_in_submodule_test() {
    let cu = Module {
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                functions: [(
                    "pooh".into(),
                    Function::default().with_card(Card::set_global_var(
                        "g_result",
                        Card::StringLiteral("poggers".to_owned()),
                    )),
                )]
                .into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        functions: [(
            "run".into(),
            Function::default().with_card(Card::call_function("pooh", vec![])),
        )]
        .into(),
    };
    let cu = Module {
        imports: ["foo.run".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("run", vec![])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn can_import_submodule_test() {
    let cu = Module {
        submodules: [(
            "winnie".into(),
            Module {
                imports: Default::default(),
                submodules: Default::default(),
                functions: [(
                    "pooh".into(),
                    Function::default().with_card(Card::set_global_var(
                        "g_result",
                        Card::StringLiteral("poggers".to_owned()),
                    )),
                )]
                .into(),
            },
        )]
        .into(),
        imports: ["winnie.pooh".into()].into(),
        functions: [(
            "run".into(),
            Function::default().with_card(Card::call_function("pooh", vec![])),
        )]
        .into(),
    };
    let cu = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), cu)].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("winnie.pooh", vec![])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn can_import_function_from_super_module_test() {
    let winnie = Module {
        imports: ["super.super.pog".into()].into(),
        submodules: Default::default(),
        functions: [(
            "pooh".into(),
            Function::default().with_card(Card::call_function("pog", vec![])),
        )]
        .into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        functions: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        functions: [
            (
                "run".into(),
                Function::default().with_card(Card::call_function("winnie.pooh", vec![])),
            ),
            (
                "pog".into(),
                Function::default().with_card(Card::set_global_var(
                    "g_result",
                    Card::StringLiteral("poggers".to_owned()),
                )),
            ),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("bar.run", vec![])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn import_super_module_test() {
    let winnie = Module {
        imports: ["super.super.bar".into()].into(),
        submodules: Default::default(),
        functions: [(
            "pooh".into(),
            Function::default().with_card(Card::call_function("bar.pog", vec![])),
        )]
        .into(),
    };
    let foo = Module {
        submodules: [("winnie".into(), winnie)].into(),
        imports: Default::default(),
        functions: [].into(),
    };
    let bar = Module {
        imports: ["foo.winnie".into()].into(),
        submodules: [("foo".into(), foo)].into(),
        functions: [
            (
                "run".into(),
                Function::default().with_card(Card::call_function("winnie.pooh", vec![])),
            ),
            (
                "pog".into(),
                Function::default().with_card(Card::set_global_var(
                    "g_result",
                    Card::StringLiteral("poggers".to_owned()),
                )),
            ),
        ]
        .into(),
    };
    let cu = Module {
        imports: [].into(),
        submodules: [("bar".into(), bar)].into(),
        functions: [(
            "main".into(),
            Function::default().with_card(Card::call_function("bar.run", vec![])),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read foo variable");

    let result = unsafe { result.as_str().unwrap() };

    assert_eq!(result, "poggers");
}

#[test]
fn local_variable_regression_test() {
    /*
    In main:
        In composite card:
            set local var (1)

    In other function:
        in composite card:
            set another local var (2) (name can be different)
        in another composite card:
            read the local var

    Bug: Reads the variable (1) from main
    Should read variable (2)
    */
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [
            (
                "main".into(),
                Function::default().with_card(Card::call_function(
                    "tiggers",
                    vec![Card::set_var("pog", Card::scalar_int(69))],
                )),
            ),
            (
                "tiggers".into(),
                Function::default()
                    .with_card(Card::set_var("foo", Card::scalar_int(42)))
                    .with_card(Card::set_global_var("pooh", Card::read_var("foo"))),
            ),
        ]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("pooh", &program.variables)
        .expect("Failed to read pooh variable");

    assert_eq!(result.as_int().unwrap(), 42);
}

#[test]
fn simple_while_test() {
    const N: i64 = 42;
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [(
            "main".to_string(),
            Function::default()
                .with_card(Card::set_var("i", Card::ScalarInt(N)))
                .with_card(Card::set_global_var("pooh", Card::ScalarInt(0)))
                .with_card(Card::While(Box::new([
                    Card::read_var("i"),
                    Card::composite_card(
                        "body",
                        vec![
                            // Increment pooh
                            Card::set_global_var(
                                "pooh",
                                Card::Add(Box::new([Card::ScalarInt(1), Card::read_var("pooh")])),
                            ),
                            // decrement loop counter
                            Card::set_var(
                                "i",
                                Card::Sub(Box::new([Card::read_var("i"), Card::ScalarInt(1)])),
                            ),
                        ],
                    ),
                ]))),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("pooh", &program.variables)
        .expect("Failed to read pooh variable");

    assert_eq!(result.as_int().unwrap(), N);
}

#[test]
fn set_var_to_empty_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [(
            "main".to_string(),
            Function::default().with_card(Card::set_var("i", Card::ScalarNil)),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");
}

#[test]
fn callback_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [
            (
                "main".to_string(),
                Function::default().with_cards(vec![
                    Card::set_global_var("i", Card::ScalarInt(0)),
                    Card::call_function("call_callback", vec![Card::function_value("callback")]),
                    Card::call_function("call_callback", vec![Card::function_value("callback")]),
                    Card::call_function("call_callback", vec![Card::function_value("callback")]),
                    Card::call_function("call_callback", vec![Card::function_value("callback")]),
                ]),
            ),
            (
                "call_callback".to_string(),
                Function::default()
                    .with_arg("cb")
                    .with_cards(vec![Card::dynamic_call(Card::read_var("cb"), vec![])]),
            ),
            (
                "callback".to_string(),
                Function::default().with_cards(vec![Card::set_global_var(
                    "i",
                    Card::Add(Box::new([Card::read_var("i"), Card::ScalarInt(1)])),
                )]),
            ),
        ]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("i", &program.variables)
        .expect("Failed to read pooh variable");

    assert_eq!(result.as_int().unwrap(), 4);
}

#[test]
fn read_set_property_shorthand_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [(
            "main".to_string(),
            Function::default().with_cards(vec![
                Card::set_global_var("winnie", Card::CreateTable),
                //
                Card::set_var("winnie.foo", Card::ScalarInt(1)),
                //
                Card::set_var(
                    "winnie.foo",
                    Card::Add(Box::new([Card::ScalarInt(1), Card::read_var("winnie.foo")])),
                ),
            ]),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("winnie", &program.variables)
        .expect("Failed to read winnie variable");

    let result = unsafe { result.as_table().unwrap() };

    let foo = result.get("foo").unwrap();
    assert_eq!(*foo, Value::Integer(2));
}

#[test]
fn read_property_shorthand_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [(
            "main".to_string(),
            Function::default().with_arg("table").with_cards(vec![
                // table is pushed onto the stack
                Card::set_global_var("i", Card::read_var("table")),
                Card::set_global_var("j", Card::read_var("i.foo")),
            ]),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    let mut table_ptr = vm.init_table().unwrap();
    let table = table_ptr.deref_mut().as_table_mut().unwrap();
    let key = vm.init_string("foo").unwrap();
    table
        .insert(Value::Object(key.into_inner()), Value::Integer(42))
        .unwrap();
    vm.stack_push(Value::Object(table_ptr.into_inner()))
        .unwrap();

    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("j", &program.variables)
        .expect("Failed to read i variable");

    assert_eq!(result, Value::Integer(42));
}

#[tracing_test::traced_test]
#[test]
fn nested_read_set_property_shorthand_test() {
    let cu = Module {
        imports: [].into(),
        submodules: [].into(),
        functions: [(
            "main".to_string(),
            Function::default().with_cards(vec![
                Card::set_global_var("winnie", Card::CreateTable),
                //
                Card::set_var("winnie.foo", Card::CreateTable),
                Card::set_var("winnie.foo.bar", Card::CreateTable),
                Card::set_var("winnie.foo.bar.baz", Card::CreateTable),
                //
                Card::set_var("winnie.foo.bar.baz.pooh", Card::ScalarInt(42)),
            ]),
        )]
        .into(),
    };

    let program = compile(cu, CompileOptions::new()).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("winnie", &program.variables)
        .expect("Failed to read winnie variable");

    unsafe {
        let result = result.as_table().unwrap();
        let foo = result.get("foo").unwrap().as_table().unwrap();
        let bar = foo.get("bar").unwrap().as_table().unwrap();
        let baz = bar.get("baz").unwrap().as_table().unwrap();
        let pooh = baz.get("pooh").unwrap().as_int().unwrap();

        assert_eq!(pooh, 42);
    }
}

#[test]
fn native_function_object_call_test() {
    let name = "fooboi";
    let test_str = "tiggers boi";

    struct State {
        res: String,
    }

    let fun = move |vm: &mut Vm<State>, arg: &str| {
        vm.auxiliary_data.res = arg.to_string();
        Ok(Value::Nil)
    };

    let mut vm = Vm::new(State {
        res: "".to_string(),
    })
    .unwrap();
    vm.register_native_function(name, into_f1(fun)).unwrap();

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [(
            "main".into(),
            Function::default()
                .with_card(Card::StringLiteral(test_str.to_string()))
                .with_card(Card::dynamic_call(
                    Card::NativeFunction(name.to_string()),
                    vec![],
                )),
        )]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    vm.run(&program).expect("run");
    let aux = vm.unwrap_aux();

    assert_eq!(aux.res, test_str);
}

#[test]
#[tracing_test::traced_test]
fn closure_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                // create a closure that captures a local variable
                // and sets a global variable
                "createClosure".into(),
                Function::default()
                    .with_card(Card::set_var(
                        "result",
                        Card::string_card("winnie the pooh"),
                    ))
                    .with_card(Card::return_card(Card::Closure(Box::new(
                        Function::default()
                            .with_card(Card::set_global_var("g_result", Card::read_var("result"))),
                    )))),
            ),
            (
                "main".into(),
                Function::default()
                    .with_card(Card::set_var(
                        "fun",
                        Card::call_function("createClosure", vec![]),
                    ))
                    .with_card(Card::dynamic_call(Card::read_var("fun"), vec![])),
            ),
        ]
        .into(),
    };

    let program = compile(cu, None).expect("compile");
    program.print_disassembly();

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read g_result variable");

    dbg!(result);

    unsafe {
        let result = result.as_str().unwrap();

        assert_eq!(result, "winnie the pooh");
    }
}

#[test]
#[tracing_test::traced_test]
fn nested_function_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "createClosure".into(),
                Function::default().with_card(Card::return_card(Card::Closure(Box::new(
                    Function::default().with_card(Card::set_global_var(
                        "g_result",
                        Card::string_card("winnie the pooh"),
                    )),
                )))),
            ),
            (
                "main".into(),
                Function::default()
                    .with_card(Card::set_var(
                        "fun",
                        Card::call_function("createClosure", vec![]),
                    ))
                    .with_card(Card::dynamic_call(Card::read_var("fun"), vec![])),
            ),
        ]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read g_result variable");

    unsafe {
        let result = result.as_str().unwrap();

        assert_eq!(result, "winnie the pooh");
    }
}

#[test]
fn native_functions_can_call_cao_lang_function() {
    struct State {
        res: i64,
    }

    let fun = move |vm: &mut Vm<State>, arg: Value| {
        dbg!(vm.get_aux().res);
        let res = vm.run_function(arg)?;
        vm.get_aux_mut().res = res.as_int().unwrap();
        Ok(Value::Nil)
    };

    let mut vm = Vm::new(State { res: 0 }).unwrap();
    vm.register_native_function("foo", into_f1(fun)).unwrap();

    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                "main".into(),
                Function::default().with_card(Card::dynamic_call(
                    Card::NativeFunction("foo".to_string()),
                    vec![Card::Function("bar".to_string())],
                )),
            ),
            (
                "bar".into(),
                Function::default().with_card(Card::Return(UnaryExpression {
                    card: Box::new(Card::ScalarInt(42)),
                })),
            ),
        ]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    vm.run(&program).expect("run");
    let aux = vm.unwrap_aux();

    assert_eq!(aux.res, 42);
}

#[test]
#[tracing_test::traced_test]
fn closure_shared_capture_test() {
    let cu = CaoProgram {
        imports: Default::default(),
        submodules: Default::default(),
        functions: [
            (
                // create a closure that captures a local variable
                // and sets a global variable
                "createClosures".into(),
                Function::default()
                    .with_card(Card::set_var("foo", Card::string_card("winnie the pooh")))
                    .with_card(Card::set_global_var(
                        "g_write",
                        Card::Closure(Box::new(
                            Function::default()
                                // write to the same upvalue twice
                                // test if multiple captures of the same upvalue point to the same
                                // actual value
                                .with_card(Card::set_var("foo", Card::string_card("tiggers")))
                                .with_card(Card::set_var("foo", Card::string_card("kanga"))),
                        )),
                    ))
                    .with_card(Card::set_global_var(
                        "g_read",
                        Card::Closure(Box::new(
                            Function::default()
                                .with_card(Card::set_global_var("g_result", Card::read_var("foo"))),
                        )),
                    )),
            ),
            (
                "main".into(),
                Function::default()
                    .with_card(Card::set_var(
                        "fun",
                        Card::call_function("createClosures", vec![]),
                    ))
                    .with_card(Card::dynamic_call(Card::read_var("g_write"), vec![]))
                    .with_card(Card::dynamic_call(Card::read_var("g_read"), vec![])),
            ),
        ]
        .into(),
    };

    let program = compile(cu, None).expect("compile");

    let mut vm = Vm::new(()).unwrap();
    vm.run(&program).expect("run");

    let result = vm
        .read_var_by_name("g_result", &program.variables)
        .expect("Failed to read g_result variable");

    unsafe {
        let result = result.as_str().unwrap();

        assert_eq!(result, "kanga");
    }
}
