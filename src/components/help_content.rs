use std::borrow::Cow;

use crate::components::tooltip::HelpTopic;
use iced_fonts::Bootstrap;

/// Send Screen help topics
pub fn send_screen_topics() -> Vec<HelpTopic> {
    vec![
        HelpTopic::new(
            "send_fee_estimation",
            Bootstrap::Calculator,
            "Ước tính phí giao dịch",
            "Fee Estimation",
            "Phí cao hơn = giao dịch được xác nhận nhanh hơn. Phí được ước tính dựa trên tình trạng mạng lưới hiện tại.",
            "Higher fees = faster confirmation. Fee is estimated based on current network conditions.",
        )
        .with_detail(
            "Phí trung bình hiện tại: ~5 sat/vB. Giao dịch thường được xác nhận trong 10-60 phút.",
            "Average fee is ~5 sat/vB. Transactions typically confirm within 10-60 minutes.",
        ),
        HelpTopic::new(
            "send_input_strategy",
            Bootstrap::Grid,
            "Chiến lược chọn UTXO",
            "Input Strategy",
            "Tự động chọn UTXOs tối ưu để giảm phí giao dịch và bảo vệ quyền riêng tư.",
            "Automatically selects optimal UTXOs to minimize fees and protect privacy.",
        )
        .with_detail(
            "Ứng dụng sử dụng chiến lược coin selection tốt nhất để giảm số lượng đầu vào.",
            "The app uses the best coin selection strategy to minimize input count.",
        ),
        HelpTopic::new(
            "send_change_address",
            Bootstrap::Shuffle,
            "Địa chỉ tiền thừa (Change)",
            "Change Address",
            "Địa chỉ trả lại Bitcoin thừa sau giao dịch. Đây là cơ chế bình thường của Bitcoin.",
            "Address that receives the remaining Bitcoin after a transaction. This is normal Bitcoin behavior.",
        )
        .with_detail(
            "Bitcoin không thể chia nhỏ giao dịch, nên phần thừa sẽ được gửi lại địa chỉ change mới.",
            "Bitcoin cannot split transactions, so change is sent to a new address.",
        ),
    ]
}

/// Wallets Screen help topics
pub fn wallets_screen_topics() -> Vec<HelpTopic> {
    vec![
        HelpTopic::new(
            "wallet_slip0039",
            Bootstrap::Puzzle,
            "SLIP-0039 Shamir Secret Sharing",
            "SLIP-0039 Shamir Secret Sharing",
            "Chia mnemonic thành nhiều mảnh. Cần tối thiểu K mảnh trong N mảnh để khôi phục ví.",
            "Split mnemonic into multiple shares. Need minimum K shares out of N to recover wallet.",
        )
        .with_detail(
            "Ví dụ: 3/5 nghĩa là cần 3 trong 5 mảnh để khôi phục. Hữu ích cho việc phân tán rủi ro.",
            "Example: 3/5 means you need 3 out of 5 shares. Useful for distributing risk.",
        ),
        HelpTopic::new(
            "wallet_backup_test",
            Bootstrap::ShieldCheck,
            "Kiểm tra backup",
            "Backup Test",
            "Xác minh bạn đã backup mnemonic đúng cách bằng cách nhập lại các từ đã chọn.",
            "Verify you've backed up your mnemonic correctly by re-entering selected words.",
        ),
        HelpTopic::new(
            "wallet_derivation_path",
            Bootstrap::Arrows,
            "Đường dẫn phái sinh (Derivation Path)",
            "Derivation Path",
            "BIP84: m/84'/0'/0' cho Native SegWit (bc1). Đây là chuẩn hiện đại cho Bitcoin.",
            "BIP84: m/84'/0'/0' for Native SegWit (bc1). This is the modern Bitcoin standard.",
        )
        .with_detail(
            "Native SegWit có phí thấp hơn và được hỗ trợ rộng rãi. Tránh dùng Legacy (m/44').",
            "Native SegWit has lower fees and is widely supported. Avoid Legacy (m/44').",
        ),
    ]
}

/// History Screen help topics
pub fn history_screen_topics() -> Vec<HelpTopic> {
    vec![
        HelpTopic::new(
            "history_confirmations",
            Bootstrap::ArrowRepeat,
            "Số lần xác nhận (Confirmations)",
            "Confirmations",
            "Số block được thêm vào blockchain sau giao dịch của bạn. Càng nhiều = càng an toàn.",
            "Number of blocks added to blockchain after your transaction. More = more secure.",
        )
        .with_detail(
            "6 xác nhận được coi là an toàn cho giao dịch lớn. Mỗi block ~10 phút.",
            "6 confirmations is considered secure for large transactions. Each block ~10 minutes.",
        ),
        HelpTopic::new(
            "history_fee_display",
            Bootstrap::Coin,
            "Phí giao dịch",
            "Transaction Fee",
            "Phí network trả cho miners để xử lý giao dịch của bạn. Phí cao hơn = nhanh hơn.",
            "Network fee paid to miners to process your transaction. Higher fee = faster.",
        ),
    ]
}

/// Settings Screen help topics
pub fn settings_screen_topics() -> Vec<HelpTopic> {
    vec![
        HelpTopic::new(
            "settings_passphrase_change",
            Bootstrap::Key,
            "Đổi passphrase",
            "Passphrase Change",
            "Passphrase dùng để mã hóa dữ liệu ví trên máy. Dùng passphrase mạnh và duy nhất.",
            "Passphrase is used to encrypt wallet data on your device. Use a strong, unique passphrase.",
        ),
        HelpTopic::new(
            "settings_export",
            Bootstrap::Download,
            "Export backup",
            "Export Backup",
            "Export backup chứa TẤT CẢ ví và dữ liệu. Giữ file này an toàn và bảo mật.",
            "Export backup contains ALL wallets and data. Keep this file safe and secure.",
        ),
    ]
}

/// Get help topic by ID for dismiss tracking
pub fn get_topic_by_id(id: &str) -> Option<HelpTopic> {
    let send = send_screen_topics();
    let wallets = wallets_screen_topics();
    let history = history_screen_topics();
    let settings = settings_screen_topics();
    
    send.into_iter()
        .chain(wallets)
        .chain(history)
        .chain(settings)
        .find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_topic_retrieval() {
        assert!(get_topic_by_id("send_fee_estimation").is_some());
        assert!(get_topic_by_id("nonexistent").is_none());
    }
}
