# lootscribe-slicer

Slicer tool for Orca Slicer + Shopee pricing calculator.

## Features

- Execute Orca Slicer CLI with K1C profile configuration
- Parse print time and filament usage from generated G-code
- Calculate optimal Shopee selling prices with fee tiers
- Interactive mode with default values
- Clipboard copy for pricing results

## Quick Start

```bash
# Build
cargo build --release

# Run with a .stl or .3mf file
./target/release/lootscribe-slicer --run --input path/to/model.stl

# Interactive mode (no --run flag)
./target/release/lootscribe-slicer
```

## CLI Arguments

| Flag | Description | Default |
|------|-------------|---------|
| `--run` | One-shot mode: process a single file and exit | — |
| `--file <PATH>` | Input file (.stl or .3mf) | Required with `--run` |
| `--config-dir <DIR>` | Orca Slicer config directory | `k1c_profile` |
| `--no-orient` | Disable auto-orientation | Enabled |
| `--no-arrange` | Disable plate arrangement | Enabled |
| `--material-price <VAL>` | Filament price R$/kg | 125 |
| `--profit <VAL>` | Profit multiplier | 2.0 |
| `--fail-rate <VAL>` | Failure rate multiplier | 1.1 |
| `--paint <VAL>` | Paint/extra cost R$ | 0 |
| `--time-cost <VAL>` | Machine cost R$/h | 0.8 |
| `--no-clipboard` | Disable copy to clipboard | Disabled |

## Roadmap

See [ROADMAP.md](ROADMAP.md) for planned features and tracked tasks.