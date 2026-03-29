use iced::{
    widget::{
        button, column, container, image, mouse_area, opaque, pick_list, row, scrollable, stack,
        text, Space,
    },
    Alignment, Background, Border, Color, Element, Length, Shadow, Theme,
};
use qrcode::{types::Color as QrColor, QrCode};
use std::fmt;

use crate::i18n::t;
use crate::theme::{
    card_style, color_with_alpha, pick_list_menu_style, pick_list_style, primary_button_style,
    secondary_button_style, text_color, Colors,
};
use crate::wallet::WalletEntry;

#[derive(Debug, Clone)]
pub enum ReceiveMessage {
    SelectWallet(usize),
    CopyAddress(String),
    ToggleQrCode(String),
    CloseQrPopup,
    DeriveNewAddress,
    SelectAddress(usize),
}

#[derive(Debug, Clone)]
pub enum ReceiveEvent {
    SelectWallet(usize),
    CopyAddress(String),
    DeriveAddresses(u32),
}

pub struct ReceiveView {
    selected_index: usize,
    copied: bool,
    show_qr: bool,
    qr_address: Option<String>,
    qr_handle: Option<image::Handle>,
    qr_error: Option<String>,
}

impl ReceiveView {
    pub fn new() -> Self {
        Self {
            selected_index: 0,
            copied: false,
            show_qr: false,
            qr_address: None,
            qr_handle: None,
            qr_error: None,
        }
    }

    pub fn update(&mut self, message: ReceiveMessage) -> Option<ReceiveEvent> {
        match message {
            ReceiveMessage::SelectWallet(index) => {
                self.selected_index = 0;
                self.copied = false;
                self.clear_qr_state();
                Some(ReceiveEvent::SelectWallet(index))
            }
            ReceiveMessage::CopyAddress(addr) => {
                self.copied = true;
                Some(ReceiveEvent::CopyAddress(addr))
            }
            ReceiveMessage::ToggleQrCode(address) => {
                let is_same_address = self.qr_address.as_deref() == Some(address.as_str());
                if self.show_qr && is_same_address {
                    self.close_qr_popup();
                    return None;
                }

                match build_qr_handle(&address) {
                    Ok(handle) => {
                        self.show_qr = true;
                        self.qr_address = Some(address);
                        self.qr_handle = Some(handle);
                        self.qr_error = None;
                    }
                    Err(err) => {
                        self.show_qr = false;
                        self.qr_error = Some(err);
                    }
                }
                None
            }
            ReceiveMessage::CloseQrPopup => {
                self.close_qr_popup();
                None
            }
            ReceiveMessage::DeriveNewAddress => {
                self.copied = false;
                self.clear_qr_state();
                Some(ReceiveEvent::DeriveAddresses(1))
            }
            ReceiveMessage::SelectAddress(index) => {
                self.selected_index = index;
                self.copied = false;
                self.clear_qr_state();
                None
            }
        }
    }

