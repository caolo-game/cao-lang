use super::err_to_js;
use cao_lang::compiler as cc;
use cao_lang::compiler::NodeId;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=AstNode, inspectable)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    #[wasm_bindgen(skip)]
    pub instruction: cc::InstructionNode,
    pub child: Option<NodeId>,
}

#[wasm_bindgen(js_class=AstNode)]
impl AstNode {
    #[wasm_bindgen(constructor)]
    pub fn new(instruction: JsValue, child: Option<NodeId>) -> Result<AstNode, JsValue> {
        let instruction: cc::InstructionNode = instruction.into_serde().map_err(err_to_js)?;
        let node = Self { instruction, child };
        Ok(node)
    }

    /// Check if the passed-in object is a valid AstNode.
    /// Returns null if it is, and an error otherwise.
    #[wasm_bindgen(js_name=checkError)]
    pub fn check_error(value: &JsValue) -> Option<String> {
        let parsed: Result<AstNode, _> = value.into_serde();
        match parsed {
            Ok(_) => None,
            Err(e) => Some(format!("{:?}", e)),
        }
    }

    #[wasm_bindgen]
    pub fn empty() -> Self {
        Self {
            instruction: cc::InstructionNode::Pass,
            child: None,
        }
    }

    #[wasm_bindgen(js_name=loadInstruction)]
    pub fn load_instructon(&self) -> JsValue {
        JsValue::from_serde(&self.instruction).unwrap()
    }

    /// Return the name of this AstNode variant
    #[wasm_bindgen(js_name=variant)]
    pub fn variant(&self) -> String {
        self.instruction.name().to_owned()
    }

    /// Sets the instruction of `this` or throw an error if `instruction` was not a valid
    /// Instruction instance.
    #[wasm_bindgen(js_name=setInstruction)]
    pub fn set_instruction(&mut self, instruction: JsValue) -> Result<(), JsValue> {
        let instruction: cc::InstructionNode = instruction.into_serde().map_err(err_to_js)?;
        self.instruction = instruction;
        Ok(())
    }

    /// Sets the value of this instruction or throw an error if the input value was invalid.
    #[wasm_bindgen(js_name=setValue)]
    pub fn set_value(&mut self, value: JsValue) -> Result<(), JsValue> {
        use cc::InstructionNode::*;

        macro_rules! map_err {
            () => {
                |e| {
                    let err = format!(
                        "InstructionNode {:?} got an invalid value: {:?}, error: {:?}",
                        self.instruction, value, e
                    );
                    JsValue::from_serde(&err).unwrap()
                }
            };
        };

        let mut instruction = self.instruction.clone();
        match &mut instruction {
            Start | Pass | Add | Sub | Mul | Div | Exit | CopyLast | Less | LessOrEq | Equals
            | NotEquals | Pop | ClearStack => {
                if !value.is_null() {
                    return Err(JsValue::from_serde(&format!(
                        "InstructionNode {:?} must have `null` value but {:?} was provided",
                        self.instruction, value
                    ))
                    .unwrap());
                }
            }
            ScalarInt(node) => *node = value.into_serde().map_err(map_err!())?,
            ScalarFloat(node) => *node = value.into_serde().map_err(map_err!())?,
            ScalarLabel(node) => *node = value.into_serde().map_err(map_err!())?,
            ScalarArray(node) => *node = value.into_serde().map_err(map_err!())?,
            StringLiteral(node) => *node = value.into_serde().map_err(map_err!())?,
            Call(node) => *node = value.into_serde().map_err(map_err!())?,
            JumpIfTrue(node) => *node = value.into_serde().map_err(map_err!())?,
            Jump(node) => *node = value.into_serde().map_err(map_err!())?,
            SetVar(node) => *node = value.into_serde().map_err(map_err!())?,
            ReadVar(node) => *node = value.into_serde().map_err(map_err!())?,
            SubProgram(node) => *node = value.into_serde().map_err(map_err!())?,
        }
        self.instruction = instruction;
        Ok(())
    }
}
