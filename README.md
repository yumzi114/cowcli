# cowcmd
## .cargo/config.toml sample

'''
[env]
SOCKETURL = "wss://socketurl"
SERIAL_DEVICE = "/dev/usb/ttyUSB"

[build]
rustflags = ["--cfg", "tokio_unstable"]
'''


