# tracking-numbers

A Rust library for validating and identifying tracking numbers from various shipping carriers (UPS, FedEx, USPS, and more). The
underlying tracking number formats are provided by the [`tracking_number_data`](https://github.com/jkeen/tracking_number_data/)
project and provides a Rust wrapper around them.

Also: a fun way to experiment with Rust.

## Features

- 🔍 **Automatic Courier Detection** - Identifies which courier a tracking number belongs to
- ✅ **Validation** - Validates tracking numbers using:
  - Regex pattern matching
  - Checksum algorithms (Mod10, Mod7, Luhn, S10, etc.)
  - Additional business rules (country codes, service types)
- 🚀 **Zero Runtime Dependencies** - All courier data embedded at compile time

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
tracking-numbers = "0.1"
```

## Usage

```rust
use tracking_numbers::track;

fn main() {
    // Track a UPS package
    if let Some(result) = track("1Z5R89390357567127") {
        println!("Courier: {}", result.courier);
        println!("Service: {}", result.service);
        println!("Tracking URL: {}", result.tracking_url);
    }
}
```

**Output:**
```
Courier: UPS
Service: UPS
Tracking URL: https://wwwapps.ups.com/WebTracking/track?track=yes&trackNums=1Z5R89390357567127
```

## Data Source

Courier configuration and validation rules are sourced from the [tracking_number_data](https://github.com/jkeen/tracking_number_data) repository, which is included as a git submodule.

## Building from Source

```bash
# Clone with submodules
git clone --recurse-submodules https://github.com/yourusername/tracking-numbers.git
cd tracking-numbers

# Or if already cloned, initialize submodules
git submodule update --init

# Build
cargo build --release

# Run tests
cargo test
```

## Benchmarking

Run initial benchmark and save baseline:
```bash
# Save the baseline for comparison
cargo bench --bench tracking_benchmark -- --save-baseline before
```

After changes are made, run new benchmark comparing against baseline
```bash
# Run benchmark with the new baseline
cargo bench --bench tracking_benchmark -- --baseline before
```

## License

MIT (See LICENSE.md).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
