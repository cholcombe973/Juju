extern crate chrono;
extern crate rusqlite;
extern crate serde;
extern crate serde_json;

use std::collections::HashMap;
use std::env;
use std::iter;
use std::path::PathBuf;

use self::chrono::{DateTime, Utc};
use self::rusqlite::Connection;
use self::rusqlite::Error::QueryReturnedNoRows;
use self::serde::{Deserialize, Serialize};
use self::serde_json::Value;
use super::JujuError;

#[derive(Debug)]
/// A connection to the unit's Key/Value data
/// Simple key value database for local unit state within charms.
/// Values are automatically json encoded/decoded.
pub struct Storage {
    conn: Connection,
    revision: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    slots: HashMap<String, String>,
}

#[derive(Debug)]
pub struct History {
    data: Value,
    date: DateTime<Utc>,
    hook: String,
    key: String,
    revision: u32,
}

impl Storage {
    /// Connect to the unit's database
    pub fn new(path: Option<PathBuf>) -> Result<Self, JujuError> {
        let db_path = match path {
            Some(p) => p,
            None => {
                PathBuf::from(env::var("UNIT_STATE_DB")
                    .unwrap_or(format!("{}.unit-state.db", env::var("CHARM_DIR").unwrap())))
            }
        };

        let conn = Connection::open(db_path)?;
        let storage = Storage {
            conn: conn,
            revision: None,
        };
        storage.init()?;
        Ok(storage)
    }

