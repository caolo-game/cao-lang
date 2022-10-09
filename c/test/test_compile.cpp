extern "C" {
#include <cao-lang.h>
}

#include <gtest/gtest.h>

TEST(Compile, MultiLaneProgram) {
  const char *program_json = R"prog(
{
    "submodules":{
    },
    "imports":[
    ],
    "cards":{
        "1":{
            "StringLiteral":"RESOURCE"
        },
        "2":{
            "CallNative":"parse_find_constant"
        },
        "3":{
            "CallNative":"find_closest"
        },
        "4":{
            "SetVar":"resource"
        },
        "5":{
            "ReadVar":"resource"
        },
        "6":{
            "ScalarNil":null
        },
        "7":{
            "Equals":null
        },
        "8":{
            "IfTrue":15
        },
        "9":{
            "ReadVar":"resource"
        },
        "10":{
            "ReadVar":"resource"
        },
        "11":{
            "CallNative":"mine"
        },
        "12":{
            "ScalarInt":0
        },
        "13":{
            "Equals":null
        },
        "14":{
            "IfElse":{
                "then":16,
                "else":17
            }
        },
        "15":{
            "Jump":"resource_error"
        },
        "16":{
            "Jump":"mine_success"
        },
        "17":{
            "Jump":"approach_resource"
        },
        "18":{
            "ReadVar":"resource"
        },
        "19":{
            "StringLiteral":"Work work...\nMove Result: "
        },
        "20":{
            "CallNative":"console_log"
        },
        "21":{
            "CallNative":"approach_entity"
        },
        "22":{
            "CallNative":"console_log"
        },
        "23":{
            "StringLiteral":"No resource found"
        },
        "24":{
            "CallNative":"console_log"
        },
        "25":{
            "Abort":null
        },
        "26":{
            "StringLiteral":"I be mining baws"
        },
        "27":{
            "CallNative":"console_log"
        }
    },
    "lanes":{
        "main":{
            "arguments":[
                
            ],
            "cards":[
                1,
                2,
                3,
                4,
                5,
                6,
                7,
                8,
                9,
                10,
                11,
                12,
                13,
                14,
                15,
                16,
                17
            ]
        },
        "approach_resource":{
            "arguments":[
                "resource"
            ],
            "cards":[
                18,
                19,
                20,
                21,
                22
            ]
        },
        "resource_error":{
            "arguments":[
                
            ],
            "cards":[
                23,
                24,
                25
            ]
        },
        "mine_success":{
            "arguments":[
                
            ],
            "cards":[
                26,
                27
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
  "cards": {},
  "lanes": {
    "main": {
      "arguments": [],
      "cards": []
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
