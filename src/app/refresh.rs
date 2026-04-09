use iced::Task;

use crate::app::structure::*;
use crate::i18n::t;

impl App {
    pub fn update_dashboard(&mut self) {
        let total: i64 = self.wallets.iter().map(|wallet| wallet.balance()).sum();

        let confirmed: i64 = self
            .wallets
            .iter()
            .map(|wallet| wallet.confirmed_balance())
            .sum();

        let pending = total - confirmed;
        let backup_needed = self
            .wallets
            .iter()
            .filter(|wallet| wallet.has_mnemonic && !wallet.mnemonic_backed_up)
            .count();

        self.dashboard.update_balances(
            total,
            confirmed,
            pending,
            self.wallets.len(),
            backup_needed,
        );
    }

    pub fn refresh_all_wallets(&mut self) -> Task<AppMessage> {
        if self.wallets.is_empty() {
            self.add_info_toast(t("Không có ví để làm mới", "No wallets to refresh").to_string());
            return Task::none();
        }

        if self.is_refreshing {
            return Task::none();
        }

        self.is_refreshing = true;
        self.add_info_toast(
            t("Đang làm mới dữ liệu ví...", "Refreshing wallet data...").to_string(),
        );
        self.error = None;

        let wallets = self.wallets.clone();
        let wallet_vault = self.wallet_vault.clone();
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let mut wallets = wallets;
                    let mut refreshed_wallets = 0usize;
                    let mut refreshed_txs = 0usize;
                    let mut errors = Vec::new();

                    for wallet in &mut wallets {
                        let Some(secrets) = wallet_vault.get(wallet.wallet_id()) else {
                            errors.push(format!("{}: thiếu wallet secret", wallet.name));
                            continue;
                        };

                        match wallet.refresh_history(secrets.as_ref()) {
                            Ok(count) => {
                                refreshed_wallets += 1;
                                refreshed_txs += count;
                            }
                            Err(err) => {
                                errors.push(format!("{}: {}", wallet.name, err));
                            }
                        }
                    }

                    Ok(RefreshWalletsResult {
                        wallets,
                        refreshed_wallets,
                        refreshed_txs,
                        errors,
                    })
                })
                .await
                .unwrap_or_else(|err| Err(format!("Wallet refresh task failed: {err}")))
            },
            AppMessage::RefreshWalletsFinished,
        )
    }
}
