use sqlx::PgPool;
use sqlx::postgres::PgListener;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum TableChangeEvent {
    Insert(serde_json::Value),
    Update(serde_json::Value),
    Delete(serde_json::Value),
}

pub async fn start_table_change_listener(
    pool: PgPool,
) -> Result<mpsc::Receiver<TableChangeEvent>, Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let mut listener = PgListener::connect_with(&pool).await.unwrap();
        listener.listen("sqlx_mapping_changes").await.unwrap();

        while let Ok(notification) = listener.recv().await {
            println!("notification: {:?}", notification);
        }
    });

    Ok(rx)
}

async fn create_trigger(pool: &PgPool) -> Result<(), sqlx::Error> {
   let a = sqlx::query(r#"
-- 创建通知函数
CREATE OR REPLACE FUNCTION notify_sqlx_mapping_change()
RETURNS TRIGGER AS $$
DECLARE
    notification JSON;
BEGIN
    -- 构建通知内容
    notification = json_build_object(
        'event', TG_OP,
        'oid', COALESCE(NEW.oid, OLD.oid),
        'old', OLD,
        'new', NEW
    );

    -- 发送通知到特定频道
    PERFORM pg_notify('sqlx_mapping_changes', notification::text);
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
"#).execute(pool).await?;

    sqlx::query(
        r#"
        DO $$
        BEGIN
            IF NOT EXISTS (
                SELECT 1 FROM pg_trigger
                WHERE tgname = 'trg_sqlx_mapping_notify'
                AND tgrelid = '_sqlx_mapping'::regclass
            ) THEN
                CREATE TRIGGER trg_sqlx_mapping_notify
                AFTER INSERT OR UPDATE OR DELETE ON _sqlx_mapping
                FOR EACH ROW EXECUTE FUNCTION notify_sqlx_mapping_change();
            END IF;
        END
        $$
        "#
    )
        .execute(pool)
        .await?;
    Ok(())
}

async fn watch_table_changes(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut listener = PgListener::connect_with(pool).await?;
    listener.listen("sqlx_mapping_changes").await?;  // 监听特定通道

    loop {
        let notification = listener.recv().await?;
        println!("into");
        println!("表变更通知: {}", notification.payload());
        // 解析payload获取具体变更信息
    }
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new().max_connections(5).connect("postgres://postgres:postgres@192.168.10.107/paotui").await?;

    create_trigger(&pool).await?;

    watch_table_changes(&pool).await?;

    Ok(())
}