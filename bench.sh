#!/bin/bash

run() {
    find "maps/$1/" -name '*.map' -exec target/release/examples/scen -a "$2" {} \; 2>/dev/null | awk "{total += \$2}END{print \"$2\", total}"
}

cargo build --release --example scen
run bitgrid jps
run bitgrid astar
run bitgrid dijkstra
