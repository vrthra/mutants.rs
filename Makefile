nmutants=10000
programlen=100000
nfaults=1
nchecks=1
ntests=1

release:
	cargo build --release

debug:
	cargo build

run:
	./target/release/mutants --nmutants=$(nmutants) \
		--programlen=$(programlen) --nfaults=$(nfaults) --ntests=$(ntests) --nchecks=$(nchecks)
