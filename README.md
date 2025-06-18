## About

OpenSMTPd filter which rejects eMails containing the header X-Google-Group-Id.

## Build

Compile like any other Rust program: `cargo build -r`

Find the resulting binary directly under `target/release/`.

## Usage

Integrate this filter into smtpd.conf(5).
