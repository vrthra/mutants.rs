extern crate num_bigint;
extern crate rand;
extern crate num_traits;
extern crate getopts;

use num_bigint::{BigUint};
use rand::distributions::{IndependentSample, Range};
use num_traits::{FromPrimitive};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::iter::repeat;
use getopts::Options;
use std::env;

use std::collections::HashMap;

#[derive(Debug)]
struct MyOptions {
    mutantlen: u64,
    nmutants: u64,
    ntests: u64,
    nfaults: u64,
    nchecks: u64,
    nequivalents: u64,
}

impl ToString for MyOptions {
   fn to_string(&self) -> String {
      return format!("data/mutantlen={:?}/nequivalents={:?}/nmutants={:?}/nfaults={:?}/ntests={:?}/nchecks={:?}/",
                         self.mutantlen, self.nequivalents, self.nmutants, self.nfaults, self.ntests, self.nchecks);
   }
}

fn genbits(bitlen: u64, nflipped:u64) -> BigUint {
    let mut rng = rand::thread_rng();
    let faulty_bits: u64 = Range::new(1, nflipped + 1).ind_sample(&mut rng);
    let mut m: BigUint = FromPrimitive::from_usize(0).unwrap();
    for _ in 0 .. faulty_bits {
        let pos : u64 = Range::new(0, bitlen).ind_sample(&mut rng);
        let one: BigUint = FromPrimitive::from_usize(1).unwrap();
        let fault = one << pos as usize;
        m |= fault;
    }
    return m;
}

fn gen_lst(num: u64, len: u64, nflipped: u64) -> Vec<BigUint> {
   let mut vec = Vec::new();
   for _i in 0 .. num {
     vec.push(genbits(len, nflipped));
   }
   return vec;
}

fn gen_mutants(nmutants: u64, mutantlen: u64, nfaults: u64) -> Vec<BigUint>  {
    return gen_lst(nmutants, mutantlen, nfaults);
}

fn gen_tests(ntests: u64, mutantlen: u64, nchecks: u64) -> Vec<BigUint>  {
    return gen_lst(ntests, mutantlen, nchecks);
}

fn kills(test: &BigUint, mutant: &BigUint) -> bool {
    return (test & mutant) > FromPrimitive::from_usize(0).unwrap();
}

fn zeros(size: usize) -> Vec<BigUint> {
    repeat(FromPrimitive::from_usize(0).unwrap()).take(size).collect()
}

fn mutant_killed_by(m: &BigUint, tests: &Vec<BigUint>) -> u64 {
    let mut v = 0;
    for t in tests {
        if kills(&t, m) {
            v += 1;
        }
    }
    return v;
}

fn mutant_killscore(_opts: &MyOptions, mutants: &Vec<BigUint>, equivalents: &Vec<BigUint>, my_tests: &Vec<BigUint>) -> HashMap<u64, u64>{
    let mut mutant_kills = HashMap::new();
    let mut idx = 0;
    for m in mutants {
       mutant_kills.insert(idx, mutant_killed_by(&m, my_tests));
       idx += 1;
    }
    for m in equivalents {
       mutant_kills.insert(idx, mutant_killed_by(&m, my_tests));
       idx += 1;
    }
    return mutant_kills;
}

fn do_statistics(opts: &MyOptions, mutant_kills: &HashMap<u64, u64>) -> () {
    let mut ntests = Vec::new();
    for i in 0..1001 {
        let mut e = 0;
        let mut a = 0;
        let mut s = 0;
        for (_m, k) in mutant_kills {
            if *k == i { e += 1; }
            if *k >= i { a += 1; }
            if *k <= i { s += 1; }
        }
        ntests.push((i,a,s,e))
    }
    let fname = format!("{:}kills.csv", opts.to_string());
    let mut f = File::create(&fname).expect(&format!("Unable to create file: {}", &fname));

	f.write_all("ntests, atleast, atmost, exactly\n".as_bytes()).expect("Unable to write data");
    for &(i, a, s, e) in &ntests {
        let data = format!("{}, {}, {}, {}\n", i, a, s, e);
        f.write_all(data.as_bytes()).expect("Unable to write data");
    }
}

fn main() {

    let args: Vec<String> = env::args().map(|x| x.to_string()).collect();

    let ref _program = args[0];
    let mut opts = Options::new();
    opts.optopt("l", "mutantlen", "length of a mutant", "mutantlen");
    opts.optopt("m", "nmutants", "number of mutants", "nmutants");
    opts.optopt("t", "ntests", "number of tests", "ntests");
    opts.optopt("f", "nfaults", "maximum number of faults per mutant", "nfaults");
    opts.optopt("c", "nchecks", "maximum number of checks per test", "nchecks");
    opts.optopt("e", "nequivalents", "number of equivalents", "nequivalents");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => { panic!(f.to_string()) }
    };

    let mutantlen = match matches.opt_str("l") {
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

    let opts: MyOptions = MyOptions {nmutants, mutantlen, nfaults, ntests, nchecks, nequivalents};
    eprintln!("{:?}", opts);

    fs::create_dir_all(opts.to_string()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });

    // first generate our tests
    let my_tests = gen_tests(ntests, mutantlen, nchecks);
    // Now generate n mutants
    let mutants = gen_mutants(nmutants, mutantlen, nfaults);

    let equivalents = zeros(nequivalents as usize);

    // how many tests killed this mutant?
    let mutant_kills = mutant_killscore(&opts, &mutants, &equivalents, &my_tests);

    do_statistics(&opts, &mutant_kills);
}
