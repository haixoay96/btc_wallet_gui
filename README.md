# Bitcoin Wallet GUI

A modern Bitcoin wallet GUI application built with [iced.rs](https://github.com/iced-rs/iced), inspired by Exodus wallet design.

## Features

### Completed
- Login/Register with passphrase
- Encrypted storage using ChaCha20-Poly1305 + Argon2id
- Dashboard with balance overview
- Multiple wallet management (create/select/delete)
- Mnemonic backup with verification test + export PDF
- SLIP-0039 support: split mnemonic (K/N) to shares and import wallet from shares
- Send BTC with fee options (auto/fixed, send-all, advanced input/change options)
- Receive BTC with address list, derive new address, copy to clipboard
- Transaction history with incoming/outgoing/all filters
- Settings: passphrase change, encrypted backup export/import
- Balance & history refresh from Blockstream API

## Build & Run

```bash
cd btc_wallet_gui
cargo build --release
cargo run
```

## Design System

Dark theme inspired by Exodus wallet with purple (#7B61FF) and teal (#00D4AA) accent colors.
