extern "C" {
#include <cao-lang.h>
}

#include <gtest/gtest.h>

TEST(Compile, MultiLaneProgram) {
  const char *program_json = R"prog(
{
  "lanes": {
    "main": {
      "name": "main",
      "cards": [
        {
          "ty": "StringLiteral",
          "val": "RESOURCE"
        },
        {
          "ty": "CallNative",
          "val": "parse_find_constant"
        },
        {
          "ty": "CallNative",
          "val": "find_closest"
        },
        {
          "ty": "SetVar",
          "val": "resource"
        },
        {
          "ty": "ReadVar",
          "val": "resource"
        },
        {
          "ty": "ScalarNil"
        },
        {
          "ty": "Equals"
        },
        {
          "ty": "IfTrue",
          "val": 
            "resource_error"
          
        },
        {
          "ty": "ReadVar",
          "val": "resource"
        },
        {
          "ty": "ReadVar",
          "val": "resource"
        },
        {
          "ty": "CallNative",
          "val": "mine"
        },
        {
          "ty": "ScalarInt",
          "val": 0
        },
        {
          "ty": "Equals"
        },
        {
          "ty": "IfElse",
          "val": {
            "then": 
              "mine_success"
            ,
            "else": 
              "approach_resource"
            
          }
        }
      ]
    },
    "approach_resource": {
      "name": "approach_resource",
      "arguments": [
        "resource"
      ],
      "cards": [
        {
          "ty": "ReadVar",
          "val": "resource"
        },
        {
          "ty": "StringLiteral",
          "val": "Work work...\nMove Result: "
        },
        {
          "ty": "CallNative",
          "val": "console_log"
        },
        {
          "ty": "CallNative",
          "val": "approach_entity"
        },
        {
          "ty": "CallNative",
          "val": "console_log"
        }
      ]
    },
    "resource_error": {
      "name": "resource_error",
      "cards": [
        {
          "ty": "StringLiteral",
          "val": "No resource found"
        },
        {
          "ty": "CallNative",
          "val": "console_log"
        },
        {
          "ty": "Abort"
        }
      ]
    },
    "mine_success": {
      "name": "mine_success",
      "cards": [
        {
          "ty": "StringLiteral",
          "val": "I be mining baws"
        },
        {
          "ty": "CallNative",
          "val": "console_log"
        }
      ]
    }
}
}
)prog";

  cao_CaoCompiledProgram program = cao_new_compiled_program();
  const cao_CompileResult result = cao_compile_json(
      (const uint8_t *)program_json, strlen(program_json), &program);

  EXPECT_EQ(result, cao_CompileResult_Ok);

  cao_free_compiled_program(&program);
}

TEST(Runs, EmptyProgram) {
  const uint8_t *program_json = (uint8_t *)R"prog(
{
  "lanes": {
    "main": {
      "name": "main",
      "cards": [
      ]
    }
  }
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
