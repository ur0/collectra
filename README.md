# Collectra

Collectra is the Electra statistics collector. It stores anonymized information
from devices running Electra.

Here's what is stored:
- The SHA256 hash of your device's UDID
- Your device's model name (eg iPad6,11)
- Your device's iOS version

## Why?

Collectra provides up-to-date statistics regarding device and version
popularity to tweak developers, allowing tweak developers to have a better idea
of the jailbroken iOS ecosystem.

## Privacy

No personally identifying information is stored at any point of time. We use
your device's UDID hash to ensure that no device is added twice to our
database.

The UDID is hashed using a cryptographically strong hash algorithm, and the
hashing is performed on your device. Collectra never has access to your real
UDID.

We also do not log request IP addresses.

## Building and running

Collectra requires a nightly version of [Rust](https://rust-lang.org) to build.

To run a development build, execute `cargo run` from your terminal. You will
also need a database for storing the information (PostgreSQL) — set the
`DATABASE_URL` variable to your connection string.

## License

Licensed under the MIT license, see `COPYING` for details.
