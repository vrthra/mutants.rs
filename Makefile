nmutants=10000
programlen=100000
nfaults=1
nchecks=1
ntests=1000
DIR=data

release:
	cargo build --release

debug:
	cargo build

plot:
	find $(DIR) -name \*kills.csv | while read a; do ./plot.py $$a & done

all-tests:
	for f in 1 10 100 1000 ; do for c in 1 10 100 1000 ; do $(MAKE) run-tests nfaults=$$f nchecks=$$c & done ; done

run-tests:
	for k in 100 1000 10000 100000 1000000; do $(MAKE) run ntests=$$k & done

run-checks:
	for k in 1 10 100 1000 ; do $(MAKE) run nchecks=$$c & done

run-faults:
	for f in 1 10 100 1000 ; do $(MAKE) run nfaults=$$f & done

run:
	./target/release/mutants \
	   --nmutants=$(nmutants) \
	   --programlen=$(programlen) \
	   --nfaults=$(nfaults) \
	   --ntests=$(ntests) \
	   --nchecks=$(nchecks)

