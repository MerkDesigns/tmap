use rusqlite::{params, Connection};
use tauri::{AppHandle, Manager};

const KEY_CLOCK_STYLE: &str = "clock_style";

fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
  let base = app
    .path()
    .app_data_dir()
    .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
  fs::create_dir_all(&base).map_err(|e| format!("failed to create app data dir: {e}"))?;
  Ok(base.join("tmap.sqlite"))
}

fn open_db(app: &AppHandle) -> Result<Connection, String> {
  let path = db_path(app)?;
  let conn = Connection::open(path).map_err(|e| format!("failed to open sqlite db: {e}"))?;
  conn
    .execute(
      "CREATE TABLE IF NOT EXISTS kv (
          key TEXT PRIMARY KEY NOT NULL,
          value TEXT NOT NULL
        )",
      [],
    )
    .map_err(|e| format!("failed to initialize sqlite schema: {e}"))?;
  Ok(conn)
}

fn load_key(app: &AppHandle, key: &str) -> Result<Option<String>, String> {
  let conn = open_db(app)?;
  let mut stmt = conn
    .prepare("SELECT value FROM kv WHERE key = ?1 LIMIT 1")
    .map_err(|e| format!("failed to prepare sqlite select: {e}"))?;
  let mut rows = stmt
    .query(params![key])
    .map_err(|e| format!("failed to query sqlite: {e}"))?;

  if let Some(row) = rows.next().map_err(|e| format!("failed to read sqlite row: {e}"))? {
    let value: String = row
      .get(0)
      .map_err(|e| format!("failed to decode sqlite value: {e}"))?;
    return Ok(Some(value));
  }

  Ok(None)
}

fn save_key(app: &AppHandle, key: &str, value: &str) -> Result<(), String> {
  let conn = open_db(app)?;
  conn
    .execute(
      "INSERT INTO kv (key, value) VALUES (?1, ?2)
       ON CONFLICT(key) DO UPDATE SET value = excluded.value",
      params![key, value],
    )
    .map_err(|e| format!("failed to write sqlite value: {e}"))?;
  Ok(())
}

#[tauri::command]
fn load_clock_style(app: AppHandle) -> Result<Option<String>, String> {
  load_key(&app, KEY_CLOCK_STYLE)
}

#[tauri::command]
fn save_clock_style(app: AppHandle, style: String) -> Result<(), String> {
  save_key(&app, KEY_CLOCK_STYLE, &style)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      load_clock_style,
      save_clock_style
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
