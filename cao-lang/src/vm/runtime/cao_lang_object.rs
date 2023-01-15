use super::{cao_lang_string::CaoLangString, cao_lang_table::CaoLangTable};

// note Gray is not actually useful for now, but it might come in handy if we want to do finalizers
#[derive(Debug, Clone, Copy)]
pub enum GcMarker {
    /// Unprocessed
    White,
    /// Visited
    Gray,
    /// Done
    Black,
}

pub struct CaoLangObject {
    pub marker: GcMarker,
    pub body: CaoLangObjectBody,
}

pub enum CaoLangObjectBody {
    Table(CaoLangTable),
    String(CaoLangString),
}

impl CaoLangObject {
    pub fn type_name(&self) -> &'static str {
        match &self.body {
            CaoLangObjectBody::Table(_) => "Table",
            CaoLangObjectBody::String(_) => "String",
        }
    }

    pub fn as_table(&self) -> Option<&CaoLangTable> {
        match &self.body {
            CaoLangObjectBody::Table(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_table_mut(&mut self) -> Option<&mut CaoLangTable> {
        match &mut self.body {
            CaoLangObjectBody::Table(v) => Some(v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.body {
            CaoLangObjectBody::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn len(&self) -> usize {
        match &self.body {
            CaoLangObjectBody::Table(t) => t.len(),
            CaoLangObjectBody::String(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl std::hash::Hash for CaoLangObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match &self.body {
            CaoLangObjectBody::Table(o) => {
                for (k, v) in o.iter() {
                    k.hash(state);
                    v.hash(state);
                }
            }
            CaoLangObjectBody::String(s) => {
                s.as_str().hash(state);
            }
        }
    }
}

impl PartialEq for CaoLangObject {
    fn eq(&self, other: &Self) -> bool {
        match (&self.body, &other.body) {
            (CaoLangObjectBody::Table(lhs), CaoLangObjectBody::Table(rhs)) => {
                if lhs.len() != rhs.len() {
                    return false;
                }
                for ((kl, vl), (kr, vr)) in lhs.iter().zip(rhs.iter()) {
                    if kl != kr || vl != vr {
                        return false;
                    }
                }
                true
            }
            (CaoLangObjectBody::String(lhs), CaoLangObjectBody::String(rhs)) => {
                lhs.as_str().eq(rhs.as_str())
            }
            _ => false,
        }
    }
}

impl PartialOrd for CaoLangObject {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.eq(other)
            .then_some(std::cmp::Ordering::Equal)
            .or_else(|| {
                // equal len but non-eq objects should not return Equal
                let res = self.len().cmp(&other.len());
                match res {
                    std::cmp::Ordering::Equal => None,
                    _ => Some(res),
                }
            })
    }
}
