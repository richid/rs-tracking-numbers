# rs-tracking-numbers
Rust library for gathering information about tracking numbers from UPS, FedEx, etc.

# TODO

- [x] Regex matching
- [x] Checksum validation
- [x] Additional check validation
- [ ] Only run regex once, reuse for other checks
- [ ] Move to services, not couriers
- [ ] Figure out partners
- [ ] Cache files/courier structs, don't read from disk each time
- [ ] Profile / benchmark for fun
