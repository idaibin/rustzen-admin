/// 分页参数转换工具
pub struct Pagination;

impl Pagination {
    /// 标准化分页参数，返回 (page, limit, offset)
    pub fn normalize(current: Option<i64>, page_size: Option<i64>) -> (i64, i64, i64) {
        let page = current.unwrap_or(1).max(1);
        let limit = page_size.unwrap_or(10).min(100).max(1);
        let offset = (page - 1) * limit;
        (limit, offset, page)
    }
}
