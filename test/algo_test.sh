#!/bin/bash

set -e

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ALGO=$( cd "$DIR/../target/debug" && pwd)/algo

function die {
    echo $1; exit 1
}

function test_algo {
    set -e
    echo
    echo ----- algo $@
    $ALGO $@
}


test_algo run "kenny/Factor" -d 72

echo '[{"a": {"b":1}}, "a", "b"]' > $DIR/graph.json
test_algo run "anowell/dijkstra" -J $DIR/graph.json
rm $DIR/graph.json
