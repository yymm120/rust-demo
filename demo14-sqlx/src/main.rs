use sqlx::{Connection, FromRow};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgBindIterExt;

#[derive(Debug, Serialize, Deserialize, FromRow)]
struct User {}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    Ok(())
}

#[cfg(test)]
mod test {
    use sqlx::{Column, FromRow, Row, SqlSafeStr};

    /// 基础查询
    #[tokio::test]
    async fn test_query() -> Result<(), sqlx::Error> {
        let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;

        // 带参数绑定的查询
        sqlx::query("INSERT INTO users (name) VALUES ($1)")
            .bind("Alice")
            .execute(&pool)
            .await?;

        Ok(())
    }

    /// 类型化查询
    #[tokio::test]
    async fn test_query_as() -> Result<(), sqlx::Error> {
        #[derive(Debug, FromRow)]
        struct User {
            id: i32,
            name: String,
        }

        let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;

        // 查询并映射到结构体
        let user: User = sqlx::query_as::<_, User>("SELECT id, name FROM users WHERE id = $1")
            .bind(1)
            .fetch_one(&pool)
            .await?;

        Ok(())
    }

    /// 流式查询
    // #[tokio::test]
    // async fn test_fetch_stream() -> Result<(), sqlx::Error> {
    //     use futures::TryStreamExt;
    //
    //     let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;
    //
    //     let mut stream = sqlx::query("SELECT id, name FROM users")
    //         .fetch(&pool);
    //
    //     while let Some(row) = stream.try_next().await? {
    //         let id: i32 = row.get("id");
    //         let name: &str = row.get("name");
    //     }
    //
    //     Ok(())
    // }

    /// 事务处理
    #[tokio::test]
    async fn test_transaction() -> Result<(), sqlx::Error> {
        let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;

        let mut tx = pool.begin().await?;

        sqlx::query("INSERT INTO users (name) VALUES ($1)")
            .bind("Bob")
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }


    /// 批量操作
    // #[tokio::test]
    // async fn test_batch_operations() -> Result<(), sqlx::Error> {
    //     let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;
    //
    //     // 批量插入
    //     let names = vec!["Charlie", "David"];
    //     let mut query = sqlx::query("INSERT INTO users (name) VALUES ($1)");
    //
    //     for name in names {
    //         query.bind(name).execute(&pool).await?;
    //     }
    //
    //     Ok(())
    // }

    /// 自定义类型处理
    #[tokio::test]
    async fn test_custom_types() -> Result<(), sqlx::Error> {
        use sqlx::{Type, Encode, Decode};
        use std::fmt;

        #[derive(Debug, Clone)]
        struct Email(String);

        impl fmt::Display for Email {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl Type<sqlx::Postgres> for Email {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                <String as Type<sqlx::Postgres>>::type_info()
            }
        }

        impl Encode<'_, sqlx::Postgres> for Email {
            fn encode_by_ref(&self, buf: &mut sqlx::postgres::PgArgumentBuffer) -> sqlx::encode::IsNull {
                self.0.encode_by_ref(buf).expect("REASON")
            }
        }

        impl Decode<'_, sqlx::Postgres> for Email {
            fn decode(value: sqlx::postgres::PgValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
                Ok(Email(String::decode(value)?))
            }
        }

        let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;

        let email = Email("test@example.com".to_string());
        sqlx::query("INSERT INTO users (email) VALUES ($1)")
            .bind(email)
            .execute(&pool)
            .await?;

        Ok(())
    }

    /// 行数据处理
    #[tokio::test]
    async fn test_row_handling() -> Result<(), sqlx::Error> {
        let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;

        let row = sqlx::query("SELECT id, name FROM users LIMIT 1")
            .fetch_one(&pool)
            .await?;

        // 按索引获取
        let id: i32 = row.get(0);
        // 按列名获取
        let name: String = row.get("name");

        // 遍历所有列
        for column in row.columns() {
            println!("Column: {}, type: {:?}", column.name(), column.type_info());
        }

        Ok(())
    }


    // #[tokio::test]
    // async fn test_dynamic_query() -> Result<(), sqlx::Error> {
    //     let pool = sqlx::postgres::PgPool::connect("postgres://user:pass@localhost/db").await?;
    //
    //     let sql = "SELECT * FROM users WHERE 1=1".to_string();
    //
    //     // 三种修复方式任选其一：
    //     // let mut query = sqlx::query(&*sql);      // 方案1
    //     // let mut query = sqlx::query(sql.as_str());  // 方案2
    //     let  query = sqlx::query(sql.into_sql_str());     // 方案3
    //
    //     // if true { query = query.bind("Alice"); }
    //     // if true { query = query.bind(10); }
    //     //
    //     let _rows = query.fetch_all(&pool).await?;
    //
    //     println!("Sql: {:?}", &_rows);
    //     //
    //     Ok(())
    // }

}
