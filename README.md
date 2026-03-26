# Bitcoin Wallet GUI - Exodus Style

A modern Bitcoin wallet GUI application built with [iced.rs](https://github.com/iced-rs/iced), inspired by Exodus wallet design.

## Features

### Completed
- Encrypted storage using AES-256-GCM + Argon2
- Multiple wallet management
- Exodus-style dark theme with purple/teal accents
- Dashboard with balance overview
- Sidebar navigation

### In Development
- Send BTC transactions
- Receive BTC with QR codes
- Transaction history
- Settings and passphrase management
- Balance refresh from Blockstream API

## Build & Run

```bash
cd btc_wallet_gui
cargo build --release
cargo run
```

## Design System

Dark theme inspired by Exodus wallet with purple (#7B61FF) and teal (#00D4AA) accent colors.