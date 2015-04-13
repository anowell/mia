#!/bin/bash

set -e

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ALGO=$( cd "$DIR/../target/debug" && pwd)/algo
ALGOUSER=${ALGOUSER:-$USERNAME}
COLLECTION=$ALGOUSER/test

function die {
    echo $1; exit 1
}

function test_algo {
    set -e
    echo
    echo ----- algo $@
    $ALGO $@
}

test_algo mkdir $COLLECTION
touch $DIR/sample
test_algo upload $COLLECTION sample
rm $DIR/sample
test_algo ls $COLLECTION
test_algo rmdir $COLLECTION

