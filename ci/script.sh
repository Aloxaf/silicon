# This script takes care of testing your crate

set -ex

main() {
    if [ $TARGET == "x86_64-unknown-linux-gnu" ]; then
        CARGO=cargo
    else
        CARGO=cross
    fi

    $CARGO build --target $TARGET
    $CARGO build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    $CARGO test --target $TARGET
    $CARGO test --target $TARGET --release

    # cross run --target $TARGET
    # cross run --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi
