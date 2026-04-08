# Bitcoin Wallet GUI

Ứng dụng ví Bitcoin desktop viết bằng Rust + `iced`, tập trung vào trải nghiệm quản lý nhiều ví, backup mnemonic rõ ràng, và thao tác send/receive nhanh.

## 0. Cập nhật gần đây

### Giao diện & Trải nghiệm (UI/UX)
- **Light/Dark Theme:** Hỗ trợ 3 chế độ giao diện (Tối / Sáng / Theo hệ thống)
- **Chế độ thu gọn (Compact Mode):** Giảm khoảng cách và padding để hiển thị nhiều nội dung hơn
- **Hiển thị Satoshi:** Tùy chọn hiển thị số dư satoshi bên cạnh BTC
- **Điều hướng bằng phím:** Dùng phím Mũi tên (↑/↓) để chuyển qua lại giữa các mục trong Sidebar
- **Menu trợ giúp phím tắt:** Bấm `F1` hoặc `Ctrl + /` để xem danh sách phím tắt
- **Phím tắt điều hướng:** `Ctrl + 1 đến 6` để nhảy nhanh đến Dashboard, Ví, Gửi, Nhận, Lịch sử, Cài đặt

### Onboarding Tour
- Hướng dẫn 5 bước cho người dùng mới làm quen với ứng dụng
- Hiển thị tự động ngay khi tạo ví lần đầu
- Có thể xem lại bất cứ lúc nào qua menu Cài đặt

### Cài đặt (Settings) mới
- **Trợ năng (Accessibility):** Thanh trượt cỡ chữ (80% - 150%), nút bật/tắt Tương phản cao
- **Mạng lưới (Network):** Nhập Esplora endpoint tùy chọn, chọn Timeout, nút Test kết nối
- **Nâng cao (Advanced):** Bật/tắt Debug logging, Tự động refresh
- **Đặt lại cài đặt:** Nút khôi phục toàn bộ cài đặt về mặc định

### Tính năng mới (Send & History)
- **Nhập số lượng theo BTC:** Nhập số lượng trực tiếp bằng BTC (ví dụ: 0.001), tự động chuyển đổi sang satoshi
- **Nút MAX:** Tự động tính và điền số lượng tối đa có thể gửi (sau khi trừ phí ước tính)
- **Tự động điền phí:** Khi ước tính phí, phí sẽ tự động được điền vào ô phí
- **Chọn ví trong History:** Thêm dropdown chọn ví trong trang Lịch sử giao dịch

### Giao diện Login (Màn hình khởi động)
- **Khi chưa có passphrase:** Chỉ hiện 2 tab "Tạo passphrase mới" và "Import backup" (ẩn tab "Đăng nhập" vì chưa có dữ liệu)
- **Import backup:** Chỉ hiện nút "Chọn file backup" (bỏ ô nhập đường dẫn thủ công), hiển thị đường dẫn file sau khi chọn
- **Nút "Chọn file backup":** Nhỏ gọn, vừa đủ nội dung
- **Chọn ngôn ngữ:** Có sẵn ngay tại màn hình Login, không cần đăng nhập trước

### Giao diện Settings (Cài đặt)
- **Xuất backup:** Bấm nút → mở dialog chọn nơi lưu ngay (giống export mnemonic), không cần nhập path thủ công
- **Kích thước nút:** Các nút vừa đủ nội dung, không chiếm toàn bộ chiều ngang

### Hệ thống ngôn ngữ
- Hỗ trợ English và Tiếng Việt
- Ngôn ngữ đã chọn được lưu lại, hiển thị đúng khi mở app lần sau
- Mặc định lần chạy đầu tiên: English

## 1. Tính năng chính

### Đăng nhập & Khởi tạo
- Đăng nhập bằng passphrase (khi đã có dữ liệu)
- Tạo dữ liệu ví mới với passphrase + nickname (khi chưa có dữ liệu)
- Import backup khi app chưa có dữ liệu

### Quản lý ví
- Tạo ví mới (chọn network: Testnet/Mainnet)
- Import ví từ BIP39 mnemonic
- Import ví từ SLIP-0039 shares
- Xóa ví
- Xem danh sách tất cả ví
- **Gán nhãn & Màu sắc:** Phân loại ví (đang phát triển)

### Gửi BTC (Send)
- Chọn ví gửi
- Nhập địa chỉ nhận và số lượng (nhập theo đơn vị BTC, ví dụ: 0.001)
- **Nút MAX:** Tự động tính và điền số lượng tối đa có thể gửi (sau khi trừ phí)
- **Ước tính phí:** Tự động điền phí ước tính vào ô phí
- Chọn chế độ phí: Auto hoặc Fixed
- Tùy chọn gửi toàn bộ số dư (Send-all)
- Tùy chọn nâng cao: input source, change address

