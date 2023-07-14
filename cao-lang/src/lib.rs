//! ## Programs
//!
//! __TBA__
//!

#![recursion_limit = "256"]

mod alloc;
pub mod collections;
pub mod compiled_program;
pub mod compiler;
pub mod instruction;
pub mod prelude;
pub mod procedures;
pub mod stdlib;
pub mod traits;
pub mod value;
pub mod vm;

mod bytecode;

pub mod version {
    include!(concat!(env!("OUT_DIR"), "/cao_lang_version.rs"));
}

use std::{mem::size_of, str::FromStr};

use bytemuck::{Pod, Zeroable};

use crate::instruction::Instruction;

#[derive(Pod, Zeroable, Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct VariableId(u32);

impl FromStr for VariableId {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = u32::from_str(s)?;
        Ok(VariableId(inner))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct StrPointer(pub *mut u8);

impl StrPointer {
    /// # Safety
    ///
    /// Must be called with ptr obtained from a `string_literal` instruction, before the last `clear`!
    ///
    /// # Return value
    ///
    /// Returns None if the underlying string is not valid utf8
    pub unsafe fn get_str<'a>(self) -> Option<&'a str> {
        let ptr = self.0;
        if ptr.is_null() {
            return None;
        }
        let len = *(ptr as *const u32);
        let ptr = ptr.add(size_of::<u32>());
        std::str::from_utf8(std::slice::from_raw_parts(ptr, len as usize)).ok()
    }
}

pub type InputString = String;
pub type VarName = String;
