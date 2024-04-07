use vstd::prelude::*;

verus! {

spec fn fib(n: nat) -> (f: nat)
    decreases n,
{
    if n <= 1 {
        1
    } else {
        fib((n - 2) as nat) + fib((n - 1) as nat)
    }
}

proof fn fib_mono(x: nat, y: nat)
    requires
        x <= y,
    ensures
        fib(x) <= fib(y),
    decreases y,
{
    if x < y {
        reveal_with_fuel(fib, 2);
        fib_mono(x, (y - 1) as nat);
    }
}

const FIB_U64_MAX_N: u64 = 92;

proof fn lemma_fib_maxn_fits_u64()
    ensures
        fib(FIB_U64_MAX_N as nat) <= u64::MAX,
{
    reveal_with_fuel(fib, 4);
    assert(fib(5) < fib(10));
    assert(fib(15) < fib(20));
    assert(fib(25) < fib(30));
    assert(fib(35) < fib(40));
    assert(fib(45) < fib(50));
    assert(fib(55) < fib(60));
    assert(fib(65) < fib(70));
    assert(fib(75) < fib(80));
    assert(fib(85) < fib(90));

    // shorter but slower:
    // reveal_with_fuel(fib, 11);
    // assert(fib(20) < fib(40));
    // assert(fib(60) < fib(80));

    assert(fib(FIB_U64_MAX_N as nat + 1) > u64::MAX);
}

fn calc_fib(n: u64) -> (f: u64)
    requires
        n <= FIB_U64_MAX_N,
    ensures
        f == fib(n as nat),
{
    if n <= 1 {
        return 1;
    }

    let mut a = 1;
    let mut b = 1;
    let mut index = 0;

    while index < n - 1
        invariant
            index < n <= FIB_U64_MAX_N,
            a == fib(index as nat),
            b == fib((index + 1) as nat),
    {
        assert((a + b) as nat <= u64::MAX) by {
            lemma_fib_maxn_fits_u64();
            fib_mono((index + 2) as nat, FIB_U64_MAX_N as nat);
        }
        let t = a + b;
        a = b;
        b = t;
        index += 1;
    }

    b
}

fn test() {
    for i in 0..93 {
        calc_fib(i);
    }
}

} // verus!

fn main() {
    for i in 0..=FIB_U64_MAX_N {
        println!("{i}: {}", calc_fib(i));
    }
}
