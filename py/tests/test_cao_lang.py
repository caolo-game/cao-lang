import pytest
import cao_lang as caoc


def test_compile_and_run():
    """
    Test if we can take a simple program and parse, compile and run it without error
    """

    PROGRAM_YAML = """
lanes:
    main: 
        cards:
            - ty: ScalarInt
              val: 5
            - ty: ScalarInt
              val: 5
            - ty: Add
"""

    program = caoc.CompilationUnit.from_yaml(PROGRAM_YAML)
    options = caoc.CompilationOptions()

    program = caoc.compile(program, options)

    caoc.run(program)


def test_get_version():
    v = caoc.native_version()
    assert isinstance(v, str)


def test_json():
    PROGRAM_JSON = """
    {
        "lanes": {
            "main": {
                "cards": [
                    { "ty": "Noop" }
                ]
            }
        }
    }
    """
    program = caoc.CompilationUnit.from_json(PROGRAM_JSON)
    options = caoc.CompilationOptions()

    program = caoc.compile(program, options)

    caoc.run(program)


def test_bad_json_is_value_error():
    PROGRAM_JSON = """
    {
        "lanes": {
            "main": {
                "cards": [ {} ]
            }
        }
    }
    """
    with pytest.raises(ValueError):
        caoc.CompilationUnit.from_json(PROGRAM_JSON)
