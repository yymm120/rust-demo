use std::fmt::Formatter;
use axum::response::{IntoResponse, Response};
use deadpool_postgres::PoolError;
use crate::error;

// step1: 定义AppError
/// message: 存储错误的文本
/// cause: 存储上游错误
/// types: 存储错误的类型
#[derive(Debug)]
pub struct AppError {
    pub message: Option<String>,
    pub cause: Option<Box<dyn std::error::Error>>,
    pub types: AppErrorType,
}

// step2: 实现AppError
/// new: 构造函数
/// from_err: 通过上游错误进行构造
/// from_str: 通过message进行构造
/// notfound: 未找到之类的错误
impl AppError {
    fn new(message: Option<String>, cause: Option<Box<dyn std::error::Error>>, types: AppErrorType) -> Self {
        Self { message, cause, types }
    }
    fn from_err(cause: Box<dyn std::error::Error>, types: AppErrorType) -> Self {
        Self::new(None, Some(cause), types)
    }
    fn from_str(msg: &str, types: AppErrorType) -> Self {
        Self::new(Some(msg.to_string()), None, types)
    }
    pub fn notfound_opt(message: Option<String>) -> Self {
        Self::new(message, None, AppErrorType::Notfound)
    }
    pub fn notfound_msg(msg: &str) -> Self {
        Self::notfound_opt(Some(msg.to_string()))
    }
    pub fn notfound() -> Self {
        Self::notfound_msg("没有找到符合条件的数据")
    }
}

// step3: 兼容标准库Error
// 标准库的Error实现了Display trait和Debug trait, 所以我们也要实现Display trait和Debug trait
impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for AppError {}

// step4: 为了实现所有错误统一有AppError处理，必须将其他相关的Error转换为AppError
// 最佳实践是From trait
// 4.1. 将连接池错误转为AppError
impl From<deadpool_postgres::PoolError> for AppError {
    fn from(value: PoolError) -> Self {
        Self::from_err(Box::new(value), AppErrorType::Db)
    }
}
// 4.2. 将数据库操作的错误转为AppError
impl From<tokio_postgres::Error> for AppError {
    fn from(err: tokio_postgres::Error) -> Self {
        Self::from_err(Box::new(err), AppErrorType::Db)
    }
}
// 4.3. 将模板操作的错误转换为AppError
impl From<askama::Error> for AppError {
    fn from(err: askama::Error) -> Self {
        Self::from_err(Box::new(err), AppErrorType::Template)
    }
}


// step5: 让AppError实现IntoResponse, 以便能让其作为axum的响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let msg = match self.message {
            None => "有错误发生".to_string(),
            Some(msg) => msg.clone(),
        };
        msg.into_response()
    }
}



#[derive(Debug)]
pub enum AppErrorType {
    Db,         // 数据库相关的错误
    Template,   // 模板渲染相关的错误
    Notfound    // 未找到错误
}


pub type Result<T> = std::result::Result<T, error::AppError>;



