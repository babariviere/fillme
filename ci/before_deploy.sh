# This script takes care of building your crate and packaging it for release

set -ex

main() {
    cross build --target $TARGET --release

	mkdir -p bin/
    cp target/$TARGET/release/fillme bin/$CRATE_NAME-$TARGET
}

main
