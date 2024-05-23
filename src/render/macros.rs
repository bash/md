#[macro_export]
macro_rules! take {
    (for $ident:ident in $expr:expr; until $pat:pat => $block:block) => {
        while let Some($ident) = $expr.next() {
            if let $pat = $ident {
                break;
            }
            $block
        }
    };
}
