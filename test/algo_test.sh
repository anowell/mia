#!/bin/bash

set -e

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
MIA=$( cd "$DIR/../target/debug" && pwd)/mia

function die {
    echo $1; exit 1
}

function test_mia {
    set -e
    echo
    echo ----- mia $@
    $MIA $@
}


test_mia run "kenny/Factor" -d 72

echo '[{"a": {"b":1}}, "a", "b"]' > $DIR/graph.json
test_mia run "anowell/dijkstra" -J $DIR/graph.json
rm $DIR/graph.json
