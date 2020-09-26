use crate::ast_node::AstNode;
use cao_lang::compiler as cc;
use cao_lang::compiler::NodeId;
use serde_derive::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name=CompilationUnit, inspectable)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilationUnit {
    #[wasm_bindgen(skip)]
    pub inner: cc::CompilationUnit,
}

#[wasm_bindgen(js_class=CompilationUnit)]
impl CompilationUnit {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: cc::CompilationUnit::default(),
        }
    }

    #[wasm_bindgen(js_name=nodeDel)]
    /// Remove the node by given ID and return it if it was in `this`
    pub fn del_node(&mut self, id: i32) -> Option<AstNode> {
        self.inner.nodes.remove(&id).map(|node| AstNode {
            child: node.child,
            instruction: node.node.clone(),
        })
    }

    /// Gets a node by `id`. If the node was not found returns `null`.
    /// Note that this method will copy the node! If you want to persist changes to the node, use
    /// `nodeSet` once you're done!
    #[wasm_bindgen(js_name=nodeGet)]
    pub fn get_node(&self, id: i32) -> Option<AstNode> {
        self.inner.nodes.get(&id).map(|node| AstNode {
            child: node.child,
            instruction: node.node.clone(),
        })
    }

    #[wasm_bindgen(js_name=nodeSet)]
    pub fn set_node(&mut self, id: i32, node: AstNode) {
        let child = node.child;
        let node = cc::AstNode {
            child,
            node: node.instruction,
        };
        self.inner.nodes.insert(id, node);
    }

    #[wasm_bindgen(js_name=subProgramSet)]
    pub fn set_sub_program(&mut self, name: &str, start: NodeId) {
        let sub_programs = self
            .inner
            .sub_programs
            .get_or_insert_with(|| Default::default());
        sub_programs.insert(name.to_owned(), cc::SubProgram { start });
    }

    /// Gets a sub_program by `name`. If the sub_program was not found returns `null`.
    /// Note that this method will copy the sub_program! If you want to persist changes to the sub_program, use
    /// `sub_programSet` once you're done!
    #[wasm_bindgen(js_name=subProgramGet)]
    pub fn get_sub_program(&self, name: &str) -> JsValue {
        let sub_program = self
            .inner
            .sub_programs
            .as_ref()
            .and_then(|sub_programs| sub_programs.get(name));

        JsValue::from_serde(&sub_program).unwrap()
    }

    #[wasm_bindgen(js_name=subProgramHas)]
    pub fn has_sub_program(&self, name: &str) -> bool {
        self.inner
            .sub_programs
            .as_ref()
            .map(|sub_programs| sub_programs.contains_key(name))
            .unwrap_or(false)
    }

    /// Does nothing if `this` does not contain the sub_program.
    #[wasm_bindgen(js_name=subProgramDel)]
    pub fn del_sub_program(&mut self, name: &str) {
        if let Some(sub_programs) = self.inner.sub_programs.as_mut() {
            sub_programs.remove(name);
        }
    }
}

impl CompilationUnit {
    pub fn with_node(mut self, id: i32, node: AstNode) -> Self {
        self.set_node(id, node);
        self
    }
}
