#!/bin/bash

set -e

DIR=$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )
MIA=$( cd "$DIR/../target/debug" && pwd)/mia
ALGOUSER=${ALGOUSER:-.my}
DATADIR=data://$ALGOUSER/test

function die {
    echo $1; exit 1
}

function test_mia {
    set -e
    echo
    echo ----- mia $@
    $MIA $@
}

test_mia mkdir $DATADIR
touch $DIR/sample
test_mia cp $DIR/sample $DATADIR
rm $DIR/sample
test_mia ls $DATADIR
test_mia rmdir -f $DATADIR

