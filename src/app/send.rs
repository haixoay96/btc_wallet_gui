use iced::Task;

use crate::i18n::t;
use crate::utils::short_txid;
use crate::views::send::{SendEvent, SendMessage};
use crate::wallet::{validate_address_for_network, TxBuildOptions};

use super::{App, AppMessage, SendExecutionResult};
use crate::views::send::SendRequest;

impl App {
    pub fn handle_send_message(&mut self, msg: SendMessage) -> Task<AppMessage> {
        // Track address changes to update matched contact label
        if let SendMessage::ToAddressChanged(addr) = &msg {
            if let Some(contact) = self.address_book.find_by_address(addr) {
                self.send_view.matched_contact_name = Some(contact.name.clone());
            } else {
                self.send_view.matched_contact_name = None;
            }
        }
        
        // Handle Contact Book messages first
        match &msg {
            SendMessage::SaveContact => {
                let name = self.send_view.contact_form_name.clone();
                let address = self.send_view.contact_form_address.clone();
                let note = self.send_view.contact_form_note.clone();
                let editing_id = self.send_view.editing_contact_id.clone();
                
                if name.trim().is_empty() || address.trim().is_empty() {
                    self.error = Some(t("Tên và địa chỉ không được để trống", "Name and address are required").to_string());
                    return Task::none();
                }
                
                // Validate BTC address before saving
                if let Some(wallet) = self.wallets.get(self.selected_wallet) {
                    if let Err(e) = validate_address_for_network(&address, wallet.network) {
                        self.error = Some(format!("{}: {}", t("Địa chỉ không hợp lệ", "Invalid address"), e));
                        return Task::none();
                    }
                } else {
                    // If no wallet selected, validate as mainnet
                    if let Err(e) = validate_address_for_network(&address, crate::wallet::WalletNetwork::Mainnet) {
                        self.error = Some(format!("{}: {}", t("Địa chỉ không hợp lệ", "Invalid address"), e));
                        return Task::none();
                    }
                }
                
                if let Some(id) = editing_id {
                    self.address_book.update_contact(&id, &name, &address, &note);
                    self.add_success_toast(t("Đã cập nhật contact", "Contact updated").to_string());
                } else {
                    let new_id = self.address_book.add_contact(&name, &address, &note);
                    self.send_view.editing_contact_id = Some(new_id);
                    self.add_success_toast(t("Đã thêm contact", "Contact added").to_string());
                }
                
                // Auto-save address book
                let _ = self.address_book.save();
                self.send_view.show_contact_form = false;
                self.send_view.contact_form_name.clear();
                self.send_view.contact_form_address.clear();
                self.send_view.contact_form_note.clear();
                self.send_view.editing_contact_id = None;
                return Task::none();
            }
            SendMessage::SelectContact(address) => {
                // Set address from contact
                self.send_view.to_address = address.clone();
                self.send_view.show_contact_picker = false;
                
                // Update matched contact label
                if let Some(contact) = self.address_book.find_by_address(&address) {
                    self.send_view.matched_contact_name = Some(contact.name.clone());
                } else {
                    self.send_view.matched_contact_name = None;
                }
                
                // Validate address and show error if invalid
                if let Some(wallet) = self.wallets.get(self.selected_wallet) {
                    match validate_address_for_network(&address, wallet.network) {
                        Ok(_) => {
                            self.send_view.to_address_error = None;
                        }
                        Err(e) => {
                            self.send_view.to_address_error = Some(e);
                        }
                    }
                } else {
                    // No wallet selected, validate as mainnet
                    match validate_address_for_network(&address, crate::wallet::WalletNetwork::Mainnet) {
                        Ok(_) => {
                            self.send_view.to_address_error = None;
                        }
                        Err(e) => {
                            self.send_view.to_address_error = Some(e);
                        }
                    }
                }
                return Task::none();
            }
            SendMessage::DeleteContact(id) => {
                self.address_book.delete_contact(&id);
                let _ = self.address_book.save();
                
                // If we were editing this contact, close the form
                if self.send_view.editing_contact_id.as_deref() == Some(&id) {
                    self.send_view.show_contact_form = false;
                    self.send_view.editing_contact_id = None;
                }
                
                self.add_success_toast(t("Đã xóa contact", "Contact deleted").to_string());
                return Task::none();
            }
            SendMessage::EditContact(id) => {
                // Load contact data into form
                if let Some(contact) = self.address_book.get_contact(&id) {
                    self.send_view.editing_contact_id = Some(id.clone());
                    self.send_view.contact_form_name = contact.name.clone();
                    self.send_view.contact_form_address = contact.address.clone();
                    self.send_view.contact_form_note = contact.note.clone();
                    self.send_view.contact_form_address_error = None;
                    self.send_view.show_contact_form = true;
                    self.send_view.show_contact_picker = false;
                }
                return Task::none();
            }
            _ => {}
        }
        
        if let Some(event) = self.send_view.update(msg) {
            match event {
                SendEvent::SelectWallet(index) => {
                    // Re-validate address when wallet changes
                    self.update_matched_contact_label();
                    return self.handle_select_wallet(index);
                }
                SendEvent::EstimateSendFee {
                    amount_sat,
                    input_source,
                } => return self.handle_estimate_send_fee(amount_sat, input_source),
                SendEvent::MaxAmount { input_source } => {
                    return self.handle_max_amount(input_source)
                }
                SendEvent::SendTransaction(req) => return self.handle_send_transaction(req),
            }
        }
        Task::none()
    }

