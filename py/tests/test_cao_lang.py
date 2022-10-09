import json

import pytest
import cao_lang as caoc


def test_compile_and_run():
    """
    Test if we can take a simple program and parse, compile and run it without error
    """

    program_yaml = """
cards:
            1: !ScalarInt 5
            2: !Add
            3: !Jump "foo.bar"
lanes:
    main: 
        arguments: []
        cards:
            - 1
            - 1
            - 2
            - 3
imports: []
submodules:
    foo:
        imports: []
        submodules: {}
        cards:
            1: !ScalarInt 42
        lanes:
            bar:
                arguments: []
                cards:
                    - 1

"""

    program = caoc.CompilationUnit.from_yaml(program_yaml)
    options = caoc.CompilationOptions()

    program = caoc.compile(program, options)

    caoc.run(program)


def test_get_version():
    v = caoc.native_version()
    assert isinstance(v, str)


def test_json():
    program_json = """
    {
        "cards": {
            "1": {  "Jump": "foo.bar" }
        },
        "lanes": {
            "main": {
                "arguments": [],
                "cards": [ 1 ]
            }
        },
        "imports": [],
        "submodules": {
            "foo": {
                "imports": [],
                "submodules": {},
                "cards": {
                    "1": {  "Noop":null }
                },
                "lanes": {
                    "bar": {
                        "arguments": [],
                        "cards": [ 1 ]
                    }
                }
            }
        }
    }
    """
    program = caoc.CompilationUnit.from_json(program_json)
    options = caoc.CompilationOptions()

    program = caoc.compile(program, options)

    caoc.run(program)


def test_bad_json_is_value_error():
    program_json = """
    {
        "lanes": {
            "main": {
                "cards": [ {} ]
            }
        }
    }
    """
    with pytest.raises(ValueError):
        caoc.CompilationUnit.from_json(program_json)


def test_recursion_limit():
    program = {
        "imports": [],
        "submodules": {},
        "cards": {},
        "lanes": {"main": {"arguments": [], "cards": []}},
    }
    _pr = program
    for _ in range(2):
        _pr["submodules"]["foo"] = {
            "imports": [],
            "submodules": {},
            "lanes": {},
            "cards": {},
        }
        _pr = _pr["submodules"]["foo"]

    program = caoc.CompilationUnit.from_json(json.dumps(program))
    options = caoc.CompilationOptions()

    # default options should not an raise error
    _ = caoc.compile(program, options)

    with pytest.raises(ValueError):
        options.recursion_limit = 1
        _ = caoc.compile(program, options)
