all: fib

%: src/%.rs
	verus --compile $<
	mv $@ target
