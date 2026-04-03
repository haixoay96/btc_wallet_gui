use iced::Task;

use crate::i18n::t;
use crate::utils::short_txid;
use crate::views::send::{SendEvent, SendMessage};
use crate::wallet::{validate_address_for_network, TxBuildOptions};

use super::{App, AppMessage, SendExecutionResult};
use crate::views::send::SendRequest;

impl App {
    pub fn handle_send_message(&mut self, msg: SendMessage) -> Task<AppMessage> {
        if let Some(event) = self.send_view.update(msg) {
            match event {
                SendEvent::SelectWallet(index) => return self.handle_select_wallet(index),
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
        self.status = Some(t("Đang ước tính phí...", "Estimating fee...").to_string());
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
        self.status =
            Some(t("Đang tính số lượng tối đa...", "Calculating max amount...").to_string());
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
        self.status = Some(t("Đang gửi giao dịch...", "Sending transaction...").to_string());
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
                self.status = Some(format!("{}: {fee} sat", t("Phí ước tính", "Estimated fee")));
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
                self.status = Some(send_message);
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
