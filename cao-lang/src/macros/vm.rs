#[macro_export(local_inner_macros)]
macro_rules! load_ptr {
    ($val: expr, $from: ident) => {{
        use crate::collections::key_map::Key;
        let key = Key::from_u32($val.0);
        $from
            .objects
            .get(key)
            .ok_or(ExecutionError::InvalidArgument { context: None })?
    }};
}

#[macro_export(local_inner_macros)]
macro_rules! load_var {
    ($val: expr, $from: ident) => {
        $from
            .variables
            .get($val)
            .cloned()
            .ok_or(ExecutionError::InvalidArgument { context: None })?;
    };
}

#[macro_export(local_inner_macros)]
macro_rules! pop_stack {
    ($from : ident) => {{
        $from.runtime_data.stack.pop()
    }};
}

#[macro_export(local_inner_macros)]
macro_rules! binary_compare {
        ($from:ident, $cmp: tt, $return_on_diff_type: expr) => {
            {
                let b = pop_stack!($from);
                let a = pop_stack!($from);

                let res = match (a, b) {
                    (Scalar::Pointer(a), Scalar::Pointer(b)) => {
                        let a = load_ptr!(&a, $from);
                        let b = load_ptr!(&b, $from);
                        if a.size != b.size || a.index.is_none() || b.index.is_none() {
                            $return_on_diff_type
                        } else {
                            let size = a.size as usize;
                            let ind = a.index.unwrap().0 as usize;
                            let a = &$from.runtime_data.memory[ind..ind + size];
                            let ind = b.index.unwrap().0 as usize;
                            let b = &$from.runtime_data.memory[ind..ind + size];

                            a.iter().zip(b.iter()).all(|(a, b)| a $cmp b)
                        }
                    }
                    _ => a $cmp b,
                };
                $from.runtime_data.stack.push(Scalar::Integer(res as i32))
                    .map_err(|_|ExecutionError::Stackoverflow)?;
            }
        };
    }