    /// Update matched contact label based on current to_address
    pub fn update_matched_contact_label(&mut self) {
        let address = self.send_view.to_address.clone();
        if let Some(contact) = self.address_book.find_by_address(&address) {
            self.send_view.matched_contact_name = Some(contact.name.clone());
        } else {
            self.send_view.matched_contact_name = None;
        }
    }

    pub fn handle_estimate_send_fee(
        &mut self,
        amount_sat: u64,
        input_source: crate::wallet::InputSource,
    ) -> Task<AppMessage> {
        if self.is_estimating_fee {
            return Task::none();
        }

        let Some(wallet) = self.wallets.get(self.selected_wallet).cloned() else {
            let message = t("Chưa chọn ví", "No wallet selected").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
            return Task::none();
        };

        self.is_estimating_fee = true;
        self.add_info_toast(t("Đang ước tính phí...", "Estimating fee...").to_string());
        self.error = None;

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    wallet
                        .estimate_auto_fee_for_amount(amount_sat, &input_source)
                        .map_err(|err| err.to_string())
                })
                .await
                .unwrap_or_else(|err| Err(format!("Fee estimation task failed: {err}")))
            },
            AppMessage::EstimateSendFeeFinished,
        )
    }

    pub fn handle_max_amount(
        &mut self,
        input_source: crate::wallet::InputSource,
    ) -> Task<AppMessage> {
        if self.is_calculating_max {
            return Task::none();
        }

        let Some(wallet) = self.wallets.get(self.selected_wallet).cloned() else {
            let message = t("Chưa chọn ví", "No wallet selected").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
            return Task::none();
        };
        if wallet.balance() <= 0 {
            let message = t("Số dư bằng 0", "Balance is zero").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
            return Task::none();
        }

        self.is_calculating_max = true;
        self.add_info_toast(t("Đang tính số lượng tối đa...", "Calculating max amount...").to_string());
        self.error = None;

        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    wallet
                        .estimate_fee_for_send_all(&input_source)
                        .map_err(|err| err.to_string())
                })
                .await
                .unwrap_or_else(|err| Err(format!("Max amount task failed: {err}")))
            },
            AppMessage::MaxAmountFinished,
        )
    }

    pub fn handle_send_transaction(&mut self, request: SendRequest) -> Task<AppMessage> {
        if self.is_sending {
            return Task::none();
        }

        let Some(wallet) = self.wallets.get(self.selected_wallet).cloned() else {
            let message = t("Chưa chọn ví", "No wallet selected").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
            return Task::none();
        };
        let Some(secrets) = self.wallet_secret_by_index(self.selected_wallet) else {
            let message = t("Thiếu secret của ví", "Wallet secret is missing").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
            return Task::none();
        };

        let amount_sat = match request.amount_sat {
            Some(value) if value > 0 => value,
            _ => {
                self.send_view
                    .set_error(t("Vui lòng nhập số lượng", "Please enter amount"));
                return Task::none();
            }
        };

        let fee_sat = match request.fee_sat {
            Some(value) if value > 0 => value,
            _ => {
                self.send_view.set_error(t(
                    "Vui lòng nhập phí hoặc bấm 'Ước tính phí'",
                    "Please enter fee or click 'Estimate Fee'",
                ));
                return Task::none();
            }
        };

        if let Err(err) = validate_address_for_network(&request.to_address, wallet.network) {
            self.send_view.set_error(err.clone());
            self.error = Some(err);
            return Task::none();
        }

        self.is_sending = true;
        self.add_info_toast(t("Đang gửi giao dịch...", "Sending transaction...").to_string());
        self.error = None;

        let wallet_id = wallet.account_xpub.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut wallet = wallet;
                    let tx_options = TxBuildOptions {
                        broadcast: true,
                        input_source: request.input_source,
                        change_strategy: request.change_strategy,
                    };
                    let tx_result = wallet
                        .create_transaction_with_options(
                            secrets.as_ref(),
                            &request.to_address,
                            amount_sat,
                            fee_sat,
                            tx_options,
                        )
                        .map_err(|err| err.to_string())?;

                    Ok(SendExecutionResult {
                        wallet_id,
                        wallet,
                        txid: tx_result.txid,
                        broadcasted: tx_result.broadcasted,
                    })
                })
                .await
                .unwrap_or_else(|err| Err(format!("Send task failed: {err}")))
            },
            AppMessage::SendTransactionFinished,
        )
    }

    pub fn handle_estimate_send_fee_finished(
        &mut self,
        result: Result<u64, String>,
    ) -> Task<AppMessage> {
        self.is_estimating_fee = false;

        match result {
            Ok(fee) => {
                self.send_view.set_fee_amount(fee);
                self.add_success_toast(format!("{}: {fee} sat", t("Phí ước tính", "Estimated fee")));
                self.error = None;
            }
            Err(err) => {
                self.send_view.set_error(err.clone());
                self.error = Some(format!(
                    "{}: {err}",
                    t("Ước tính phí thất bại", "Fee estimation failed")
                ));
            }
        }

        Task::none()
    }

    pub fn handle_max_amount_finished(
        &mut self,
        result: Result<(u64, u64), String>,
    ) -> Task<AppMessage> {
        self.is_calculating_max = false;

        match result {
            Ok((max_amount, fee)) => {
                self.send_view.set_max_amount(max_amount);
                self.send_view.set_fee_amount(fee);
                self.add_success_toast(format!(
                    "{}: {} sat (- {} sat {})",
                    t("Số lượng tối đa", "Max amount"),
                    max_amount,
                    fee,
                    t("phí", "fee")
                ));
                self.error = None;
            }
            Err(err) => {
                self.send_view.set_error(err.clone());
                self.error = Some(format!(
                    "{}: {err}",
                    t(
                        "Không thể tính số lượng tối đa",
                        "Cannot calculate max amount"
                    )
                ));
            }
        }

        Task::none()
    }

    pub fn handle_send_transaction_finished(
        &mut self,
        result: Result<SendExecutionResult, String>,
    ) -> Task<AppMessage> {
        self.is_sending = false;

        match result {
            Ok(payload) => {
                if let Some(wallet) = self
                    .wallets
                    .iter_mut()
                    .find(|wallet| wallet.account_xpub == payload.wallet_id)
                {
                    *wallet = payload.wallet;
                } else {
                    self.send_view.set_error(t(
                        "Không tìm thấy ví để cập nhật sau khi gửi",
                        "Could not find wallet to update after sending",
                    ));
                    return Task::none();
                }

                let save_succeeded = self.save_state();
                self.update_dashboard();

                let short_txid = short_txid(&payload.txid);
                let send_message = if payload.broadcasted {
                    format!(
                        "{}: {short_txid}",
                        t("Đã broadcast giao dịch", "Transaction broadcasted")
                    )
                } else {
                    format!(
                        "{}: {short_txid}",
                        t(
                            "Đã tạo giao dịch (chưa broadcast)",
                            "Transaction created (not broadcast)",
                        )
                    )
                };
                self.send_view.set_success(send_message.clone());
                self.send_view.clear_form();
                self.add_info_toast(send_message);
                if save_succeeded {
                    self.error = None;
                }
            }
            Err(err) => {
                self.send_view.set_error(err.clone());
                self.error = Some(format!(
                    "{}: {err}",
                    t("Gửi giao dịch thất bại", "Send transaction failed")
                ));
            }
        }

        Task::none()
    }
}
