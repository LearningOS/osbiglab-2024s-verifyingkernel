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

struct AbstractFibState {
    n: nat,
    value: nat,
}

spec fn abstract_fib_init(s: AbstractFibState) -> bool {
    s.n == 0 && s.value == 1
}

spec fn abstract_fib_step(s1: AbstractFibState, s2: AbstractFibState) -> bool {
    &&& s1.n + 1 == s2.n
    &&& s1.value == fib(s1.n)
    &&& s2.value == fib(s2.n)
}

struct ConcreteFibState {
    n: nat,
    a: nat,
    b: nat,
}

impl ConcreteFibState {
    spec fn inv(self) -> bool {
        self.a == fib(self.n) && self.b == fib(self.n + 1)
    }

    spec fn interp(self) -> AbstractFibState {
        AbstractFibState { n: self.n, value: self.a }
    }
}

spec fn concrete_fib_init(s: ConcreteFibState) -> bool {
    s.n == 0 && s.a == 1 && s.b == 1
}

spec fn concrete_fib_step(s1: ConcreteFibState, s2: ConcreteFibState) -> bool {
    &&& s2.n == s1.n + 1
    &&& s2.a == s1.b
    &&& s2.b == s1.a + s1.b
}

proof fn concrete_init_implies_inv(s: ConcreteFibState)
    requires
        concrete_fib_init(s),
    ensures
        s.inv(),
{
}

proof fn concrete_step_preserves_inv(s1: ConcreteFibState, s2: ConcreteFibState)
    requires
        s1.inv(),
        concrete_fib_step(s1, s2),
    ensures
        s2.inv(),
{
}

proof fn concrete_fib_init_refines_abstract_fib_init(s: ConcreteFibState)
    requires
        concrete_fib_init(s),
    ensures
        abstract_fib_init(s.interp()),
{
}

proof fn concrete_fib_step_refines_abstract_fib_step(s1: ConcreteFibState, s2: ConcreteFibState)
    requires
        s1.inv(),
        concrete_fib_step(s1, s2),
    ensures
        abstract_fib_step(s1.interp(), s2.interp()),
{
}

trait FibInterface: Sized {
    spec fn interp(self) -> ConcreteFibState;

    spec fn inv(self) -> bool;

    fn new() -> (s: Self)
        ensures
            s.inv(),
            concrete_fib_init(s.interp());

    spec fn step_enabled(&self) -> bool;

    fn step(&mut self)
        requires
            old(self).inv(),
            old(self).step_enabled(),
        ensures
            self.inv(),
            concrete_fib_step(old(self).interp(), self.interp());
}

const FIB_U64_MAX_N: u64 = 92;

#[verifier::spinoff_prover]
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
    assert(fib(FIB_U64_MAX_N as nat + 1) > u64::MAX);
}

proof fn lemma_fib_mono(x: nat, y: nat)
    requires
        x <= y,
    ensures
        fib(x) <= fib(y),
    decreases y,
{
    if x < y {
        reveal_with_fuel(fib, 2);
        lemma_fib_mono(x, (y - 1) as nat);
    }
}

struct FibImpl {
    n: Ghost<nat>,
    a: u64,
    b: u64,
}

impl FibInterface for FibImpl {
    spec fn interp(self) -> ConcreteFibState {
        ConcreteFibState {
            n: self.n@,
            a: self.a as nat,
            b: self.b as nat,
        }
    }

    spec fn inv(self) -> bool {
        self.interp().inv()
    }

    fn new() -> (s: Self) {
        Self {
            n: Ghost(0),
            a: 1,
            b: 1,
        }
    }

    spec fn step_enabled(&self) -> bool {
        self.n@ + 2 <= FIB_U64_MAX_N
    }

    fn step(&mut self) {
        self.n = Ghost(self.n@ + 1);
        proof {
            lemma_fib_maxn_fits_u64();
            lemma_fib_mono(self.n@ + 1, FIB_U64_MAX_N as nat);
        }
        let t = self.a + self.b;
        self.a = self.b;
        self.b = t;
    }
}

spec fn is_fib_seq(s: Seq<u64>) -> bool {
    forall |n: nat| n < s.len() ==> #[trigger] s[n as int] == fib(n)
}

fn fib_seq() -> (result: Vec<u64>)
    ensures
        result.len() == FIB_U64_MAX_N + 1,
        is_fib_seq(result@),
{
    let mut f = FibImpl::new();
    let mut result = Vec::new();

    loop
        invariant_except_break
            result.len() == f.n@ <= FIB_U64_MAX_N - 1,
        invariant
            f.inv(),
            is_fib_seq(result@),
        ensures
            result.len() == FIB_U64_MAX_N + 1,
    {
        if result.len() as u64 == FIB_U64_MAX_N - 1 {
            result.push(f.a);
            result.push(f.b);
            break;
        }
        result.push(f.a);
        f.step();
    }

    result
}

} // verus!

fn main() {
    let result = fib_seq();
    for (i, x) in result.iter().enumerate() {
        println!("{i}: {x}");
    }
}
