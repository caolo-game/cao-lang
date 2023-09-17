import json

import pytest
import cao_lang as caoc


def test_compile_and_run():
    """
    Test if we can take a simple program and parse, compile and run it without error
    """

    program_yaml = """
functions:
    - - main
      - arguments: []
        cards:
            - !Add
                - !ScalarInt 5
                - !ScalarInt 5
            - !Call
                function_name: "foo.bar"
                args: []
imports: []
submodules:
    - - foo
      - imports: []
        submodules: []
        functions:
            - - bar
              - arguments: []
                cards:
                    - !ScalarInt 42

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
        "functions": [
            ["main", {
                "arguments": [],
                "cards": [
                    {  "Call": {"function_name": "foo.bar", "args":[] } }
                ]
            }]
        ],
        "imports": [],
        "submodules": [
            ["foo", {
                "imports": [],
                "submodules": [],
                "functions": [
                    ["bar", {
                        "arguments": [],
                        "cards": [
                            {  "ScalarNil":null }
                        ]
                    }]
                ]
            }]
        ]
    }
    """
    program = caoc.CompilationUnit.from_json(program_json)
    options = caoc.CompilationOptions()

    program = caoc.compile(program, options)

    caoc.run(program)


def test_bad_json_is_value_error():
    program_json = """
    {
        "functions": [
            ["main", {
                "cards": [ {} ]
            }]
        ]
    }
    """
    with pytest.raises(ValueError):
        caoc.CompilationUnit.from_json(program_json)


def test_recursion_limit():
    program = {
        "imports": [],
        "submodules": [],
        "functions": [["main", {"arguments": [], "cards": []}]],
    }
    _pr = program
    for _ in range(2):
        _pr["submodules"].append(
            ["foo", {"imports": [], "submodules": [], "functions": []}]
        )
        _pr = _pr["submodules"][0][1]

    program = caoc.CompilationUnit.from_json(json.dumps(program))
    options = caoc.CompilationOptions()

    # default options should not an raise error
    _ = caoc.compile(program, options)

    with pytest.raises(ValueError):
        options.recursion_limit = 1
        _ = caoc.compile(program, options)
