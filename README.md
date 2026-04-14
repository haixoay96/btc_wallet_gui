# Bitcoin Wallet GUI

Ứng dụng ví Bitcoin desktop đa nền tảng, viết bằng **Rust + iced**, tập trung vào bảo mật, quản lý nhiều ví, backup mnemonic rõ ràng, và thao tác gửi/nhận nhanh.

---

## ✨ Tính năng nổi bật

### 🔐 Bảo mật & Mã hóa
- **Argon2id + ChaCha20-Poly1305**: Mã hóa toàn bộ dữ liệu ví
- **Secure memory**: Xóa mnemonic/key khỏi RAM khi không dùng
- **Passphrase strength meter**: Đánh giá độ mạnh passphrase realtime
- **Backup verification test**: Bài test điền từ mnemonic để xác nhận đã backup đúng
- **SLIP-0039 Shamir Secret Sharing**: Chia mnemonic thành K/N mảnh, tăng an toàn

### 💼 Quản lý nhiều ví
- Tạo ví mới (Mainnet/Testnet) với BIP39 mnemonic
- Import từ BIP39 mnemonic hoặc SLIP-0039 shares
- **Gán nhãn & màu sắc**: Phân loại ví (Personal, Business, Savings, ...)
- **Sắp xếp**: Theo số dư, tên, ngày tạo, network
- **Tìm kiếm**: Lọc ví theo tên hoặc tag

### 💸 Gửi BTC
- **Unit selector**: Nhập số lượng theo BTC, Satoshi, hoặc mBTC
- **MAX button**: Tự động tính số lượng tối đa (trừ phí)
- **Auto fee estimation**: Ước lượng phí từ Esplora API, tự điền vào form
- **Contact book**: Chọn người nhận từ danh bạ đã lưu, auto-match địa chỉ
- **Advanced options**: Chọn input source, change address strategy
- **Confirmation dialog**: Yêu cầu nhập passphrase trước khi gửi

### 📥 Nhận BTC
- Derive địa chỉ mới on-demand
- **QR code popup**: Hiển thị mã QR để quét nhanh
- Copy địa chỉ với 1 click
- Xem lịch sử địa chỉ đã tạo

### 📊 Dashboard
- **Tổng quan**: Total balance, confirmed, pending balance
- **Sparkline**: Biểu đồ 7 ngày biến động số dư
- **BTC price widget**: Giá BTC/USD realtime từ CoinGecko, 24h change %
- **Network status**: Trạng thái kết nối Esplora API
- **Backup reminder**: Cảnh báo ví chưa backup (có thể snooze 7 ngày)
- **Recent transactions**: Preview 10 giao dịch gần nhất

### 📜 Lịch sử giao dịch
- **Lọc nâng cao**: Incoming / Outgoing / Pending / Self-Transfer
- **Tìm kiếm**: Theo query text
- **Bộ lọc**: Khoảng ngày, min/max amount
- **Phân trang**: Configurable items per page
- **Export**: CSV (UTF-8 BOM) hoặc PDF
- **Block explorer**: Mở giao dịch trên Blockstream (network-aware)

### 🎨 Giao diện & Trải nghiệm
- **3 theme**: Dark / Light / System (auto-detect OS)
- **Compact mode**: Giảm padding/spacing, hiển thị nhiều content hơn
- **Font scale**: Thanh trượt 80% - 150%
- **High contrast mode**: Tăng độ tương phản
- **Show satoshis**: Toggle hiển thị số dư satoshi bên cạnh BTC
- **Toast notifications**: Success / Error / Info, auto-dismiss
- **Skeleton loading**: Placeholder animation khi load dữ liệu
- **Modal system**: QR code, confirmation dialogs, contact forms

### ⌨️ Phím tắt
| Phím | Chức năng |
|------|-----------|
| `Ctrl + 1-6` | Chuyển nhanh: Dashboard, Ví, Gửi, Nhận, Lịch sử, Cài đặt |
| `↑ / ↓` | Điều hướng sidebar |
| `Enter / Space` | Xác nhận form |
| `Esc` | Đóng popup / Hủy |
| `Ctrl + C` | Copy địa chỉ / TxID |
| `Ctrl + V` | Dán địa chỉ |
| `F1 / Ctrl + /` | Xem danh sách phím tắt |

