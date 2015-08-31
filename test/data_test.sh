#!/bin/bash

set -e

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
ALGO=$( cd "$DIR/../target/debug" && pwd)/algo
ALGOUSER=${ALGOUSER:-.my}
DATADIR=$ALGOUSER/test

function die {
    echo $1; exit 1
}

function test_algo {
    set -e
    echo
    echo ----- algo $@
    $ALGO $@
}

test_algo mkdir $DATADIR
touch $DIR/sample
test_algo upload $DATADIR $DIR/sample
rm $DIR/sample
test_algo ls $DATADIR
test_algo rmdir -f $DATADIR

