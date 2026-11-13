# Short entry points. See README.md for the full workflow.
CARGO ?= cargo
PY ?= uv run python
REASONER := reasoner/target/release/reasoner
JOB ?= jobs/demo

.PHONY: help build demo prior check clean

help:
	@echo "make build   build the Rust reasoner (release)"
	@echo "make demo    solve and grade the two toy tasks in examples/"
	@echo "make prior   rebuild reasoner/src/prior.txt from the committed grade files (E17)"
	@echo "make check   re-derive two published comparisons from the committed grade files"
	@echo "make clean   remove build output and job folders"

build:
	cd reasoner && $(CARGO) build --release

demo: build
	python3 tools/arc_job.py prepare examples/tasks examples/ids.txt $(JOB)/in
	ARC_JOB_IN=$(JOB)/in ARC_JOB_OUT=$(JOB) $(REASONER) job --cpu-budget 5 --threads 2
	python3 tools/arc_job.py grade examples/tasks examples/ids.txt $(JOB)/out $(JOB)/grade.json

prior:
	python3 tools/make_prior.py

check:
	cd experiments && python3 ../tools/compare_runs.py v1 e13-dsl-v1/grades/train.json e13-dsl-v1/grades/select.json \
		-- v0 e12-reasoner-v0/grades/train.json e12-reasoner-v0/grades/select.json
	cd experiments && python3 ../tools/compare_clustered.py hints-v2 e23-fair-pool/grades/p15d5v2.json \
		-- control e23-fair-pool/grades/control.json

clean:
	rm -rf reasoner/target jobs
