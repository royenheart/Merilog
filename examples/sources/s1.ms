//! This module is used to be a example code
//! * Note that it is ...

/*
    Block Comment 
    * No use
 */

struct a {
    b: Type3,
    c: Type4
}

fn main(param1: Type1, param2: Type2) -> Identity_Type {
    let mut _a: Type3 = 3;
    let mut b3 = 4.3;
    let cons = true;
    let s = "Nihao";
    // line comment
    if _a > b3 {
        _a = 4;
        print("");
    } else if (_a < b3) {
        print("Hello");
    } else if (_a == b3) {
        print("cd");
    }

    while _a > 3 || cons == false {
        a = a - 1;
    }

    {
        b3 = _a - 1;
        if _a > 3 {
            break;
        }
    } loop b3 <= 3

    match cons {
        true: {
            return;
        },
        false: {
            return;
        }
    }

    return 3;
}
