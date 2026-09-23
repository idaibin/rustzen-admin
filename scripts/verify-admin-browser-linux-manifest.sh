#!/usr/bin/env bash

verify_manifest_screenshots() {
  candidate=$1
  manifest="$candidate/manifest.json"
  jq -e '
    [ .faultCases[].artifact, (.successCases[] | select(.artifact != null) | .artifact) ] as $artifacts |
    ($artifacts | length) > 0 and
    ($artifacts | all(.file | type == "string" and test("^[A-Za-z0-9][A-Za-z0-9_-]*\\.png$"))) and
    ($artifacts | all(.sha256 | type == "string" and test("^[0-9a-f]{64}$"))) and
    ($artifacts | all(.dimensions | type == "string" and test("^[1-9][0-9]* x [1-9][0-9]*$"))) and
    (($artifacts | map(.file) | unique | length) == ($artifacts | length))
  ' "$manifest" >/dev/null
  failed=0
  while IFS=$'\t' read -r file expected_sha expected_dimensions; do
    actual_sha=$(sha256sum "$candidate/$file" | awk '{print $1}')
    actual_dimensions=$(file "$candidate/$file" | sed -E 's/.*PNG image data, ([0-9]+ x [0-9]+).*/\1/')
    if [ "$actual_sha" != "$expected_sha" ] || [ "$actual_dimensions" != "$expected_dimensions" ]; then
      failed=1
    fi
  done < <(jq -r '[.faultCases[].artifact, (.successCases[] | select(.artifact != null) | .artifact)][] | [.file,.sha256,.dimensions] | @tsv' "$manifest")
  test "$failed" -eq 0
}
