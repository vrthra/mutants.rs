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

#[allow(dead_code)]
fn bitlog(bignum: &BigUint) -> () {
    eprintln!("num:{} bits: {} size: {}", bignum,  bignum.to_str_radix(2), bignum.bits());
}

fn hamming_wt(bignum: &BigUint) -> usize {
    let mut bit_count = 0;
    let mut i: BigUint = FromPrimitive::from_usize(1).unwrap();
    let zero: BigUint = FromPrimitive::from_usize(0).unwrap();
    for _ in 0 .. bignum.bits() {
        if (&i & bignum) != zero {
            bit_count += 1;
        }
        i = i << 1 as usize;
    }
    assert!(i > *bignum);
    return bit_count;
}

fn kills(test: &BigUint, mutant: &BigUint, subtle: &u64) -> bool {
    //! If subtle == 0, we interpret the bits flipped as conditions to
    //! be satisfied. That is, all bits need to be anded.
    //! If subtle == 1, then it is same as checking if any of the bits
    //! in a mutant is detected.
    //! If subtle > 1, then it is same as adding a little bit of stubbornness
    //! to each mutant in that some of the bits are interpreted as conditions
    if *subtle == 0 {
        return &(test & mutant) == mutant;
    } else {
        return hamming_wt(&(test & mutant)) >= FromPrimitive::from_u64(*subtle).unwrap();
    }
}

fn zeros(size: usize) -> Vec<BigUint> {
    repeat(FromPrimitive::from_usize(0).unwrap())
        .take(size)
        .collect()
}

fn ntests_mutant_killed_by(m: &BigUint, tests: &Vec<BigUint>, subtle: &u64) -> usize {
    return tests.iter().filter(|t| kills(&t, m, &subtle)).count();
}

fn mutant_killedby_ntests(
    opts: &MyOptions,
    mutants: &Vec<BigUint>,
    equivalents: &Vec<BigUint>,
    my_tests: &Vec<BigUint>,
) -> HashMap<usize, usize> {
    return mutants.iter().chain(equivalents.iter())
        .map(|m| ntests_mutant_killed_by(m, my_tests, &opts.subtle))
        .enumerate().collect();
}

fn save_csv(opts: &MyOptions, mutant_kills: &HashMap<usize, usize>) -> () {
    let mut ntests = Vec::new();
    let max_tests_a_mutant_killed_by = mutant_kills.iter().map(|(_m, k)| k).max().unwrap();

    let mname = format!("{:}mutants.csv", opts.to_string());
    let mut f = File::create(&mname).expect(&format!("Unable to create file: {}", &mname));

    f.write_all("mutant,killedbynt\n".as_bytes()).expect("Unable to write data");
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

    f.write_all("ntests,atleast,atmost,exactly\n".as_bytes())
        .expect("Unable to write data");
    for &(i, a, s, e) in &ntests {
        let data = format!("{},{},{},{}\n", i, a, s, e);
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().map(|x| x.to_string()).collect();

    let ref program = args[0];
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
        print_usage(&program, opts);
        return;
    }
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
    let subtle = match matches.opt_str("s") {
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
        subtle,
    };
    eprintln!("{:?}", opts);

    fs::create_dir_all("./data/").unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    // first generate our tests
    let my_tests = gen_tests(ntests, programlen, nchecks);
    // Now generate n mutants
    let mutants = gen_mutants(nmutants, programlen, nfaults);

    let equivalents = zeros(nequivalents as usize);

    // how many tests killed this mutant?
    let mutant_kills = mutant_killedby_ntests(&opts, &mutants, &equivalents, &my_tests);

    save_csv(&opts, &mutant_kills);
}
