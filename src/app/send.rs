use iced::Task;

use crate::i18n::t;
use crate::utils::short_txid;
use crate::views::send::{SendEvent, SendMessage};
use crate::wallet::TxBuildOptions;

use super::{App, AppMessage};
use crate::views::send::SendRequest;

impl App {
    pub fn handle_send_message(&mut self, msg: SendMessage) -> Task<AppMessage> {
        if let Some(event) = self.send_view.update(msg) {
            match event {
                SendEvent::SelectWallet(index) => return self.handle_select_wallet(index),
                SendEvent::EstimateSendFee { amount_sat, input_source } => return self.handle_estimate_send_fee(amount_sat, input_source),
                SendEvent::MaxAmount { input_source } => return self.handle_max_amount(input_source),
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
        if let Some(wallet) = self.wallets.get(self.selected_wallet) {
            match wallet.estimate_auto_fee_for_amount(amount_sat, &input_source) {
                Ok(fee) => {
                    self.send_view.set_fee_amount(fee);
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

    pub fn handle_max_amount(
        &mut self,
        input_source: crate::wallet::InputSource,
    ) -> Task<AppMessage> {
        if let Some(wallet) = self.wallets.get(self.selected_wallet) {
            let balance = wallet.balance();
            if balance <= 0 {
                let message = t("Số dư bằng 0", "Balance is zero").to_string();
                self.send_view.set_error(message.clone());
                self.error = Some(message);
                return Task::none();
            }

            // Use estimate_fee_for_send_all to get accurate max amount and fee
            match wallet.estimate_fee_for_send_all(&input_source) {
                Ok((max_amount, fee)) => {
                    self.send_view.set_max_amount(max_amount);
                    self.send_view.set_fee_amount(fee);
                    self.status = Some(format!(
                        "{}: {} sat (- {} sat {})",
                        t("Số lượng tối đa", "Max amount"),
                        max_amount,
                        fee,
                        t("phí", "fee")
                    ));
                    self.error = None;
                }
                Err(err) => {
                    self.send_view.set_error(err.to_string());
                    self.error = Some(format!(
                        "{}: {err}",
                        t("Không thể tính số lượng tối đa", "Cannot calculate max amount")
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
        if let Some(wallet) = self.wallets.get_mut(self.selected_wallet) {
            let tx_options = TxBuildOptions {
                broadcast: true,
                input_source: request.input_source.clone(),
                change_strategy: request.change_strategy.clone(),
            };

            let amount_sat = match request.amount_sat {
                Some(value) if value > 0 => value,
                _ => {
                    self.send_view.set_error(t(
                        "Vui lòng nhập số lượng",
                        "Please enter amount",
                    ));
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

            let result = wallet.create_transaction_with_options(
                &request.to_address,
                amount_sat,
                fee_sat,
                tx_options,
            );

            match result {
                Ok(tx_result) => {
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
                    self.send_view.clear_form();
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