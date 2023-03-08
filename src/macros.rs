#[macro_export]
macro_rules! check_max_depth {
    ($v: expr, $max_depth: expr, $default_max_depth: expr, $block: stmt) => {
        if let Some(max_depth) = $max_depth {
            if $v >= max_depth {
                $block
            }
        } else {
            if $v >= $default_max_depth {
                panic!("max depth exceeded: {}", $default_max_depth)
            }
        }
    };
}
