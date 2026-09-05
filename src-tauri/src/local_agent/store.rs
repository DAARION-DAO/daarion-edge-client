use super::folder::FolderReport;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Clone, Serialize, Deserialize)]
pub struct Profile {
    pub node_id: String,
    pub agent_id: String,
    pub name: String,
    pub purpose: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct CityTaskContext {
    pub request_hash: String,
    pub thread_id: String,
    pub run_id: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub created_at: String,
    pub status: String,
    pub model: Option<String>,
    pub report: Option<FolderReport>,
    pub answer: Option<String>,
    pub error: Option<String>,
    #[serde(default)]
    pub city: Option<CityTaskContext>,
}

pub struct Store(Connection);
fn failure(_: impl std::fmt::Display) -> String {
    "Локальне сховище недоступне.".into()
}
impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        let parent = path.parent().ok_or("Немає каталогу сховища.")?;
        std::fs::create_dir_all(parent).map_err(failure)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                .map_err(failure)?;
        }
        let db = Connection::open(path).map_err(failure)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))
                .map_err(failure)?;
        }
        db.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;
            CREATE TABLE IF NOT EXISTS profile (singleton INTEGER PRIMARY KEY CHECK(singleton=1), body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS task (id TEXT PRIMARY KEY, body TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS audit (sequence INTEGER PRIMARY KEY, event TEXT NOT NULL, at TEXT NOT NULL);") .map_err(failure)?;
        let store = Self(db);
        // A process that exited cannot leave an apparently running task after reopening.
        for mut task in store.tasks()? {
            if task.status == "running" {
                task.status = "interrupted".into();
                task.error = Some("Застосунок закрився до завершення завдання.".into());
                store.save_task(&task)?;
            }
        }
        Ok(store)
    }
    pub fn profile(&self) -> Result<Option<Profile>, String> {
        let body: Option<String> = self
            .0
            .query_row("SELECT body FROM profile WHERE singleton=1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(failure)?;
        body.map(|s| serde_json::from_str(&s).map_err(failure))
            .transpose()
    }
    pub fn create_profile(&self, name: String, purpose: String) -> Result<Profile, String> {
        let name = name.trim().to_string();
        let purpose = purpose.trim().to_string();
        if name.is_empty()
            || name.chars().count() > 80
            || purpose.chars().count() > 500
            || name.chars().any(char::is_control)
        {
            return Err("Вкажіть ім’я до 80 символів і опис до 500 символів.".into());
        }
        if self.profile()?.is_some() {
            return Err("Власний агент уже створений.".into());
        }
        let profile = Profile {
            node_id: uuid::Uuid::new_v4().to_string(),
            agent_id: uuid::Uuid::new_v4().to_string(),
            name,
            purpose,
        };
        self.0
            .execute(
                "INSERT INTO profile(singleton,body) VALUES(1,?1)",
                [serde_json::to_string(&profile).map_err(failure)?],
            )
            .map_err(failure)?;
        self.audit("profile_created")?;
        Ok(profile)
    }
    pub fn tasks(&self) -> Result<Vec<Task>, String> {
        let mut statement = self
            .0
            .prepare("SELECT body FROM task ORDER BY rowid DESC LIMIT 50")
            .map_err(failure)?;
        let tasks = statement
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(failure)?
            .map(|s| serde_json::from_str(&s.map_err(failure)?).map_err(failure))
            .collect();
        tasks
    }
    pub fn task(&self, id: &str) -> Result<Option<Task>, String> {
        let body: Option<String> = self
            .0
            .query_row("SELECT body FROM task WHERE id=?1", [id], |r| r.get(0))
            .optional()
            .map_err(failure)?;
        body.map(|s| serde_json::from_str(&s).map_err(failure))
            .transpose()
    }
    pub fn save_task(&self, task: &Task) -> Result<(), String> {
        self.0.execute("INSERT INTO task(id,body) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET body=excluded.body", params![task.id, serde_json::to_string(task).map_err(failure)?]).map_err(failure)?;
        Ok(())
    }
    pub fn audit(&self, event: &str) -> Result<(), String> {
        self.0
            .execute(
                "INSERT INTO audit(event,at) VALUES(?1,?2)",
                params![event, chrono::Utc::now().to_rfc3339()],
            )
            .map_err(failure)?;
        self.0
            .execute(
                "DELETE FROM audit WHERE sequence <= (SELECT MAX(sequence)-500 FROM audit)",
                [],
            )
            .map_err(failure)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profile_and_actual_result_survive_restart_and_running_task_is_interrupted() {
        let dir = std::env::temp_dir().join(format!("edge-store-{}", uuid::Uuid::new_v4()));
        let path = dir.join("pilot.sqlite3");
        let db = Store::open(&path).unwrap();
        let profile = db
            .create_profile("Працівник".into(), "Локальна робота".into())
            .unwrap();
        assert_ne!(profile.agent_id, profile.node_id);
        assert!(db
            .create_profile("Duplicate".into(), String::new())
            .is_err());
        let mut task = Task {
            id: "one".into(),
            created_at: "now".into(),
            status: "completed".into(),
            model: None,
            report: None,
            answer: Some("saved result".into()),
            error: None,
            city: None,
        };
        db.save_task(&task).unwrap();
        db.save_task(&task).unwrap();
        task.id = "two".into();
        task.status = "running".into();
        db.save_task(&task).unwrap();
        drop(db);
        let db = Store::open(&path).unwrap();
        assert_eq!(db.profile().unwrap().unwrap().agent_id, profile.agent_id);
        let tasks = db.tasks().unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].status, "interrupted");
        assert_eq!(tasks[1].answer.as_deref(), Some("saved result"));
        drop(db);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