    pub fn view<'a>(
        &'a self,
        wallets: &'a [WalletEntry],
        selected_wallet: usize,
    ) -> Element<'a, ReceiveMessage> {
        let wallet_options = wallet_choices(wallets);
        let selected_wallet_option = selected_wallet_choice(wallets, selected_wallet);
        let wallet = wallets.get(selected_wallet);

        let title = text(t("Nhận BTC", "Receive BTC"))
            .size(32)
            .style(text_color(Colors::TEXT_PRIMARY));

        let wallet_selector = column![
            text(t("Ví", "Wallet"))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            Space::with_height(4),
            pick_list(wallet_options, selected_wallet_option, |choice| {
                ReceiveMessage::SelectWallet(choice.index)
            })
            .placeholder(t(
                "Chọn ví để nhận BTC...",
                "Select wallet to receive BTC..."
            ))
            .width(Length::Fill)
            .padding(12)
            .style(pick_list_style())
            .menu_style(pick_list_menu_style()),
        ]
        .spacing(4);

        let mut content = column![title, wallet_selector].spacing(16).padding(32);

        if let Some(wallet) = wallet {
            let balance: i64 = wallet.history.iter().map(|tx| tx.amount_sat).sum();
            let balance_btc = balance as f64 / 100_000_000.0;

            content = content.push(
                text(format!(
                    "{}: {:.8} BTC | {}: {}",
                    t("Số dư", "Balance"),
                    balance_btc,
                    t("Mạng", "Network"),
                    wallet.network.as_str()
                ))
                .size(14)
                .style(text_color(Colors::TEXT_SECONDARY)),
            );

            let derive_button =
                button(text(t("+ Tạo địa chỉ mới", "+ Derive New Address")).size(14))
                    .on_press(ReceiveMessage::DeriveNewAddress)
                    .padding(10)
                    .style(primary_button_style());

            content = content.push(derive_button);

            if wallet.addresses.is_empty() {
                content = content.push(
                    text(t(
                        "Ví chưa có địa chỉ, hãy bấm 'Tạo địa chỉ mới'.",
                        "This wallet has no address yet, click 'Derive New Address'.",
                    ))
                    .size(14)
                    .style(text_color(Colors::TEXT_MUTED)),
                );
            } else {
                let selected_index = self.selected_index.min(wallet.addresses.len() - 1);
                if let Some(addr) = wallet.addresses.get(selected_index) {
                    content = content.push(Space::with_height(12));
                    content = content.push(
                        text(t("Địa chỉ đang chọn:", "Selected Address:"))
                            .size(16)
                            .style(text_color(Colors::TEXT_PRIMARY)),
                    );
                    content = content.push(
                        container(
                            text(addr.address.clone())
                                .size(14)
                                .style(text_color(Colors::ACCENT_TEAL)),
                        )
                        .style(card_style())
                        .padding(16)
                        .width(Length::Fill),
                    );
                    let qr_visible_for_selected =
                        self.show_qr && self.qr_address.as_deref() == Some(addr.address.as_str());

                    content = content.push(
                        row![
                            button(
                                text(if self.copied {
                                    t("Đã copy!", "Copied!")
                                } else {
                                    t("Copy địa chỉ", "Copy Address")
                                })
                                .size(14),
                            )
                            .on_press(ReceiveMessage::CopyAddress(addr.address.clone()))
                            .padding(10)
                            .style(if self.copied {
                                secondary_button_style()
                            } else {
                                primary_button_style()
                            }),
                            Space::with_width(8),
                            button(
                                text(if qr_visible_for_selected {
                                    t("Ẩn QR", "Hide QR")
                                } else {
                                    t("Hiện QR", "Show QR")
                                })
                                .size(14),
                            )
                            .on_press(ReceiveMessage::ToggleQrCode(addr.address.clone()))
                            .padding(10)
                            .style(secondary_button_style()),
                        ]
                        .align_y(Alignment::Center),
                    );

                    if !qr_visible_for_selected && self.qr_error.is_some() {
                        let err = self
                            .qr_error
                            .as_deref()
                            .unwrap_or(t("Không tạo được QR", "Failed to generate QR"));
                        content = content.push(text(err).size(13).style(text_color(Colors::ERROR)));
                    }
                }

                content = content.push(Space::with_height(16));
                content = content.push(
                    text(t("Tất cả địa chỉ", "All Addresses"))
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                );

                let mut list = column![];
                for (i, addr) in wallet.addresses.iter().enumerate() {
                    let is_selected = i == selected_index;
                    let row_content = row![
                        text(format!("#{}", addr.index))
                            .size(12)
                            .style(text_color(Colors::TEXT_MUTED)),
                        Space::with_width(8),
                        text(addr.address.clone())
                            .size(11)
                            .style(text_color(Colors::TEXT_PRIMARY)),
                        Space::with_width(Length::Fill),
                        if is_selected {
                            text(t("Đang chọn", "Selected"))
                                .size(11)
                                .style(text_color(Colors::SUCCESS))
                        } else {
                            text("")
                        },
                    ]
                    .align_y(Alignment::Center);

                    list = list.push(
                        button(container(row_content).width(Length::Fill))
                            .on_press(ReceiveMessage::SelectAddress(i))
                            .padding(8)
                            .style(if is_selected {
                                primary_button_style()
                            } else {
                                secondary_button_style()
                            })
                            .width(Length::Fill),
                    );
                    list = list.push(Space::with_height(6));
                }

                content = content.push(scrollable(list).height(Length::Fill));
            }
        } else {
            content = content.push(
                text(t("Chưa chọn ví", "No wallet selected"))
                    .size(16)
                    .style(text_color(Colors::ERROR)),
            );
        }

        let base: Element<'a, ReceiveMessage> = container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();

        if let (true, Some(handle), Some(address)) = (
            self.show_qr,
            self.qr_handle.as_ref(),
            self.qr_address.as_ref(),
        ) {
            let popup = container(
                column![
                    text(t("QR Code nhận BTC", "BTC Receive QR Code"))
                        .size(18)
                        .style(text_color(Colors::TEXT_PRIMARY)),
                    text(address.as_str())
                        .size(12)
                        .style(text_color(Colors::TEXT_SECONDARY)),
                    Space::with_height(10),
                    container(
                        image::Image::new(handle.clone())
                            .width(Length::Fixed(240.0))
                            .height(Length::Fixed(240.0)),
                    )
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                    Space::with_height(10),
                    button(text(t("Đóng", "Close")).size(14))
                        .on_press(ReceiveMessage::CloseQrPopup)
                        .padding(10)
                        .style(primary_button_style()),
                ]
                .align_x(Alignment::Center)
                .spacing(6),
            )
            .style(card_style())
            .padding(18)
            .width(Length::Fixed(380.0));

            let backdrop = container(
                mouse_area(
                    container(Space::with_width(Length::Fill))
                        .width(Length::Fill)
                        .height(Length::Fill),
                )
                .on_press(ReceiveMessage::CloseQrPopup),
            )
            .style(qr_backdrop_style())
            .width(Length::Fill)
            .height(Length::Fill);

            let popup_layer = container(popup)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill);

            let overlay = stack(vec![opaque(backdrop).into(), popup_layer.into()])
                .width(Length::Fill)
                .height(Length::Fill);

            return stack(vec![base, overlay.into()])
                .width(Length::Fill)
                .height(Length::Fill)
                .into();
        }

        base
    }
}

