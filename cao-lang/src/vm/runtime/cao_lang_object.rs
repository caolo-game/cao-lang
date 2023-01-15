use super::cao_lang_table::CaoLangTable;

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

#[derive(Clone, Debug)]
pub enum CaoLangObjectBody {
    Table(CaoLangTable),
}

impl CaoLangObject {
    pub fn as_table(&self) -> Option<&CaoLangTable> {
        match &self.body {
            CaoLangObjectBody::Table(v) => Some(v),
        }
    }

    pub fn as_table_mut(&mut self) -> Option<&mut CaoLangTable> {
        match &mut self.body {
            CaoLangObjectBody::Table(v) => Some(v),
        }
    }

    pub fn len(&self) -> usize {
        match &self.body {
            CaoLangObjectBody::Table(t) => t.len(),
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
        }
    }
}
