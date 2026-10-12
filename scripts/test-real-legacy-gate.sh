#!/usr/bin/env bash
# OE-LEGACY-SELECT-01 v2 gate: the only supported runner for the real
# legacy admission test (DOT-OE-1900-03). The test is #[ignore]d, so
# ordinary CI cargo test invocations never execute it.
#
# v2 hardening after independent review (P1): the gate accepts NO
# arguments at all — the target below is the single canonical
# constant, so a caller can never substitute another ignored test
# (for example the route-contract benchmark) or smuggle a
# leading-dash option into the cargo argv. Counting uses exact-line
# awk matching with always-numeric results (no grep option parsing,
# no error-swallowing || true, no empty-integer comparisons), and
# cargo's own non-zero exit code is propagated verbatim for both the
# --list and the execution call: counting can only turn a
# cargo-success with zero, duplicate, or wrong targets into a gate
# failure — it can never mask cargo's original code.
set -euo pipefail

readonly CANONICAL='features::manage::deploy::bundle::tests::accepts_the_real_legacy_signed_artifact'

if [ "$#" -ne 0 ]; then
    printf 'test-real-legacy: refusing %d argument(s); the gate target is fixed and no arguments are accepted\n' "$#" >&2
    exit 2
fi

# Number of --list lines that are exactly "<canonical>: test".
count_listed_canonical() {
    awk -v n="$CANONICAL" '$0 == n ": test" { c++ } END { print c + 0 }'
}

# Number of --list lines naming any test at all ("NAME: test").
count_listed_total() {
    awk '$0 ~ /: test$/ { c++ } END { print c + 0 }'
}

# Number of execution result lines for the canonical target.
count_ran_canonical() {
    awk -v n="$CANONICAL" 'index($0, "test " n " ... ") == 1 { c++ } END { print c + 0 }'
}

# Number of execution result lines for any test ("test NAME ... ...").
count_ran_total() {
    awk '$0 ~ /^test .+ \.\.\. / { c++ } END { print c + 0 }'
}

set +e
listed=$(cargo test -p rustzen-admin --bin rz-admin --features modules "$CANONICAL" -- --ignored --exact --list 2>&1)
list_rc=$?
set -e
printf '%s\n' "$listed"
if [ "$list_rc" -ne 0 ]; then
    printf 'test-real-legacy: cargo --list exited non-zero (%d); preserving it\n' "$list_rc" >&2
    exit "$list_rc"
fi
total=$(printf '%s\n' "$listed" | count_listed_total)
canonical=$(printf '%s\n' "$listed" | count_listed_canonical)
if [ "$total" -ne 1 ] || [ "$canonical" -ne 1 ]; then
    printf 'test-real-legacy: selection gate failed: expected exactly the one canonical target, listed %s test line(s) of which %s canonical; refusing to execute\n' "$total" "$canonical" >&2
    exit 1
fi

set +e
out=$(cargo test -p rustzen-admin --bin rz-admin --features modules "$CANONICAL" -- --ignored --exact --test-threads=1 2>&1)
run_rc=$?
set -e
printf '%s\n' "$out"
if [ "$run_rc" -ne 0 ]; then
    printf 'test-real-legacy: cargo execution exited non-zero (%d); preserving it\n' "$run_rc" >&2
    exit "$run_rc"
fi
ran_total=$(printf '%s\n' "$out" | count_ran_total)
ran_canonical=$(printf '%s\n' "$out" | count_ran_canonical)
if [ "$ran_total" -ne 1 ] || [ "$ran_canonical" -ne 1 ]; then
    printf 'test-real-legacy: execution gate failed: expected exactly the one canonical target to run, saw %s result line(s) of which %s canonical; treating as gate failure, not a pass\n' "$ran_total" "$ran_canonical" >&2
    exit 1
fi
exit "$run_rc"
