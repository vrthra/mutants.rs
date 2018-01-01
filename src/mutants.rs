extern crate getopts;
extern crate num_bigint;
extern crate num_traits;
extern crate rand;

use std::process;
use num_bigint::BigUint;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use num_traits::{FromPrimitive, One, Zero};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::repeat;
use getopts::Options;
use std::env;

use std::collections::HashMap;

#[derive(Debug)]
struct MyOptions {
    programlen: u64,
    nmutants: u64,
    ntests: u64,
    nfaults: u64,
    nchecks: u64,
    nequivalents: u64,
    subtle: u64,
}

impl ToString for MyOptions {
    fn to_string(&self) -> String {
        return format!("data/nfaults={:?}_ntests={:?}_nchecks={:?}_",
                         self.nfaults, self.ntests, self.nchecks);
    }
}

fn genbits(bitlen: u64, nflipped: u64) -> BigUint {
    let mut rng = rand::thread_rng();

    let faulty_bits: u64 = rng.gen_range(1, nflipped + 1);
    let rr = Range::new(0, bitlen as usize);

    (0..faulty_bits).fold(BigUint::zero(), |m, _|{
        let pos = rr.ind_sample(&mut rng);
        let fault = BigUint::one() << pos;
        m | fault
    })
}

fn gen_lst(num: u64, len: u64, nflipped: u64) -> Vec<BigUint> {
    (0..num).map(|_| genbits(len, nflipped)).collect()
}

fn gen_mutants(nmutants: u64, programlen: u64, nfaults: u64) -> Vec<BigUint> {
    gen_lst(nmutants, programlen, nfaults)
}

fn gen_tests(ntests: u64, programlen: u64, nchecks: u64) -> Vec<BigUint> {
    gen_lst(ntests, programlen, nchecks)
}

#[allow(dead_code)]
fn bitlog(bignum: &BigUint) -> () {
    eprintln!("num:{} bits: {} size: {}", bignum,  bignum.to_str_radix(2), bignum.bits());
}

fn hamming_wt(bignum: &BigUint) -> usize {
    let mut bit_count = 0;
    let mut i = BigUint::one();
    for _ in 0 .. bignum.bits() {
        if (&i & bignum) != BigUint::zero() {
            bit_count += 1;
        }
        i <<= 1 as usize;
    }
    assert!(i > *bignum);
    bit_count
}

fn kills(test: &BigUint, mutant: &BigUint, subtle: &u64) -> bool {
    //! If subtle == 0, we interpret the bits flipped as conditions to
    //! be satisfied. That is, all bits need to be anded.
    //! If subtle == 1, then it is same as checking if any of the bits
    //! in a mutant is detected.
    //! If subtle > 1, then it is same as adding a little bit of stubbornness
    //! to each mutant in that some of the bits are interpreted as conditions
    if *subtle == 0 {
        &(test & mutant) == mutant
    } else {
        hamming_wt(&(test & mutant)) >= FromPrimitive::from_u64(*subtle).unwrap()
    }
}

fn zeros(size: usize) -> Vec<BigUint> {
    repeat(BigUint::zero()).take(size).collect()
}

fn ntests_mutant_killed_by(m: &BigUint, tests: &[BigUint], subtle: &u64) -> usize {
    tests.iter().filter(|t| kills(t, m, subtle)).count()
}

fn mutant_killedby_ntests(
    opts: &MyOptions,
    mutants: &[BigUint],
    equivalents: &[BigUint],
    my_tests: &[BigUint],
) -> HashMap<usize, usize> {
    mutants.iter().chain(equivalents.iter())
        .map(|m| ntests_mutant_killed_by(m, my_tests, &opts.subtle))
        .enumerate().collect()
}

fn save_csv(opts: &MyOptions, mutant_kills: &HashMap<usize, usize>) -> () {
    let mut ntests = Vec::new();
    let max_tests_a_mutant_killed_by = mutant_kills.iter().map(|(_m, k)| k).max().unwrap();

    let mname = format!("{:}mutants.csv", opts.to_string());
    let mut f = File::create(&mname).expect(&format!("Unable to create file: {}", &mname));

    f.write_all(b"mutant,killedbynt\n").expect("Unable to write data");
    for (m, killedby_n) in mutant_kills {
        let data = format!("{},{}\n", m, killedby_n);
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }

    for nkillingt in 0..(*max_tests_a_mutant_killed_by + 1) {
        let mut exactlynt = 0;
        let mut atmostnt = 0;
        let mut atleastnt = 0;
        for (_m, killedby_n) in mutant_kills {
            if *killedby_n == nkillingt {
                exactlynt += 1;
            }
            if *killedby_n >= nkillingt {
                atleastnt += 1;
            }
            if *killedby_n <= nkillingt {
                atmostnt += 1;
            }
        }
        ntests.push((nkillingt, atleastnt, atmostnt, exactlynt))
    }
    let fname = format!("{:}kills.csv", opts.to_string());
    let mut f = File::create(&fname).expect(&format!("Unable to create file: {}", &fname));

    f.write_all(b"ntests,atleast,atmost,exactly\n")
        .expect("Unable to write data");
    for &(i, a, s, e) in &ntests {
        let data = format!("{},{},{},{}\n", i, a, s, e);
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }
}

fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn parse_arguments() -> MyOptions {
    let args: Vec<_> = env::args().collect();
    let program = &args[0];
    let mut opts = Options::new();
    opts.optflag("h", "help", "print help");
    opts.optopt("l", "programlen", "length of a mutant", "programlen");
    opts.optopt("m", "nmutants", "number of mutants", "nmutants");
    opts.optopt("t", "ntests", "number of tests", "ntests");
    opts.optopt("f", "nfaults", "maximum number of faults per mutant", "nfaults");
    opts.optopt("c", "nchecks", "maximum number of checks per test", "nchecks");
    opts.optopt("e", "nequivalents", "number of equivalents", "nequivalents");
    opts.optopt("s", "subtle", "subtlety of mutants (how many conditions need to be fulfilled?) -- 0 for conditions, 1 for *any* and >1 for hamming weight", "subtle");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(program, &opts);
        process::exit(0);
    };

    let numeric_arg = |name, def| matches.opt_str(name).map_or(def, |s| s.parse().unwrap());

    MyOptions {
        nmutants: numeric_arg("m", 10_1000),
        programlen: numeric_arg("l", 10_1000),
        ntests: numeric_arg("t", 10_1000),
        nfaults: numeric_arg("f", 10),
        nchecks: numeric_arg("c", 10),
        nequivalents: numeric_arg("c", 0),
        subtle: numeric_arg("s", 1),
    }
}

fn main() {
    let opts = parse_arguments();
    eprintln!("{:?}", opts);

    fs::create_dir_all("./data/").unwrap_or_else(|why| {
        panic!("! {:?}", why.kind());
    });

    // first generate our tests
    let my_tests = gen_tests(opts.ntests, opts.programlen, opts.nchecks);
    // Now generate n mutants
    let mutants = gen_mutants(opts.nmutants, opts.programlen, opts.nfaults);

    let equivalents = zeros(opts.nequivalents as usize);

    // how many tests killed this mutant?
    let mutant_kills = mutant_killedby_ntests(&opts, &mutants, &equivalents, &my_tests);

    save_csv(&opts, &mutant_kills);
}
