use gpui::Hsla;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TradeSide {
    BuyMarket,
    SellMarket,
    BuyPending,
    SellPending,
    #[default]
    Mid,
}

impl TradeSide {
    pub fn from_str(s: &str) -> Self {
        match s {
            "BUY_MARKET" => TradeSide::BuyMarket,
            "SELL_MARKET" => TradeSide::SellMarket,
            "BUY_PENDING" => TradeSide::BuyPending,
            "SELL_PENDING" => TradeSide::SellPending,
            _ => TradeSide::Mid,
        }
    }

    pub fn text_color(&self) -> Hsla {
        match self {
            TradeSide::BuyMarket => market_text(),
            TradeSide::SellMarket => market_text(),
            TradeSide::BuyPending => pending_text_blue(),
            TradeSide::SellPending => pending_text_red(),
            TradeSide::Mid => Hsla {
                h: 0.0,
                s: 0.0,
                l: 0.55,
                a: 1.0,
            },
        }
    }
}

fn market_text() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 1.0,
    }
}

fn pending_text_blue() -> Hsla {
    Hsla {
        h: 210.0 / 360.0,
        s: 0.90,
        l: 0.65,
        a: 1.0,
    }
}

fn pending_text_red() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.75,
        l: 0.65,
        a: 1.0,
    }
}
