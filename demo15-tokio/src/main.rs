use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{watch, RwLock};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub struct SqlxMapping {
    pub oid: u32,
    pub table_name: String,
}

pub struct AppState {
    mappings: RwLock<HashMap<u32, SqlxMapping>>,
    change_notifier: watch::Sender<()>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        let (tx, _) = watch::channel(());
        Arc::new(Self {
            mappings: RwLock::new(HashMap::new()),
            change_notifier: tx,
        })
    }

    pub async fn update_mapping(&self, mapping: SqlxMapping) {
        println!("Updating mapping: {}", mapping.table_name); // 添加调试输出
        let mut write_guard = self.mappings.write().await;
        write_guard.insert(mapping.oid, mapping);
        if let Err(e) = self.change_notifier.send(()) {
            println!("Failed to send notification: {}", e);
        }
    }

    pub async fn get_all(&self) -> HashMap<u32, SqlxMapping> {
        let read_guard = self.mappings.read().await;
        read_guard.clone()
    }

    pub fn subscribe_changes(&self) -> watch::Receiver<()> {
        self.change_notifier.subscribe()
    }
}

async fn watch_for_changes(state: Arc<AppState>) {
    println!("Listener started"); // 添加调试输出
    let mut rx = state.subscribe_changes();

    // 明确跳过初始通知
    if rx.changed().await.is_err() {
        println!("Initial channel error");
        return;
    }

    loop {
        println!("Waiting for changes..."); // 添加调试输出
        if rx.changed().await.is_err() {
            println!("Sender dropped, exiting listener");
            break;
        }

        let mappings = state.get_all().await;
        println!("Current mappings: {:?}",
                 mappings.values().map(|m| &m.table_name).collect::<Vec<_>>());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Program started"); // 添加调试输出

    let app_state = AppState::new();

    // 启动监听任务
    let listener = tokio::spawn(watch_for_changes(app_state.clone()));

    // 确保监听任务已启动
    sleep(Duration::from_millis(100)).await;

    // 测试更新
    app_state.update_mapping(SqlxMapping {
        oid: 1,
        table_name: "users".to_string(),
    }).await;

    // 等待监听任务处理
    sleep(Duration::from_millis(1000)).await;

    println!("Main thread ending"); // 添加调试输出
    listener.abort();

    Ok(())
}