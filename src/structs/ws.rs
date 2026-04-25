pub enum WsEvent {
    StockCompleted,
    StockFailed,
    DataRefreshed,
}

impl WsEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::StockCompleted => "stock_completed",
            Self::StockFailed => "stock_failed",
            Self::DataRefreshed => "data_refreshed",
        }
    }
}
