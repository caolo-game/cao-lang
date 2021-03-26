import cao_lang as caoc


def test_compile_and_run():
    """
    Test if we can take a simple program and parse, compile and run it without error
    """

    PROGRAM_YAML = """
lanes:
    - cards:
        - ScalarInt: 5 
        - ScalarInt: 5 
        - Add: null
"""

    program = caoc.CompilationUnit.from_yaml(PROGRAM_YAML)
    options = caoc.CompilationOptions(breadcrumbs=True)

    program = caoc.compile(program, options)

    caoc.run(program)
