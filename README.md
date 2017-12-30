# Mutation Analysis Simulation

```
$ ./target/release/mutants \
           --nmutants=$nmutants \
           --programlen=$programlen \
           --nfaults=$nfaults \
           --ntests=$ntests \
           --nchecks=$nchecks
```

# Options

* nmutants : The number of mutants to be generated
* programlen : The size of the original program (number of bits)
* nfaults : The number of faults to be introduced into the mutant (it is *upto* $nfaults per mutant -- minimum of 1).
* ntests : The number of tests to be generated.
* nchecks : The number of bits checked per test (it is *upto* $nchecks per mutant -- minimum of 1)

# Description

The original program is a string of bits all set to zero, and any mutant as a
variant of that string of bits with some bits flipped. That is, `*nfaults* = number of bits set to 1`
A test is also a string of bits with a few bits set to 1. That is, `*nchecks* = number bits that are 1`.
A test kills a mutant if `(test & mutant) != 0`.

# Syntactic neighborhood (The competent programmer hypothesis)

This is controlled by `--nfaults` which produces *upto* `$nfaults` differences. If the lexical distance is to be just a single fault, then `set $nfaults=1`.

# The coupling-effect

Any test detecting a mutant will also detect any `OR` combination of mutants.

In a real system, there are always mutants that are harder to detect than simple lexical faults. These may be simulated by making (some of) the bits set in a mutant the conditions *necessary* to detect it. That is, test kills a mutant if all conditions are satisfied instead of just one. That is, `(test & mutant) == mutant`.

# Coverage

A test can be said to cover a given range between its most significant bit set, and the least significant bit set. A program is completely covered if all its bits are covered by the test suite.
