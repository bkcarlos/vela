# Cli

## Testing

You can test your changes to the `cli` crate by first building the main vela binary:

```
cargo build -p vela
```

And then building and running the `cli` crate with the following parameters:

```
 cargo run -p cli -- --vela ./target/debug/vela.exe
```
