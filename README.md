# cowcmd
## .cargo/config.toml sample

```
[env]
SOCKETURL=""
SERIAL_DEVICE=""
SIZE = "1500:1500"

[build]
rustflags = ["--cfg", "tokio_unstable"]
```


