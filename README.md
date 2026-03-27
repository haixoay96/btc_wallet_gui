# Bitcoin Wallet GUI

Ứng dụng ví Bitcoin desktop viết bằng Rust + `iced`, tập trung vào trải nghiệm quản lý nhiều ví, backup mnemonic rõ ràng, và thao tác send/receive nhanh.

## 1. Tính năng chính

- Đăng nhập bằng passphrase.
- Tạo dữ liệu ví mới với passphrase + nickname.
- Import backup khi app chưa có dữ liệu.
- Dashboard tổng quan số dư (tổng, confirmed, số lượng ví).
- Quản lý nhiều ví: tạo, chọn, xóa.
- Import ví từ:
  - BIP39 mnemonic.
  - SLIP-0039 shares.
- Send BTC:
  - Chọn ví gửi.
  - Fee auto/fixed.
  - Send-all.
  - Tùy chọn nâng cao input/change.
- Receive BTC:
  - Chọn ví nhận.
  - Derive địa chỉ mới.
  - Copy địa chỉ.
  - Hiển thị QR code trong popup overlay.
- History giao dịch có filter All/Incoming/Outgoing.
- Settings:
  - Đổi passphrase.
  - Export backup mã hóa toàn app.
  - Clear toàn bộ dữ liệu (yêu cầu passphrase hiện tại).
  - Chọn ngôn ngữ ứng dụng.
- Mnemonic backup:
  - Yêu cầu nhập passphrase để xem mnemonic.
  - Bài test xác nhận backup.
  - Cảnh báo ví chưa backup.
  - Export mnemonic ra PDF.
- SLIP-0039:
  - Tách mnemonic thành nhiều shares theo ngưỡng K/N.
  - Export thành thư mục PDF (mỗi share một file).

## 2. Đa ngôn ngữ

- Hỗ trợ:
  - English
  - Tiếng Việt
- Mặc định lần mở ứng dụng đầu tiên: `English`.
- Đổi ngôn ngữ trong `Settings`.
- Ngôn ngữ đã chọn được lưu lại để màn hình khởi động/login lần sau hiển thị đúng ngôn ngữ đó.

## 3. Các loại ví hỗ trợ

Hiện tại app hỗ trợ các workflow ví sau:

- Ví HD tạo mới trong app:
  - Sinh mnemonic.
  - Derive nhiều địa chỉ nhận.
  - Theo dõi lịch sử, số dư, gửi/nhận.
- Ví import từ BIP39 mnemonic:
  - Dùng lại seed phrase chuẩn để phục hồi ví.
  - Được đánh dấu đã có backup.
- Ví import từ SLIP-0039 shares:
  - Khôi phục mnemonic từ đủ số mảnh theo ngưỡng K.
  - Sau khi khôi phục sẽ hoạt động như ví bình thường.

Network:

- `Testnet`
- `Mainnet`

## 4. Giải thích thêm về SLIP-0039 (tách mnemonic thành nhiều mảnh)

SLIP-0039 giúp tăng an toàn backup bằng cách chia bí mật thành nhiều mảnh.

- Bạn chọn mô hình `K/N`.
  - Ví dụ `2/3`: tạo 3 mảnh, cần bất kỳ 2 mảnh để khôi phục.
- Mỗi mảnh là một cụm từ riêng (share).
- Một mảnh đơn lẻ (khi K > 1) không đủ để khôi phục toàn bộ ví.
- App cho export các share thành nhiều file PDF trong cùng 1 thư mục.

Khuyến nghị:

- Không lưu tất cả share cùng một nơi.
- Tách share ra các vị trí vật lý khác nhau.
- Bảo vệ thêm bằng passphrase SLIP-0039 nếu cần.
- Vẫn nên kiểm tra quy trình khôi phục định kỳ trên môi trường an toàn.

## 5. Hướng dẫn sử dụng nhanh theo màn hình

### Login / Startup

- Nếu app chưa có dữ liệu:
  - Tạo passphrase mới, nhập nickname.
  - Hoặc import backup.
- Nếu app đã có dữ liệu:
  - Đăng nhập bằng passphrase hiện có.

### Wallets

- Tạo ví mới (chọn network).
- Import ví từ mnemonic hoặc từ SLIP-0039 shares.
- Với ví có mnemonic:
  - Nhập passphrase để hiện mnemonic.
  - Thực hiện bài test backup.
  - Export mnemonic PDF nếu cần.
  - Có thể tách mnemonic thành SLIP-0039 shares để backup phân mảnh.

### Send

- Chọn ví gửi.
- Nhập địa chỉ nhận + amount.
- Chọn fee mode:
  - Auto (có thể estimate).
  - Fixed.
- Có thể bật send-all hoặc dùng advanced options.

### Receive

- Chọn ví nhận.
- Tạo địa chỉ mới nếu cần.
- Copy địa chỉ.
- Mở popup QR code để chia sẻ nhanh.

### History

- Xem lịch sử theo ví đang chọn.
- Lọc giao dịch theo All / Incoming / Outgoing.

### Settings

- Đổi passphrase.
- Chọn ngôn ngữ.
- Export backup mã hóa toàn app.
- Clear toàn bộ dữ liệu app (cần passphrase hiện tại).

## 6. Lưu ý an toàn

- Passphrase bảo vệ dữ liệu local của app, không thay thế mnemonic backup.
- Ai có mnemonic (hoặc đủ shares SLIP-0039) có thể khôi phục và dùng quỹ.
- Ưu tiên backup mnemonic/share offline.
- Không chụp màn hình mnemonic trên thiết bị không tin cậy.

## 7. Build và chạy ứng dụng

## 7.1 Yêu cầu

- Rust stable (khuyên dùng cài bằng `rustup`).
- Cargo (đi kèm Rust).
- Trên Linux có thể cần thêm package hệ thống cho GUI (Wayland/X11 tùy distro).

## 7.2 Chạy dev

```bash
cargo run
```

## 7.3 Build release

```bash
cargo build --release
```

Binary nằm tại:

```bash
target/release/btc_wallet_gui
```

## 7.4 Kiểm tra code

```bash
cargo fmt
cargo check
```

## 8. Công nghệ sử dụng

- UI: `iced`
- Crypto storage: `ChaCha20-Poly1305` + `Argon2id`
- Bitcoin stack: `bdk` và các thành phần liên quan
- File picker native: `rfd`
- PDF export: `printpdf`
