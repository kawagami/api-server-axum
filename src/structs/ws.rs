pub enum WsEvent {
    StockCompleted,
    StockFailed,
}

impl WsEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StockCompleted => "stock_completed",
            Self::StockFailed => "stock_failed",
        }
    }
}
