use crate::wallet::{InputSource, ChangeStrategy};
use crate::ui::components::BtcUnit;

#[derive(Debug, Clone)]
pub enum SendMessage {
    SelectWallet(usize),
    ToAddressChanged(String),
    AmountChanged(String),
    FeeAmountChanged(String),
    MaxAmount,
    FromAddressChanged(String),
    ChangeAddressChanged(String),
    ToggleAdvanced,
    EstimateFee,
    Send,
    ConfirmSend,
    CancelSend,
    ChangeUnit(BtcUnit),
    ShowAmountHelp,
    ShowFeeHelp,
    ToggleAddressHelp,
    PassphraseConfirmChanged(String),
    ToggleHelpTopic(String),
    ToggleContactPicker,
    ContactSearchChanged(String),
    SelectContact(String),
    ShowContactForm,
    ContactFormNameChanged(String),
    ContactFormAddressChanged(String),
    ContactFormNoteChanged(String),
    SaveContact,
    CancelContactForm,
    HideContactPicker,
    DeleteContact(String),
    EditContact(String),
}

pub struct SendRequest {
    pub to_address: String,
    pub amount_sat: Option<u64>,
    pub fee_sat: Option<u64>,
    pub input_source: InputSource,
    pub change_strategy: ChangeStrategy,
}

pub enum SendEvent {
    SelectWallet(usize),
    EstimateSendFee { amount_sat: u64, input_source: InputSource },
    MaxAmount { input_source: InputSource },
    SendTransaction(SendRequest),
}

pub struct SendView {
    pub to_address: String,
    pub amount: String,
    pub fee_amount: String,
    pub from_address: String,
    pub change_address: String,
    pub broadcast: bool,
    pub estimated_fee: Option<u64>,
    pub to_address_error: Option<String>,
    pub amount_error: Option<String>,
    pub fee_error: Option<String>,
    pub error: Option<String>,
    pub success: Option<String>,
    pub show_confirm: bool,
    pub show_advanced: bool,
    pub unit: BtcUnit,
    pub show_amount_help: bool,
    pub show_fee_help: bool,
    pub show_address_help: bool,
    pub passphrase_confirm: String,
    pub expanded_help_topics: std::collections::HashSet<String>,
    pub show_contact_picker: bool,
    pub contact_search_query: String,
    pub show_contact_form: bool,
    pub contact_form_name: String,
    pub contact_form_address: String,
    pub contact_form_note: String,
    pub contact_form_address_error: Option<String>,
    pub editing_contact_id: Option<String>,
    pub matched_contact_name: Option<String>,
}

