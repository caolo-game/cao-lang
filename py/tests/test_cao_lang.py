import cao_lang as caoc


def test_compile_and_run():
    """
    Test if we can take a simple program and parse, compile and run it without error
    """

    PROGRAM_YAML = """
lanes:
    - cards:
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
