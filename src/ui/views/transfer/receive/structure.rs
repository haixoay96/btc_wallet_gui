use iced::widget::image;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    SelectWallet(usize),
    CopyAddress(String),
    ToggleQrCode(String),
    CloseQrPopup,
    DeriveNewAddress,
    SelectAddress(usize),
    ToggleAddressHistory,
}

#[derive(Debug, Clone)]
pub enum ReceiveEvent {
    SelectWallet(usize),
    CopyAddress(String),
    DeriveAddresses(u32),
}

pub struct ReceiveView {
    pub selected_index: usize,
    pub copied: bool,
    pub show_qr: bool,
    pub show_all_addresses: bool,
    pub qr_address: Option<String>,
    pub qr_handle: Option<image::Handle>,
    pub qr_error: Option<String>,
}
