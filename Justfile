bench:
    cargo bench
    rm -rf result
    cp -r target/criterion result
