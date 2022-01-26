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

                let res = a $cmp b;
                $from.runtime_data.stack.push(Value::Integer(res as i64))
                    .map_err(|_|ExecutionError::Stackoverflow)?;
            }
        };
    }
