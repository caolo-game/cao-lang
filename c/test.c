#include "cao-lang.h"
#include <stdio.h>
#include <string.h>
#include <assert.h>

const uint8_t* empty_program = (uint8_t*)"{\"lanes\":[{\"name\":\"boi\",\"cards\":[]}]}\0";

int main()
{
    cao_CompiledProgram program = cao_new_compiled_program();
    const cao_CompileResult result = cao_compile_json(empty_program, strlen((const char*)empty_program), &program);

    assert(result == cao_CompileResult_Ok);

    cao_free_compiled_program(program);

    puts("Boiiiii");
}