### Nhận BTC (Receive)
- Chọn ví nhận
- Tạo địa chỉ mới
- Copy địa chỉ
- Hiển thị QR code trong popup overlay

### Lịch sử giao dịch
- **Chọn ví:** Dropdown chọn ví để xem lịch sử
- Xem lịch sử theo ví đang chọn
- Lọc: All / Incoming / Outgoing
- Copy txid
- Mở giao dịch trên block explorer

### Backup & Mnemonic
- Xem mnemonic (yêu cầu nhập passphrase)
- Bài test xác nhận backup
- Cảnh báo ví chưa backup
- Export mnemonic ra PDF
- Tách mnemonic thành SLIP-0039 shares (K/N)
- Export shares thành nhiều file PDF

### Cài đặt (Settings)
- **Bảo mật:** Đổi passphrase, Xóa toàn bộ dữ liệu
- **Giao diện (Appearance):** Chế độ Light/Dark, Cỡ chữ (80-150%), Tương phản cao
- **Mạng lưới (Network):** Endpoint Esplora tùy chọn, Timeout, Test kết nối
- **Nâng cao (Advanced):** Debug logging, Tự động refresh, Compact mode, Hiển thị satoshi
- **Dữ liệu:** Xem đường dẫn lưu trữ, dung lượng, Export/Import Settings, Reset cài đặt
- **Hỗ trợ:** Backup mã hóa toàn app, Onboarding Tour cho người mới

## 2. Hướng dẫn sử dụng theo màn hình

### Login / Startup

**Khi app chưa có dữ liệu:**
- Tab "Tạo passphrase mới": Nhập nickname + passphrase để tạo ví mới
- Tab "Import backup": Chọn file backup → nhập passphrase để khôi phục

**Khi app đã có dữ liệu:**
- Tab "Đăng nhập": Nhập passphrase để truy cập

### Dashboard
- Xem tổng quan số dư (tổng, confirmed)
- Số lượng ví
- Nút làm mới dữ liệu

### Wallets (Quản lý ví)
- Tạo ví mới
- Import ví từ mnemonic hoặc SLIP-0039 shares
- Chọn ví để xem chi tiết
- Với ví có mnemonic:
  - Xem mnemonic (cần passphrase)
  - Thực hiện bài test backup
  - Export mnemonic PDF
  - Tách thành SLIP-0039 shares

### Send (Gửi BTC)
- Chọn ví gửi từ danh sách
- Xem số dư khả dụng
- Nhập địa chỉ nhận
- Nhập số lượng (hoặc bật Send-all)
- Chọn chế độ phí
- Gửi giao dịch

### Receive (Nhận BTC)
- Chọn ví nhận
- Tạo địa chỉ mới
- Copy địa chỉ
- Mở popup QR code

### History (Lịch sử)
- Chọn ví từ dropdown để xem lịch sử
- Xem lịch sử giao dịch
- Lọc theo loại: All / Incoming / Outgoing
- Copy txid hoặc mở trên block explorer

### Settings (Cài đặt)
- Bảo mật: Đổi passphrase
- Xuất backup: Lưu file backup mã hóa
- Thông tin: Giới thiệu ứng dụng
- Vùng nguy hiểm: Xóa toàn bộ dữ liệu

### Phím tắt (Keyboard Shortcuts)
- **Điều hướng nhanh:** `Ctrl + 1` (Dashboard), `Ctrl + 2` (Ví), ... `Ctrl + 6` (Cài đặt)
- **Điều hướng tuần tự:** Phím `Mũi tên lên` / `Mũi tên xuống` (chuyển mục trong Sidebar)
- **Thao tác:** `Enter` hoặc `Space` (Xác nhận/Gửi form), `Esc` (Đóng popup/Hủy)
- **Clipboard:** `Ctrl + C` (Sao chép địa chỉ/TxID), `Ctrl + V` (Dán địa chỉ)
- **Tiện ích:** `Ctrl + S` (Lưu trạng thái), `Ctrl + F` (Tìm kiếm), `F1` (Trợ giúp phím tắt)

## 3. Đa ngôn ngữ

- **Hỗ trợ:** English, Tiếng Việt
- **Mặc định:** English (lần chạy đầu tiên)
- **Đổi ngôn ngữ tại:**
  - Màn hình Login
  - Settings
- **Lưu tự động:** Ngôn ngữ đã chọn được lưu lại cho lần sau

## 4. Các loại ví hỗ trợ

### Ví HD tạo mới
- Sinh mnemonic BIP39
- Derive nhiều địa chỉ nhận
- Theo dõi lịch sử, số dư
- Gửi/nhận BTC

### Ví import từ BIP39 mnemonic
- Khôi phục từ seed phrase chuẩn
- Đánh dấu đã có backup

