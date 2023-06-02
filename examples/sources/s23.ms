fn main() -> i32 {
    let mut a = 8;
    let mut b = 4;

    while a < 12 {
        if a > 9 {
            break;
        } else {
            a += 1;
        }
    }

    // a = 10

    if a < 10 {
        return 11;
    } else if a > 7 {
        a *= 2;
    } else {
        return 11;
    }

    // a = 20

    if a < 10 {
        return 11;
    } else if a > 7 {
        a -= 10;
    } else if a < 12 {
        return 1133;
    }

    // a = 10

    if a < 10 {
        while a > 8 {
            a -= 1 - 2 + 3;
        }
    } else if b < 7 {
        {
            if b > 10 {
                a -= 1;
            } else {
                a -= 2;
            }
            b = b * 2;
        } loop b < 17;
    } else if a > 6 {
        while a < 10  {
            a += 1;
        }
        a /= 4;
    }

    // a = 5

    a /= 4;

    // a = 1
    // b = 32

    return a;
} 