    fn init(&self) -> Result<(), JujuError> {
        let _ = self.conn
            .execute("create table if not exists kv (key text,data text,primary key (key))",
                     &[]);
        let _ = self.conn.execute("
        create table if not exists kv_revisions (key text, revision integer, data text,
            primary key (key, revision))",
                                  &[]);
        let _ = self.conn.execute("create table if not exists hooks (version integer primary key \
                           autoincrement,
           hook text, date text )",
                                  &[]);
        Ok(())
    }

    pub fn get<T>(&self, key: &str) -> Result<Option<T>, JujuError>
        where T: Deserialize
    {
        let result: String = match self.conn
            .query_row("SELECT data from kv where key=?", &[&key], |row| row.get(0)) {
            Ok(result) => result,
            Err(e) => {
                match e {
                    QueryReturnedNoRows => {
                        return Ok(None);
                    }
                    _ => {
                        return Err(JujuError::RusqliteError(e));
                    }
                }
            }
        };
        Ok(Some(serde_json::from_str(&result)?))
    }

    /// Get a range of keys starting with a common prefix as a mapping of
    /// keys to values.
    pub fn getrange(&self,
                    key_prefix: &str,
                    strip: bool)
                    -> Result<HashMap<String, Value>, JujuError> {
        let mut results: HashMap<String, Value> = HashMap::new();
        let mut stmt = self.conn
            .prepare("select key, data from kv where key like ?")?;
        let mut rows = stmt.query(&[&format!("{}%", key_prefix)])?;

        while let Some(result_row) = rows.next() {
            let row = result_row?;
            let k: String = row.get(0);
            let v: String = row.get(1);
            let value = serde_json::from_str(&v)?;
            if strip {
                results.insert(k.trim_left_matches(&key_prefix).to_string(), value);
            } else {
                results.insert(k, value);
            }
        }

        Ok(results)
    }

    /// Set the values of multiple keys at once.
    /// Accepts an optional prefix to apply to all keys before setting
    pub fn update<T>(&self,
                     mapping: HashMap<String, T>,
                     prefix: Option<String>)
                     -> Result<(), JujuError>
        where T: Serialize
    {
        let prefix = prefix.unwrap_or("".to_string());
        for (k, v) in mapping {
            self.set(&format!("{}{}", prefix, k), v)?;
        }
        Ok(())
    }

    /// Remove a key from the database entirely.
    pub fn unset(&self, key: &str) -> Result<(), JujuError> {
        let rowcount = self.conn.execute("delete from kv where key=?", &[&key])?;
        if self.revision.is_some() && rowcount > 0 {
            self.conn
                .execute("insert into kv_revisions values (?, ?, ?)",
                         &[&key, &self.revision, &String::from("\"DELETED\"")])?;
        }
        Ok(())
    }

    /// Remove a range of keys starting with a common prefix, from the database
    /// entirely.  If keys is set to None it will delete all keys
    /// Returns number of rows deleted
    pub fn unsetrange(&self,
                      keys: Option<Vec<String>>,
                      prefix: Option<String>)
                      -> Result<u32, JujuError> {
        let deleted = String::from("\"DELETED\"");
        let revision = self.revision.clone().unwrap_or(0);
        let prefix = prefix.unwrap_or("".to_string());

        match keys {
            Some(keys) => {
                let mut question_marks = Vec::new();
                for _ in 0..keys.len() {
                    question_marks.push("?");
                }
                let mut values: Vec<&rusqlite::types::ToSql> = Vec::new();
                for key in &keys {
                    values.push(key);
                }
                let delete_query = format!("delete from kv where key in ({})",
                                           question_marks.join(","));
                let rowcount = self.conn
                    .execute(&delete_query, &values[..])?;

                if self.revision.is_some() && rowcount > 0 {
                    let field_list = iter::repeat("(?, ?, ?)")
                        .take(keys.len())
                        .collect::<Vec<_>>()
                        .join(",");
                    //key, self.revision, String::from("\"DELETED\""
                    let mut values: Vec<&rusqlite::types::ToSql> = Vec::new();
                    for key in &keys {
                        values.push(key);
                        values.push(&revision);
                        values.push(&deleted);
                    }
                    let _ = self.conn
                        .execute(&format!("insert into kv_revisions values {}", field_list),
                                 &values[..]);
                }
                Ok(rowcount as u32)
            }
            None => {
                let rowcount = self.conn
                    .execute("delete from kv where key like ?",
                             &[&format!("{}%", prefix)])?;
                if self.revision.is_some() && rowcount > 0 {
                    self.conn
                        .execute("insert into kv_revisions values (?, ?, ?)",
                                 &[&format!("{}%", prefix), &revision, &deleted])?;
                }
                Ok(rowcount as u32)
            }
        }
    }

    /// Set a value in the database.
    pub fn set<T>(&self, key: &str, value: T) -> Result<(), JujuError>
        where T: Serialize
    {
        let serialized = serde_json::to_string(&value)?;

        let exists: bool = self.conn
            .query_row("select exists(select data from kv where key=?)",
                       &[&key],
                       |row| row.get(0))?;
        match exists {
            true => {
                self.conn.execute("update kv set data = ? where key = ?", &[&serialized, &key])?;
            }
            false => {
                self.conn
                    .execute("insert into kv (key, data) values (?, ?)",
                             &[&key, &serialized])?;
            }
        };

        // Save
        if !self.revision.is_some() {
            return Ok(());
        }

        let exists_with_revision = self.conn
            .query_row("select exists(select 1 from kv_revisions where key=? and revision=?)",
                       &[&key, &self.revision],
                       |row| row.get(0))?;

        match exists_with_revision {
            true => {
                self.conn
                    .execute("update kv_revisions set data = ? where key = ? and revision = ?",
                             &[&serialized, &key, &self.revision])?;
            }
            false => {
                self.conn
                    .execute("insert into kv_revisions (revision, key, data) values (?, ?, ?)",
                             &[&self.revision, &key, &serialized])?;

            }
        };

        return Ok(());
    }
    /*
    ///Scope all future interactions to the current hook execution
    ///revision
    pub fn hook_scope(&mut self, name: &str) ->Result<String,JujuError>{
        assert!(self.revision.is_none());
        let _ = self.conn.execute("insert into hooks (hook, date) values (?, ?)",
            &[name or sys.argv[0],
             datetime.datetime.utcnow().isoformat()])?;
            self.revision = self.cursor.lastrowid;
            yield self.revision;
            self.revision = None
    }
    */

    pub fn gethistory(&self, key: &str) -> Result<Vec<History>, JujuError> {
        let mut results: Vec<History> = Vec::new();
        let mut stmt = self.conn
            .prepare("select kv.revision, kv.key, kv.data, h.hook, h.date from kv_revisions kv, \
                      hooks h where kv.key=? and kv.revision = h.version")?;
        let mut rows = stmt.query(&[&key])?;

        while let Some(result_row) = rows.next() {
            let row = result_row?;

            let revision: u32 = row.get(0);
            let key: String = row.get(1);
            let data: String = row.get(2);
            let hook: String = row.get(3);
            let date: DateTime<Utc> = row.get(4);

            results.push(History {
                data: serde_json::from_str(&data)?,
                date: date,
                key: key,
                hook: hook,
                revision: revision,
            });
        }
        Ok(results)
    }
}