impl ReceiveView {
    fn clear_qr_state(&mut self) {
        self.show_qr = false;
        self.qr_address = None;
        self.qr_handle = None;
        self.qr_error = None;
    }

    fn close_qr_popup(&mut self) {
        self.show_qr = false;
        self.qr_error = None;
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WalletChoice {
    index: usize,
    label: String,
}

impl fmt::Display for WalletChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

fn wallet_choices(wallets: &[WalletEntry]) -> Vec<WalletChoice> {
    wallets
        .iter()
        .enumerate()
        .map(|(index, wallet)| WalletChoice {
            index,
            label: format!("{} ({})", wallet.name, wallet.network.as_str()),
        })
        .collect()
}

fn selected_wallet_choice(wallets: &[WalletEntry], selected_wallet: usize) -> Option<WalletChoice> {
    wallets.get(selected_wallet).map(|wallet| WalletChoice {
        index: selected_wallet,
        label: format!("{} ({})", wallet.name, wallet.network.as_str()),
    })
}

fn build_qr_handle(address: &str) -> Result<image::Handle, String> {
    let qr = QrCode::new(address.as_bytes())
        .map_err(|err| format!("{}: {err}", t("Không tạo được QR", "Failed to generate QR")))?;

    let module_count = qr.width();
    let scale = 8usize;
    let border = 4usize;
    let side = (module_count + border * 2) * scale;

    let mut rgba = vec![255u8; side * side * 4];

    for y in 0..side {
        let module_y = y / scale;
        for x in 0..side {
            let module_x = x / scale;

            let is_dark = if module_x >= border
                && module_y >= border
                && module_x < border + module_count
                && module_y < border + module_count
            {
                qr[(module_x - border, module_y - border)] == QrColor::Dark
            } else {
                false
            };

            if is_dark {
                let offset = (y * side + x) * 4;
                rgba[offset] = 24;
                rgba[offset + 1] = 24;
                rgba[offset + 2] = 30;
                rgba[offset + 3] = 255;
            }
        }
    }

    Ok(image::Handle::from_rgba(side as u32, side as u32, rgba))
}

fn qr_backdrop_style() -> Box<dyn Fn(&Theme) -> container::Style> {
    Box::new(|_theme: &Theme| container::Style {
        background: Some(Background::Color(color_with_alpha(
            Colors::BG_PRIMARY,
            0.75,
        ))),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        shadow: Shadow::default(),
        text_color: None,
    })
}
