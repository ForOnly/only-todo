mod migrations;

use std::sync::Mutex;

use rusqlite::Connection;

use crate::errors::AppError;
use crate::infrastructure::filesystem;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, AppError> {
        filesystem::ensure_dirs(app)?;
        let db_path = filesystem::db_path(app)?;
        let conn = Connection::open(db_path).map_err(|error| AppError::DbError {
            message: error.to_string(),
        })?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|error| AppError::DbError {
                message: error.to_string(),
            })?;
        migrations::run_migrations(&conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// 持锁执行闭包，不开启 SQL 事务（每条语句自动提交）。
    ///
    /// `Mutex` 不可重入：闭包内禁止再调 `with_conn` / `with_tx`，应使用传入的 `&Connection`。
    pub fn with_conn<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.conn.lock().map_err(|_| AppError::InternalError {
            message: "database lock poisoned".into(),
        })?;
        f(&conn)
    }

    /// 持锁并在同一 SQL 事务中执行闭包；失败回滚。
    ///
    /// `Mutex` 不可重入：闭包内禁止再调 `with_conn` / `with_tx`，应使用传入的 `&Connection`。
    pub fn with_tx<T, F>(&self, f: F) -> Result<T, AppError>
    where
        F: FnOnce(&Connection) -> Result<T, AppError>,
    {
        let conn = self.conn.lock().map_err(|_| AppError::InternalError {
            message: "database lock poisoned".into(),
        })?;
        let tx = conn
            .unchecked_transaction()
            .map_err(|error| AppError::DbError {
                message: format!("transaction begin failed: {error}"),
            })?;
        match f(&tx) {
            Ok(value) => {
                tx.commit().map_err(|error| AppError::DbError {
                    message: format!("transaction commit failed: {error}"),
                })?;
                Ok(value)
            }
            Err(error) => {
                if let Err(rollback_error) = tx.rollback() {
                    tracing::error!("transaction rollback failed: {rollback_error}");
                }
                Err(error)
            }
        }
    }
}