### 🌐 Đa ngôn ngữ
- **English** & **Tiếng Việt**
- Lưu tự động, hiển thị đúng khi mở lại app
- Đổi ngôn ngữ tại Login screen hoặc Settings

### 🧭 Onboarding Tour
- Hướng dẫn 5 bước cho người dùng mới
- Auto-show sau khi tạo ví lần đầu
- Xem lại bất cứ lúc nào qua Settings

### ⚙️ Cài đặt phong phú
- **Bảo mật**: Đổi passphrase, Export backup mã hóa, Xóa toàn bộ dữ liệu
- **Giao diện**: Theme, font scale, high contrast, compact mode
- **Mạng lưới**: Custom Esplora endpoint, timeout, test kết nối
- **Nâng cao**: Debug logging, auto-refresh
- **Dữ liệu**: Export/Import settings, Reset về mặc định

---

## 🛠️ Công nghệ

| Thành phần | Công nghệ |
|-----------|-----------|
| UI Framework | `iced` 0.13 (Tokio runtime) |
| Icons | `iced_fonts` (Bootstrap icons) |
| Bitcoin | `bitcoin` v0.32 |
| BIP39 Mnemonic | `bip39` |
| SLIP-0039 | `sssmc39` |
| Mã hóa | `chacha20poly1305` + `argon2` (Argon2id) |
| Secure Memory | `secrecy` + `zeroize` |
| HTTP | `reqwest` 0.12 (rustls-tls) |
| QR Code | `qrcode` 0.14 |
| PDF | `printpdf` 0.7 |
| File Dialogs | `rfd` (XDG Portal) |
| System Theme | `dark-light` |
| Logging | `tracing` + `tracing-subscriber` |
| Async | `tokio` |

---

## 📦 Cài đặt & Chạy

### Yêu cầu
- Rust stable (`rustup install stable`)
- Linux: `libgtk-3-dev`, `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`

### Quick start
```bash
cargo fmt
cargo check
cargo run
```

### Build release
```bash
cargo build --release
# Binary: target/release/btc_wallet_gui
```

### Kiểm tra code
```bash
cargo fmt --check
cargo clippy
cargo check
```

---

## 🚀 Build tự động (GitHub Actions)

### Workflow
- **release.yml**: Tự động build + tạo release khi push tag `v*`
- Hỗ trợ: macOS Intel, macOS Apple Silicon, Windows x64, Linux x64

### Tạo release
```bash
git tag v1.0.0
git push origin v1.0.0
```

Actions sẽ tự động:
1. Build cho tất cả nền tảng
2. Tạo GitHub Release với artifacts
3. Generate release notes

### Download binary
- Tab **Releases** → Chọn version → Download file theo nền tảng
- Hoặc **Actions** → Chọn workflow run → Download artifacts

---

## 🔒 Lưu ý an toàn

- **Passphrase** bảo vệ dữ liệu local, **không thay thế** mnemonic backup
- **Mnemonic/Shares**: Ai có chúng có thể khôi phục và dùng quỹ
- **Ưu tiên**: Backup mnemonic/share offline, tách các mảnh ra vị trí khác nhau
- **Tránh**: Chụp màn hình mnemonic trên thiết bị không tin cậy

---

## 📂 Cấu trúc project

```
src/
├── main.rs          # Entry point
├── app/             # Application state & logic
├── core/            # Core business logic
├── infra/           # External services (Esplora, CoinGecko)
├── shared/          # Shared types & models
├── ui/
│   ├── views/       # Screens: login, dashboard, wallets, send, receive, history, settings
│   ├── components/  # Reusable UI components
│   ├── theme/       # Theme system (colors, buttons, containers, text)
│   └── i18n.rs      # Internationalization
└── utils/           # Utilities (formatting, validation)
```

---

## 📄 License

MIT License
# Trigger GitHub Contributor Graph Refresh
