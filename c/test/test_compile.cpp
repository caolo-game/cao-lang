extern "C" {
#include <cao-lang.h>
}

#include <gtest/gtest.h>

TEST(Compile, TestEmpty)
{
    const uint8_t* empty_program = (uint8_t*)"{\"lanes\":[{\"name\":\"boi\",\"cards\":[]}]}\0";

    cao_CompiledProgram program = cao_new_compiled_program();
    const cao_CompileResult result = cao_compile_json(empty_program, strlen((const char*)empty_program), &program);

    EXPECT_EQ(result, cao_CompileResult_Ok);

    cao_free_compiled_program(program);
}

TEST(Compile, MultiLaneProgram)
{
    const uint8_t* program_json = (uint8_t*)R"prog(
{
  "lanes": [
    {
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
          "val": {
            "LaneName": "resource-error"
          }
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
            "then": {
              "LaneName": "mine-success"
            },
            "else": {
              "LaneName": "approach-resource"
            }
          }
        }
      ]
    },
    {
      "name": "approach-resource",
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
    {
      "name": "resource-error",
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
    {
      "name": "mine-success",
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
  ]
}
)prog";

    cao_CompiledProgram program = cao_new_compiled_program();
    const cao_CompileResult result = cao_compile_json(program_json, strlen((const char*)program_json), &program);

    EXPECT_EQ(result, cao_CompileResult_Ok);

    cao_free_compiled_program(program);
}
