use ratatui::style::{Color, Style};

/// Minimum USD quote in raw units (6 decimals) to show at full opacity.
/// $0.0001 = 100 raw units.
pub const USD_DUST_THRESHOLD: u128 = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AmountStyle {
    Normal,
    Dust,
}

pub struct DisplayAmount {
    pub text: String,
    pub style: AmountStyle,
}

impl DisplayAmount {
    pub fn ratatui_style(&self) -> Style {
        match self.style {
            AmountStyle::Normal => Style::default(),
            AmountStyle::Dust => Style::default().fg(Color::DarkGray),
        }
    }
}

pub fn format_usd(raw: &str) -> DisplayAmount {
    format_amount(raw, 6, 2, true)
}

/// Formats a quote in the selected display currency, using that fiat asset's
/// decimals and symbol; falls back to USD formatting when it is unknown.
pub fn format_quote(raw: &str, display_asset: Option<&koi::models::asset::Asset>) -> DisplayAmount {
    let Some(asset) = display_asset else {
        return format_usd(raw);
    };
    if asset.asset_symbol == "$" && asset.asset_decimals == 6 {
        return format_usd(raw);
    }

    let precision = 2.min(asset.asset_decimals);
    let inner = format_amount(raw, asset.asset_decimals, precision, false);
    DisplayAmount {
        text: format!("{} {}", inner.text, asset.asset_symbol),
        style: inner.style,
    }
}

pub fn format_token(raw: &str, decimals: u8) -> DisplayAmount {
    let Ok(value) = raw.parse::<u128>() else {
        return DisplayAmount {
            text: raw.to_string(),
            style: AmountStyle::Normal,
        };
    };

    if value == 0 {
        return DisplayAmount {
            text: "0".to_string(),
            style: AmountStyle::Dust,
        };
    }

    let precision = token_precision(value, decimals);
    format_amount(raw, decimals, precision, false)
}

fn token_precision(value: u128, decimals: u8) -> u8 {
    let scale = 10u128.pow(decimals as u32);
    let whole = value / scale;
    if whole >= 1_000_000 {
        2
    } else if whole >= 1 {
        4
    } else {
        decimals.min(6)
    }
}

fn format_amount(raw: &str, decimals: u8, precision: u8, currency: bool) -> DisplayAmount {
    let Ok(value) = raw.parse::<u128>() else {
        return DisplayAmount {
            text: raw.to_string(),
            style: AmountStyle::Normal,
        };
    };

    if value == 0 {
        return DisplayAmount {
            text: if currency {
                "$0.00".to_string()
            } else {
                "0".to_string()
            },
            style: AmountStyle::Dust,
        };
    }

    let is_dust = currency && value < USD_DUST_THRESHOLD;
    let style = if is_dust {
        AmountStyle::Dust
    } else {
        AmountStyle::Normal
    };

    if decimals < precision {
        return DisplayAmount {
            text: raw.to_string(),
            style,
        };
    }

    let scale = 10u128.pow(decimals as u32);
    let trim = 10u128.pow((decimals - precision) as u32);
    let half = trim / 2;
    let rounded = value.saturating_add(half);

    let whole = rounded / scale;
    let fraction = (rounded % scale) / trim;

    let min_display = trim / 2;
    if value > 0 && value < min_display {
        let threshold_text = format_fraction(0, precision, currency);
        return DisplayAmount {
            text: if currency {
                format!("<{threshold_text}")
            } else {
                format!("<{threshold_text}")
            },
            style: AmountStyle::Dust,
        };
    }

    DisplayAmount {
        text: format_fraction_with_whole(whole, fraction, precision, currency),
        style,
    }
}

fn format_fraction_with_whole(
    whole: u128,
    fraction: u128,
    precision: u8,
    currency: bool,
) -> String {
    let whole_str = format_with_commas(whole);
    let precision = precision as usize;
    let frac_str = format!("{fraction:0precision$}");
    if currency {
        format!("${whole_str}.{frac_str}")
    } else if precision == 0 {
        whole_str
    } else {
        format!("{whole_str}.{frac_str}")
    }
}

fn format_fraction(whole: u128, precision: u8, currency: bool) -> String {
    format_fraction_with_whole(whole, 0, precision, currency)
}

fn format_with_commas(value: u128) -> String {
    let s = value.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    let rem = bytes.len() % 3;
    if rem > 0 {
        out.push_str(&s[..rem]);
        if bytes.len() > rem {
            out.push(',');
        }
    }
    for (index, chunk) in bytes[rem..].chunks(3).enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(std::str::from_utf8(chunk).unwrap_or(""));
    }
    out
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeDirection {
    Up,
    Down,
    Flat,
}

pub struct PercentChange {
    pub text: String,
    pub direction: ChangeDirection,
}

impl PercentChange {
    pub fn ratatui_style(&self) -> Style {
        match self.direction {
            ChangeDirection::Up => Style::default().fg(Color::Green),
            ChangeDirection::Down => Style::default().fg(Color::Red),
            ChangeDirection::Flat => Style::default().fg(Color::DarkGray),
        }
    }

    pub fn label(&self) -> String {
        let arrow = match self.direction {
            ChangeDirection::Up => "▲",
            ChangeDirection::Down => "▼",
            ChangeDirection::Flat => "—",
        };
        format!("{arrow} {}", self.text)
    }
}

/// Percentage change from a 24h-ago quote to the current quote, matching the web UI.
pub fn percent_change(current: &str, previous_24h: &str) -> Option<PercentChange> {
    let current = current.parse::<u128>().ok()?;
    let previous = previous_24h.parse::<u128>().ok()?;
    if previous == 0 {
        return Some(PercentChange {
            text: "0.00%".to_string(),
            direction: ChangeDirection::Flat,
        });
    }

    let delta = current as i128 - previous as i128;
    let scaled = (delta * 10_000) / previous as i128;
    let direction = if scaled > 0 {
        ChangeDirection::Up
    } else if scaled < 0 {
        ChangeDirection::Down
    } else {
        ChangeDirection::Flat
    };

    Some(PercentChange {
        text: format!("{:.2}%", scaled as f64 / 100.0),
        direction,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_usd_with_commas_and_precision() {
        let formatted = format_usd("780133990000");
        assert_eq!(formatted.text, "$780,133.99");
        assert_eq!(formatted.style, AmountStyle::Normal);
    }

    #[test]
    fn marks_dust_usd() {
        let formatted = format_usd("50");
        assert_eq!(formatted.style, AmountStyle::Dust);
    }

    #[test]
    fn formats_quotes_with_the_display_asset_decimals_and_symbol() {
        let euro: koi::models::asset::Asset = serde_json::from_value(serde_json::json!({
            "asset_identity": "fiat:eur",
            "asset_name": "Euro",
            "asset_symbol": "€",
            "asset_decimals": 2,
            "asset_icon_url": null,
        }))
        .unwrap();

        assert_eq!(format_quote("123456", Some(&euro)).text, "1,234.56 €");
        assert_eq!(format_quote("780133990000", None).text, "$780,133.99");
    }

    #[test]
    fn computes_percent_change() {
        let change = percent_change("1100000", "1000000").unwrap();
        assert_eq!(change.text, "10.00%");
        assert_eq!(change.direction, ChangeDirection::Up);

        let change = percent_change("900000", "1000000").unwrap();
        assert_eq!(change.text, "-10.00%");
        assert_eq!(change.direction, ChangeDirection::Down);
    }
}
