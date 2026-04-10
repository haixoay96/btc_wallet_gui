use iced::Task;

use crate::app::structure::*;
use crate::ui::components::sparkline::calculate_7day_balance_history;
use crate::ui::i18n::t;
use crate::ui::views::dashboard::RecentTxItem;

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

        // Aggregate recent transactions from all wallets
        let recent_transactions = collect_recent_transactions(&self.wallets);

        // Calculate 7-day balance history
        let balance_history = calculate_7day_balance_history(&self.wallets);

        // Check if backup reminder should be shown
        let last_dismissed = if let Ok(storage) = crate::infra::storage::Storage::new() {
            storage.load_backup_reminder_dismissed().unwrap_or(None)
        } else {
            None
        };
        let show_backup_reminder =
            crate::ui::components::backup_reminder::should_show_backup_reminder(
                backup_needed,
                last_dismissed,
            );

        self.dashboard.update_balances(
            total,
            confirmed,
            pending,
            self.wallets.len(),
            backup_needed,
            recent_transactions,
            show_backup_reminder,
            balance_history,
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

/// Collect and sort recent transactions across all wallets
fn collect_recent_transactions(wallets: &[crate::core::wallet::Wallet]) -> Vec<RecentTxItem> {
    let mut all_txs: Vec<RecentTxItem> = Vec::new();

    for wallet in wallets {
        for tx in &wallet.history {
            all_txs.push(RecentTxItem {
                txid: tx.txid.clone(),
                amount_sat: tx.amount_sat,
                confirmed: tx.confirmed,
                block_time: tx.block_time,
                wallet_name: wallet.name.clone(),
            });
        }
    }

    // Sort by block_time descending (newest first), unconfirmed txs at top
    all_txs.sort_by(|a, b| {
        let a_time = a.block_time.unwrap_or(u64::MAX);
        let b_time = b.block_time.unwrap_or(u64::MAX);
        b_time.cmp(&a_time)
    });

    // Keep only the 10 most recent
    all_txs.truncate(10);
    all_txs
}
