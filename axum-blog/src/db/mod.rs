mod paginate;

use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::GenericClient;
use tokio_postgres::Statement;
use tokio_postgres::types::{FromSqlOwned, ToSql};
use crate::db::paginate::Paginate;
use crate::error::{AppError, Result};

// 预编译我们的Sql语句, 返回Statement对象
async fn get_stmt(client: &impl GenericClient, sql: &str) -> Result<Statement> {
    client.prepare(sql).await.map_err(AppError::from)
}


async fn query<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)]
) -> Result<Vec<T>>
where
    T: FromTokioPostgresRow,
{
    let stmt = get_stmt(client, sql).await?;
    let result = client
        .query(&stmt, params)
        .await
        .map_err(AppError::from)?
        .iter()
        .map(|row| <T>::from_row_ref(row).unwrap())
        .collect::<Vec<T>>();
    Ok(result)
}

async fn query_row_opt<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
    msg: Option<String>
) -> Result<T>
where
    T: FromTokioPostgresRow,
{
    query(client, sql, params)
        .await?
        .pop()
        .ok_or(AppError::notfound_opt(msg))
}

// 查询单条记录，并指定当记录不存在时，使用错误信息
async fn query_row_msg<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
    msg: &str,
) -> Result<T>
where
    T: FromTokioPostgresRow,
{
    query_row_opt(client, sql, params, Some(msg.to_string())).await
}

// 查询单条记录，并指定当记录不存在时，使用的默认错误信息
async fn query_row<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<T>
where
    T: FromTokioPostgresRow,
{
    query_row_opt(client, sql, params, None).await
}

/// 插入记录并返回指定数据
async fn insert<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
    msg: &str,
) -> Result<T>
where
    T: FromTokioPostgresRow,
{
    query_row_msg(client, sql, params, msg).await
}


// 查询单列数据
async fn query_col<T> (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<T>
where
    T: FromSqlOwned,
{
    let stmt = get_stmt(client, sql).await?;
    Ok(client
        .query_one(&stmt, params)
        .await
        .map_err(AppError::from)?
        .get(0))
}


async fn count (
    client: &impl GenericClient,
    sql: &str,
    params: &[&(dyn ToSql + Sync)],
) -> Result<i64> {
    query_col(client, sql, params).await
}

async fn execute(
    client: &impl GenericClient,
    sql: &str,
    args: &[&(dyn ToSql + Sync)],
) -> Result<u64> {
    let stmt = get_stmt(client, sql).await?;
    client.execute(&stmt, args).await.map_err(AppError::from)
}

// postgresql 的特殊性，对于insert:
// 1. 如果需要返回新插入的id，需要使用查询，并配合insert ... returning...
// 2. 如果不需要返回新插入的id，请使用execute

// postgresql 的Returning非常强大、方便(update/delete等sql语句中也能使用)
async fn pagination<T> (
    client: &impl GenericClient,
    sql: &str,
    count_sql: &str,
    params: &[&(dyn ToSql + Sync)],
    page: u32,
) -> Result<Paginate<Vec<T>>>
where
    T: FromTokioPostgresRow,
{
    let data = query(client, sql, params).await?;
    let total_records = count(client, count_sql, params).await?;
    Ok(Paginate::new(page, 5, total_records, data))
}







