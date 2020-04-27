# Tiqer
CLI command that shows price ticker.

```bash
tiqer binance ticker btcusdt xmrusdt

BTCUSDT: 7654.85000000
XMRUSDT: 61.14000000
```

## Installation
### Cargo
```bash
cargo install tiqer
```

### Manual
```bash
git clone git@github.com:sendyhalim/tiqer.git

cd tiqer

cargo install --path . --force
```

## Usage
### Binance
```bash
tiqer binance ticker <symbol pairs...>
```
