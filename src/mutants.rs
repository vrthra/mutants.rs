extern crate getopts;
extern crate num_bigint;
extern crate num_traits;
extern crate rand;

use num_bigint::BigUint;
use rand::distributions::{IndependentSample, Range};
use num_traits::FromPrimitive;
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
}

impl ToString for MyOptions {
    fn to_string(&self) -> String {
        return format!("data/programlen={:?}_nequivalents={:?}_nmutants={:?}/ntests={:?}_nfaults={:?}_nchecks={:?}/",
                         self.programlen, self.nequivalents, self.nmutants, self.ntests,
                         self.nfaults, self.nchecks);
    }
}

fn genbits(bitlen: u64, nflipped: u64) -> BigUint {
    let mut rng = rand::thread_rng();
    let faulty_bits: u64 = Range::new(1, nflipped + 1).ind_sample(&mut rng);
    let mut m: BigUint = FromPrimitive::from_usize(0).unwrap();
    for _ in 0..faulty_bits {
        let pos: usize = Range::new(0, bitlen).ind_sample(&mut rng) as usize;
        let one: BigUint = FromPrimitive::from_usize(1).unwrap();
        let fault = one << pos;
        m |= fault;
    }
    return m;
}

fn gen_lst(num: u64, len: u64, nflipped: u64) -> Vec<BigUint> {
    return (0..num).map(|_| genbits(len, nflipped)).collect(); //::<Vec<_>>
}

fn gen_mutants(nmutants: u64, programlen: u64, nfaults: u64) -> Vec<BigUint> {
    return gen_lst(nmutants, programlen, nfaults);
}

fn gen_tests(ntests: u64, programlen: u64, nchecks: u64) -> Vec<BigUint> {
    return gen_lst(ntests, programlen, nchecks);
}

fn kills(test: &BigUint, mutant: &BigUint) -> bool {
    return (test & mutant) > FromPrimitive::from_usize(0).unwrap();
}

fn zeros(size: usize) -> Vec<BigUint> {
    repeat(FromPrimitive::from_usize(0).unwrap())
        .take(size)
        .collect()
}

fn ntests_mutant_killed_by(m: &BigUint, tests: &Vec<BigUint>) -> usize {
    return tests.iter().filter(|t| kills(&t, m)).count();
}

fn mutant_killedby_ntests(
    _opts: &MyOptions,
    mutants: &Vec<BigUint>,
    equivalents: &Vec<BigUint>,
    my_tests: &Vec<BigUint>,
) -> HashMap<usize, usize> {
    return mutants.iter().chain(equivalents.iter())
        .map(|m| ntests_mutant_killed_by(m, my_tests))
        .enumerate().collect();
}

fn do_statistics(opts: &MyOptions, mutant_kills: &HashMap<usize, usize>) -> () {
    let mut ntests = Vec::new();
    let max = mutant_kills.iter().map(|(_m, k)| k).max().unwrap();
    for i in 0..*max {
        let mut e = 0;
        let mut a = 0;
        let mut s = 0;
        for (_m, k) in mutant_kills {
            if *k == i {
                e += 1;
            }
            if *k >= i {
                a += 1;
            }
            if *k <= i {
                s += 1;
            }
        }
        ntests.push((i, a, s, e))
    }
    let fname = format!("{:}kills.csv", opts.to_string());
    let mut f = File::create(&fname).expect(&format!("Unable to create file: {}", &fname));

    f.write_all("ntests, atleast, atmost, exactly\n".as_bytes())
        .expect("Unable to write data");
    for &(i, a, s, e) in &ntests {
        let data = format!("{}, {}, {}, {}\n", i, a, s, e);
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }
}

fn main() {
    let args: Vec<String> = env::args().map(|x| x.to_string()).collect();

    let ref _program = args[0];
    let mut opts = Options::new();
    opts.optopt("l", "programlen", "length of a mutant", "programlen");
    opts.optopt("m", "nmutants", "number of mutants", "nmutants");
    opts.optopt("t", "ntests", "number of tests", "ntests");
    opts.optopt("f", "nfaults", "maximum number of faults per mutant", "nfaults");
    opts.optopt("c", "nchecks", "maximum number of checks per test", "nchecks");
    opts.optopt("e", "nequivalents", "number of equivalents", "nequivalents");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let programlen = match matches.opt_str("l") {
        Some(s) => s.parse().unwrap(),
        None => 10000,
    };
    let nmutants = match matches.opt_str("m") {
        Some(s) => s.parse().unwrap(),
        None => 10000,
    };
    let ntests = match matches.opt_str("t") {
        Some(s) => s.parse().unwrap(),
        None => 10000,
    };
    let nfaults = match matches.opt_str("f") {
        Some(s) => s.parse().unwrap(),
        None => 10,
    };
    let nchecks = match matches.opt_str("c") {
        Some(s) => s.parse().unwrap(),
        None => 10,
    };
    let nequivalents = match matches.opt_str("e") {
        Some(s) => s.parse().unwrap(),
        None => 0,
    };

    let opts: MyOptions = MyOptions {
        nmutants,
        programlen,
        nfaults,
        ntests,
        nchecks,
        nequivalents,
    };
    eprintln!("{:?}", opts);

    fs::create_dir_all(opts.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    // first generate our tests
    let my_tests = gen_tests(ntests, programlen, nchecks);
    // Now generate n mutants
    let mutants = gen_mutants(nmutants, programlen, nfaults);

    let equivalents = zeros(nequivalents as usize);

    // how many tests killed this mutant?
    let mutant_kills = mutant_killedby_ntests(&opts, &mutants, &equivalents, &my_tests);

    do_statistics(&opts, &mutant_kills);
}
