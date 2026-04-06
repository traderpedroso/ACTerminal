use gpui::Hsla;

pub fn buy_market_gradient_top() -> Hsla {
    Hsla {
        h: 215.0 / 360.0,
        s: 1.0,
        l: 0.50,
        a: 0.50,
    }
}

pub fn buy_market_gradient_bottom() -> Hsla {
    Hsla {
        h: 220.0 / 360.0,
        s: 0.90,
        l: 0.20,
        a: 0.05,
    }
}

pub fn sell_market_gradient_top() -> Hsla {
    Hsla {
        h: 0.0 / 360.0,
        s: 0.90,
        l: 0.55,
        a: 0.50,
    }
}

pub fn sell_market_gradient_bottom() -> Hsla {
    Hsla {
        h: 0.0 / 360.0,
        s: 0.80,
        l: 0.18,
        a: 0.05,
    }
}

pub fn buy_pending_bg() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 0.0,
        a: 1.0,
    }
}

#[allow(dead_code)]
pub fn market_text() -> Hsla {
    Hsla {
        h: 0.0,
        s: 0.0,
        l: 1.0,
        a: 1.0,
    }
}

#[allow(dead_code)]
pub fn pending_text_blue() -> Hsla {
    Hsla {
        h: 210.0 / 360.0,
        s: 0.90,
        l: 0.65,
        a: 1.0,
    }
}

#[allow(dead_code)]
pub fn pending_text_red() -> Hsla {
    Hsla {
        h: 0.0 / 360.0,
        s: 0.75,
        l: 0.65,
        a: 1.0,
    }
}
