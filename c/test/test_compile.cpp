extern "C" {
#include <cao-lang.h>
}

#include <gtest/gtest.h>

TEST(Compile, MultiLaneProgram) {
  const char *program_json = R"prog(
{
  "submodules": {},
  "imports": [],
  "lanes": {
    "main": {
      "arguments": [],
      "cards": [
        {
          "StringLiteral": "RESOURCE"
        },
        {
          "CallNative": "parse_find_constant"
        },
        {
          "CallNative": "find_closest"
        },
        {
          "SetVar": "resource"
        },
        {
          "ReadVar": "resource"
        },
        {
          "ScalarNil": null
        },
        {
          "Equals": null
        },
        {
          "IfTrue": {
            "Jump": "resource_error"
          }
        },
        {
          "ReadVar": "resource"
        },
        {
          "ReadVar": "resource"
        },
        {
          "CallNative": "mine"
        },
        {
          "ScalarInt": 0
        },
        {
          "Equals": null
        },
        {
          "IfElse": {
            "then":{ 
                "Jump":"mine_success"
              }
            ,
            "else":  {
                "Jump":"approach_resource"
              }
          }
        }
      ]
    },
    "approach_resource": {
      "arguments": [
        "resource"
      ],
      "cards": [
        {
          "ReadVar": "resource"
        },
        {
          "StringLiteral": "Work work...\nMove Result: "
        },
        {
          "CallNative": "console_log"
        },
        {
          "CallNative": "approach_entity"
        },
        {
          "CallNative": "console_log"
        }
      ]
    },
    "resource_error": {
      "arguments": [],
      "cards": [
        {
          "StringLiteral": "No resource found"
        },
        {
          "CallNative": "console_log"
        },
        {
          "Abort": null
        }
      ]
    },
    "mine_success": {
      "arguments": [],
      "cards": [
        {
          "StringLiteral": "I be mining baws"
        },
        {
          "CallNative": "console_log"
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
  "submodules": {},
  "imports": [],
  "lanes": {
    "main": {
      "arguments": [],
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
