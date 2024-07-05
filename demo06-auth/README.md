

```bash
cargo watch -q -c -x "test -p demo06-auth test_config -- --nocapture"
```

```bash
cargo watch -q -c -w demo06-auth/src -w .cargo/ -x "run -p demo06-auth"
```

```bash
cargo watch -q -c -w demo06-auth/src -w .cargo/ -w demo06-auth/examples -x "run -p demo06-auth --example quick_dev"
```