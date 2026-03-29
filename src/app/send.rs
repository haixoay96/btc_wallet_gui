use iced::Task;

use crate::i18n::t;
use crate::views::send::{SendEvent, SendMessage};
use crate::wallet::{FeeMode, TxBuildOptions, Wallet};

use super::{short_txid, App, AppMessage, SendRequest};

impl App {
    pub fn handle_send_message(&mut self, msg: SendMessage) -> Task<AppMessage> {
        if let Some(event) = self.send_view.update(msg) {
            match event {
                SendEvent::SelectWallet(index) => return self.handle_select_wallet(index),
                SendEvent::EstimateSendFee { amount_sat, input_source } => return self.handle_estimate_send_fee(amount_sat, input_source),
                SendEvent::SendTransaction(req) => return self.handle_send_transaction(req),
            }
        }
        Task::none()
    }

    pub fn handle_estimate_send_fee(
        &mut self,
        amount_sat: u64,
        input_source: crate::wallet::InputSource,
    ) -> Task<AppMessage> {
        if let Some(wallet_entry) = self.wallets.get(self.selected_wallet) {
            let wallet = Wallet {
                entry: wallet_entry.clone(),
            };
            match wallet.estimate_auto_fee_for_amount(amount_sat, &input_source) {
                Ok(fee) => {
                    self.send_view.set_estimated_fee(fee);
                    self.status =
                        Some(format!("{}: {fee} sat", t("Phí ước tính", "Estimated fee")));
                    self.error = None;
                }
                Err(err) => {
                    self.send_view.set_error(err.to_string());
                    self.error = Some(format!(
                        "{}: {err}",
                        t("Ước tính phí thất bại", "Fee estimation failed")
                    ));
                }
            }
        } else {
            let message = t("Chưa chọn ví", "No wallet selected").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
        }
        Task::none()
    }

    pub fn handle_send_transaction(&mut self, request: SendRequest) -> Task<AppMessage> {
        if let Some(wallet_entry) = self.wallets.get_mut(self.selected_wallet) {
            let mut wallet = Wallet {
                entry: wallet_entry.clone(),
            };

            let tx_options = TxBuildOptions {
                broadcast: request.broadcast,
                input_source: request.input_source.clone(),
                change_strategy: request.change_strategy.clone(),
            };

            let result = if request.use_all_funds {
                wallet.create_send_all_transaction_with_options(
                    &request.to_address,
                    request.fee_mode,
                    tx_options,
                )
            } else {
                let amount_sat = match request.amount_sat {
                    Some(value) if value > 0 => value,
                    _ => {
                        self.send_view.set_error(t(
                            "Số lượng không hợp lệ cho giao dịch thường",
                            "Invalid amount for regular transaction",
                        ));
                        return Task::none();
                    }
                };

                let fee_sat = match request.fee_mode {
                    FeeMode::Auto => match wallet
                        .estimate_auto_fee_for_amount(amount_sat, &request.input_source)
                    {
                        Ok(value) => value,
                        Err(err) => {
                            self.send_view.set_error(format!(
                                "{}: {err}",
                                t(
                                    "Không thể ước tính phí tự động",
                                    "Could not estimate auto fee",
                                )
                            ));
                            return Task::none();
                        }
                    },
                    FeeMode::FixedSat(value) => value,
                };

                wallet.create_transaction_with_options(
                    &request.to_address,
                    amount_sat,
                    fee_sat,
                    tx_options,
                )
            };

            match result {
                Ok(tx_result) => {
                    *wallet_entry = wallet.entry;
                    self.save_state();
                    self.update_dashboard();

                    let short_txid = short_txid(&tx_result.txid);
                    let send_message = if tx_result.broadcasted {
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
                    self.status = Some(send_message);
                    self.error = None;
                }
                Err(err) => {
                    self.send_view.set_error(err.to_string());
                    self.error = Some(format!(
                        "{}: {err}",
                        t("Gửi giao dịch thất bại", "Send transaction failed")
                    ));
                }
            }
        } else {
            let message = t("Chưa chọn ví", "No wallet selected").to_string();
            self.send_view.set_error(message.clone());
            self.error = Some(message);
        }

        Task::none()
    }
}