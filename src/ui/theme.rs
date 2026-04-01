use gpui::Hsla;

// ──────────────────────────────────────────────────────────────────────────────
// IceTrader colour palette for trade side signals
// ──────────────────────────────────────────────────────────────────────────────

/// Strong blue gradient start for BUY_MARKET trades (like chart fill).
pub fn buy_market_gradient_top() -> Hsla {
    Hsla {
        h: 215.0 / 360.0,
        s: 1.0,
        l: 0.50,
        a: 0.50,
    }
}

/// Strong blue gradient end for BUY_MARKET trades (fades to background).
pub fn buy_market_gradient_bottom() -> Hsla {
    Hsla {
        h: 220.0 / 360.0,
        s: 0.90,
        l: 0.20,
        a: 0.05,
    }
}

/// Strong red gradient start for SELL_MARKET trades.
pub fn sell_market_gradient_top() -> Hsla {
    Hsla {
        h: 0.0 / 360.0, // red
        s: 0.90,
        l: 0.55,
        a: 0.50,
    }
}

/// Strong red gradient end for SELL_MARKET trades (fades to background).
pub fn sell_market_gradient_bottom() -> Hsla {
    Hsla {
        h: 0.0 / 360.0, // red
        s: 0.80,
        l: 0.18,
        a: 0.05,
    }
}

/// Black background for BUY_PENDING rows.
pub fn buy_pending_bg() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 1.0,
    }
}

/// Text colour used on top of the strong market backgrounds (white-ish).
#[allow(dead_code)]
pub fn market_text() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 1.0,
    }
}

/// Blue text for BUY_PENDING rows.
#[allow(dead_code)]
pub fn pending_text_blue() -> Hsla {
    Hsla {
        h: 210.0 / 360.0,
        s: 0.90,
        l: 0.65,
        a: 1.0,
    }
}

/// Red text for SELL_PENDING rows.
#[allow(dead_code)]
pub fn pending_text_red() -> Hsla {
    Hsla {
        h: 0.0 / 360.0, // red
        s: 0.75,
        l: 0.65,
        a: 1.0,
    }
}
