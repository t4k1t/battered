alias cov := coverage

# run unit tests
@test:
    echo "Runnint unit tests"
    cargo test

# generate coverage report
@coverage REPORT_TYPE='Html':
    cargo tarpaulin --out {{REPORT_TYPE}}

# clean dist
@clean:
    echo "Cleaning up existing artifacts…"
    cargo clean

# build dist
@build: clean
    echo "Building dist"
    cargo build --release

# publish to crates.io
@publish:
    echo "Publishing to crates.io"
    cargo login
    cargo publish
