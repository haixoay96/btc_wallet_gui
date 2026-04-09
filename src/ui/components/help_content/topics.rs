use crate::ui::components::tooltip::HelpTopic;

use super::structure::*;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_retrieval() {
        assert!(send_screen_topics().len() == 3);
    }
}
