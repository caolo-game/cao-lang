extern "C" {
#include <cao-lang.h>
}

#include <gtest/gtest.h>

TEST(Runs, EmptyProgram) {
  const uint8_t *program_json = (uint8_t *)R"prog(
{
  "submodules": [],
  "imports": [],
  "functions": [
    ["main", {
      "arguments": [],
      "cards": [
      ]
    }]
  ]
}
)prog";

  cao_CaoCompiledProgram program = cao_new_compiled_program();
  const cao_CompileResult compile_result = cao_compile_json(
      program_json, strlen((const char *)program_json), &program);

  ASSERT_EQ(compile_result, cao_CompileResult_Ok);

  cao_CaoVm vm = cao_new_vm();

  const cao_ExecutionResult run_result = cao_run_program(program, vm);

  cao_free_vm(&vm);
  cao_free_compiled_program(&program);
  ASSERT_EQ(run_result, cao_ExecutionResult_Ok);
}
