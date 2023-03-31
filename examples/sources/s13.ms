struct c {
    a: type1,
    b: type2,
    fn dddd(params: type1, params: type2) -> type3 {
        return 3;
    }
}

fn main() {
    /*
    let c = 1 + match true {
        false: {
            return false;
        },
        e: {
            return true;
        }
    };
    */

    let mut c = 3 + match true {
        true: {
            return 4;
        },
        e->d[10]: {
            return 5;
        },
        _: {
            return 10;
        }
    };
}