### Ví import từ SLIP-0039 shares
- Khôi phục từ K/N shares
- Hoạt động như ví bình thường sau khôi phục

### Network hỗ trợ
- Testnet
- Mainnet

## 5. SLIP-0039 (Backup phân mảnh)

### Giới thiệu
SLIP-0039 chia bí mật thành nhiều mảnh để tăng an toàn.

### Cách sử dụng
1. Chọn mô hình K/N (ví dụ: 2/3 = cần 2 trong 3 mảnh)
2. App tạo N mảnh (shares)
3. Mỗi mảnh là một cụm từ riêng
4. Cần K mảnh để khôi phục

### Khuyến nghị
- Không lưu tất cả share cùng một nơi
- Tách share ra các vị trí vật lý khác nhau
- Bảo vệ thêm bằng passphrase SLIP-0039 nếu cần
- Kiểm tra quy trình khôi phục định kỳ

## 6. Lưu ý an toàn

- **Passphrase:** Bảo vệ dữ liệu local, không thay thế mnemonic backup
- **Mnemonic/Shares:** Ai có chúng có thể khôi phục và dùng quỹ
- **Ưu tiên:** Backup mnemonic/share offline
- **Tránh:** Chụp màn hình mnemonic trên thiết bị không tin cậy

## 7. Build và chạy ứng dụng

### Yêu cầu
- Rust stable (cài bằng `rustup`)
- Cargo (đi kèm Rust)
- Linux: có thể cần package GUI (Wayland/X11)

### Quick start
```bash
cargo fmt
cargo check
cargo run
```

### Build release
```bash
cargo build --release
```

Binary tại: `target/release/btc_wallet_gui`

### Kiểm tra code
```bash
cargo fmt
cargo check
```

## 8. Build tự động với GitHub Actions

### Workflow có sẵn

Project đã setup GitHub Actions để build tự động cho tất cả nền tảng:

- **build.yml**: Build test khi push code
- **release.yml**: Tự động tạo release khi push tag

### Cách sử dụng

#### **Build test (tự động):**
- Push code lên `main` hoặc `develop` branch
- GitHub Actions tự động build cho macOS, Windows, Linux
- Xem kết quả tại tab **Actions** trong GitHub repo

#### **Tạo Release (tự động):**
```bash
# Tạo tag và push
git tag v1.0.0
git push origin v1.0.0
```

GitHub Actions sẽ tự động:
1. Build cho tất cả nền tảng
2. Tạo macOS Universal Binary (Intel + Apple Silicon)
3. Tạo GitHub Release với tất cả file binary
4. Generate release notes

#### **Chạy build thủ công:**
1. Vào GitHub repo → tab **Actions**
2. Chọn workflow **Build All Platforms**
3. Click **Run workflow**

### Kết quả build

Sau khi build xong, download artifacts:

| Nền tảng | File | Mô tả |
|----------|------|--------|
| macOS Intel | `btc_wallet_gui_macos_intel.tar.gz` | Cho MacBook Intel |
| macOS Apple Silicon | `btc_wallet_gui_macos_arm64.tar.gz` | Cho MacBook M1/M2/M3 |
| macOS Universal | `btc_wallet_gui_macos_universal.tar.gz` | Hỗ trợ cả Intel và ARM |
| Windows x64 | `btc_wallet_gui_windows_x64.zip` | Cho Windows 10/11 |
| Linux x64 | `btc_wallet_gui_linux_x64.tar.gz` | Cho Ubuntu/Debian/Fedora |

### Download binary

**Cách 1: Từ Actions (Build test)**
1. Vào **Actions** → chọn workflow run
2. Scroll xuống **Artifacts**
3. Download file theo nền tảng

**Cách 2: Từ Releases (tự động)**
1. Vào tab **Releases**
2. Chọn version mới nhất
3. Download file theo nền tảng

## 9. Công nghệ sử dụng

- **UI Framework:** `iced`
- **Mã hóa:** ChaCha20-Poly1305 + Argon2id
- **Bitcoin:** `bitcoin` crate v0.32
- **Mnemonic:** `bip39`, `sssmc39` (cho SLIP-0039)
- **File picker:** `rfd` (native)
- **PDF export:** `printpdf`
- **Networking:** `reqwest` (giao tiếp Esplora API)

## 10. Cấu trúc project

```
src/
├── app/           # Logic ứng dụng chính
├── views/         # Giao diện người dùng
├── storage/       # Lưu trữ và mã hóa
├── wallet/        # Logic ví Bitcoin
├── utils/         # Tiện ích hỗ trợ
├── theme/         # Theme & Style (Light/Dark, Colors)
├── components/    # Các component tái sử dụng
├── i18n.rs        # Đa ngôn ngữ
└── main.rs        # Điểm khởi chạy
```

## 11. License

MIT License