#[derive(Debug, Clone, Copy, Default)]
pub struct PaginationQuery {
    pub current: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Pagination {
    pub offset: u32,
    pub limit: u32,
}

impl Pagination {
    pub fn from_query(q: PaginationQuery) -> Self {
        let page = q.current.unwrap_or(1).max(1) as u64;
        let size = q.page_size.unwrap_or(10).clamp(1, 100) as u64;
        let offset = page.saturating_sub(1).saturating_mul(size).min(u32::MAX as u64);

        Self { offset: offset as u32, limit: size as u32 }
    }
}
