macro_rules! terminated {
    ($iter:expr, $pat:pat) => {
        $iter
            .take_while(|item| !matches!(item, $pat))
            .filter(|item| !matches!(item, $pat))
    };
}

macro_rules! terminated_for {
    (for $ident:ident in terminated!($expr:expr, $pat:pat) $block:block) => {
        while let Some($ident) = $expr.next() {
            if let $pat = $ident {
                break;
            }
            $block
        }
    };
}

macro_rules! reachable {
    (let $pat:pat = $expr:ident $block:block) => {
        if let $pat = $expr $block
        else {
            unreachable!()
        }
    }
}
