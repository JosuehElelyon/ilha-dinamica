use crate::domain::{
    AccountRecord, CanonicalEvent, ImportantDate, PersonRecord, PhoneDevice, PhoneSignalRecord,
    PhoneTransport, ProviderKind, UpcomingDate,
};
use chrono::{Datelike, Local, NaiveDate};
use rusqlite::{params, Connection};
use std::path::Path;

pub struct PersonalMemoryDb {
    conn: Connection,
}

impl PersonalMemoryDb {
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        Self::init(conn)
    }

    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| e.to_string())?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, String> {
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA foreign_keys = ON;

            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                label TEXT NOT NULL,
                email TEXT NOT NULL DEFAULT '',
                is_primary INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'connected',
                last_sync_ts INTEGER NOT NULL DEFAULT 0,
                details TEXT NOT NULL DEFAULT ''
            );

            CREATE TABLE IF NOT EXISTS people (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                relation TEXT NOT NULL DEFAULT '',
                email TEXT NOT NULL DEFAULT '',
                phone TEXT NOT NULL DEFAULT '',
                source_provider TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS important_dates (
                id TEXT PRIMARY KEY,
                person_id TEXT NULL REFERENCES people(id) ON DELETE SET NULL,
                person_name TEXT NOT NULL DEFAULT '',
                kind TEXT NOT NULL,
                month INTEGER NOT NULL,
                day INTEGER NOT NULL,
                year_optional INTEGER NULL,
                label TEXT NOT NULL,
                source_provider TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS canonical_events (
                id TEXT PRIMARY KEY,
                fingerprint TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                category TEXT NOT NULL,
                date TEXT NOT NULL,
                start_time TEXT NOT NULL DEFAULT '',
                end_time TEXT NOT NULL DEFAULT '',
                start_ts INTEGER NOT NULL,
                end_ts INTEGER NOT NULL,
                all_day INTEGER NOT NULL DEFAULT 0,
                join_url TEXT NOT NULL DEFAULT '',
                source_provider TEXT NOT NULL,
                account_id TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS phone_devices (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                platform TEXT NOT NULL,
                connected INTEGER NOT NULL DEFAULT 0,
                active_transport TEXT NOT NULL DEFAULT 'lan',
                battery_pct INTEGER NULL,
                charging INTEGER NOT NULL DEFAULT 0,
                last_seen_ts INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS phone_signals (
                id TEXT PRIMARY KEY,
                device_id TEXT NOT NULL,
                kind TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                app_name TEXT NOT NULL DEFAULT '',
                ts INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notification_log (
                key TEXT PRIMARY KEY,
                fired_at INTEGER NOT NULL,
                action_taken TEXT NOT NULL DEFAULT ''
            );
            ",
        )
        .map_err(|e| e.to_string())?;

        let db = Self { conn };
        db.seed_commemorative_dates()?;
        Ok(db)
    }

    fn seed_commemorative_dates(&self) -> Result<(), String> {
        let holidays: &[(&str, u32, u32, &str)] = &[
            ("hol-01-01", 1, 1, "Ano Novo"),
            ("hol-04-21", 4, 21, "Tiradentes"),
            ("hol-05-01", 5, 1, "Dia do Trabalho"),
            ("hol-06-12", 6, 12, "Dia dos Namorados"),
            ("hol-09-07", 9, 7, "Independência do Brasil"),
            ("hol-10-12", 10, 12, "Nossa Sra. Aparecida / Dia das Crianças"),
            ("hol-11-02", 11, 2, "Finados"),
            ("hol-11-15", 11, 15, "Proclamação da República"),
            ("hol-11-20", 11, 20, "Consciência Negra"),
            ("hol-12-25", 12, 25, "Natal"),
            ("hol-12-31", 12, 31, "Véspera de Ano Novo"),
        ];
        for (id, m, d, label) in holidays {
            self.conn
                .execute(
                    "INSERT OR IGNORE INTO important_dates
                     (id, person_id, person_name, kind, month, day, year_optional, label, source_provider)
                     VALUES (?1, NULL, '', 'holiday', ?2, ?3, NULL, ?4, 'local')",
                    params![id, *m as i64, *d as i64, label],
                )
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn upsert_account(&self, acc: &AccountRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO accounts (id, provider, label, email, is_primary, status, last_sync_ts, details)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                   label=excluded.label,
                   email=excluded.email,
                   is_primary=excluded.is_primary,
                   status=excluded.status,
                   last_sync_ts=excluded.last_sync_ts,
                   details=excluded.details",
                params![
                    acc.id,
                    acc.provider.as_str(),
                    acc.label,
                    acc.email,
                    acc.is_primary as i64,
                    acc.status,
                    acc.last_sync_ts,
                    acc.details
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn remove_account(&self, account_id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM accounts WHERE id = ?1", params![account_id])
            .map_err(|e| e.to_string())?;
        self.conn
            .execute(
                "DELETE FROM canonical_events WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_accounts(&self) -> Result<Vec<AccountRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, provider, label, email, is_primary, status, last_sync_ts, details
                 FROM accounts ORDER BY is_primary DESC, provider ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                let prov: String = r.get(1)?;
                Ok(AccountRecord {
                    id: r.get(0)?,
                    provider: ProviderKind::from_str(&prov),
                    label: r.get(2)?,
                    email: r.get(3)?,
                    is_primary: r.get::<_, i64>(4)? != 0,
                    status: r.get(5)?,
                    last_sync_ts: r.get(6)?,
                    details: r.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn upsert_person(&self, p: &PersonRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO people (id, name, relation, email, phone, source_provider)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name,
                   relation=CASE WHEN excluded.relation != '' THEN excluded.relation ELSE people.relation END,
                   email=CASE WHEN excluded.email != '' THEN excluded.email ELSE people.email END,
                   phone=CASE WHEN excluded.phone != '' THEN excluded.phone ELSE people.phone END",
                params![p.id, p.name, p.relation, p.email, p.phone, p.source_provider.as_str()],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn people_count(&self) -> Result<usize, String> {
        let c: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM people", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        Ok(c.max(0) as usize)
    }

    pub fn upsert_important_date(&self, d: &ImportantDate) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO important_dates
                 (id, person_id, person_name, kind, month, day, year_optional, label, source_provider)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT(id) DO UPDATE SET
                   person_name=excluded.person_name,
                   kind=excluded.kind,
                   month=excluded.month,
                   day=excluded.day,
                   year_optional=excluded.year_optional,
                   label=excluded.label",
                params![
                    d.id,
                    d.person_id,
                    d.person_name,
                    d.kind,
                    d.month as i64,
                    d.day as i64,
                    d.year_optional,
                    d.label,
                    d.source_provider.as_str()
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn delete_important_date(&self, id: &str) -> Result<(), String> {
        self.conn
            .execute("DELETE FROM important_dates WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn replace_account_events(
        &self,
        account_id: &str,
        events: &[CanonicalEvent],
    ) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM canonical_events WHERE account_id = ?1",
                params![account_id],
            )
            .map_err(|e| e.to_string())?;
        for ev in events {
            self.upsert_event(ev)?;
        }
        Ok(())
    }

    pub fn upsert_event(&self, ev: &CanonicalEvent) -> Result<(), String> {
        // Deduplica eventos idênticos vindos de fontes diferentes (ex.: Google + Windows Calendar)
        self.conn
            .execute(
                "INSERT INTO canonical_events
                 (id, fingerprint, title, category, date, start_time, end_time, start_ts, end_ts, all_day, join_url, source_provider, account_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
                 ON CONFLICT(fingerprint) DO UPDATE SET
                   join_url=CASE WHEN canonical_events.join_url = '' THEN excluded.join_url ELSE canonical_events.join_url END,
                   end_ts=excluded.end_ts,
                   end_time=excluded.end_time",
                params![
                    ev.id,
                    ev.fingerprint,
                    ev.title,
                    ev.category,
                    ev.date,
                    ev.start_time,
                    ev.end_time,
                    ev.start_ts,
                    ev.end_ts,
                    ev.all_day as i64,
                    ev.join_url,
                    ev.source_provider.as_str(),
                    ev.account_id
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_events_for_date(&self, ymd: &str) -> Result<Vec<CanonicalEvent>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, fingerprint, title, category, date, start_time, end_time, start_ts, end_ts, all_day, join_url, source_provider, account_id
                 FROM canonical_events WHERE date = ?1 ORDER BY start_ts ASC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt.query_map(params![ymd], Self::map_event).map_err(|e| e.to_string())?;
        let out = Self::collect_events(rows);
        out
    }

    pub fn list_upcoming_events(&self, from_ymd: &str, limit: usize) -> Result<Vec<CanonicalEvent>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, fingerprint, title, category, date, start_time, end_time, start_ts, end_ts, all_day, join_url, source_provider, account_id
                 FROM canonical_events WHERE date >= ?1 ORDER BY date ASC, start_ts ASC LIMIT ?2",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![from_ymd, limit as i64], Self::map_event)
            .map_err(|e| e.to_string())?;
        let out = Self::collect_events(rows);
        out
    }

    fn map_event(r: &rusqlite::Row<'_>) -> rusqlite::Result<CanonicalEvent> {
        let prov: String = r.get(11)?;
        Ok(CanonicalEvent {
            id: r.get(0)?,
            fingerprint: r.get(1)?,
            title: r.get(2)?,
            category: r.get(3)?,
            date: r.get(4)?,
            start_time: r.get(5)?,
            end_time: r.get(6)?,
            start_ts: r.get(7)?,
            end_ts: r.get(8)?,
            all_day: r.get::<_, i64>(9)? != 0,
            join_url: r.get(10)?,
            source_provider: ProviderKind::from_str(&prov),
            account_id: r.get(12)?,
        })
    }

    fn collect_events<I>(rows: I) -> Result<Vec<CanonicalEvent>, String>
    where
        I: Iterator<Item = rusqlite::Result<CanonicalEvent>>,
    {
        let mut out = Vec::new();
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn list_upcoming_dates(&self, today: NaiveDate, window_days: i64) -> Result<Vec<UpcomingDate>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, person_name, kind, month, day, year_optional, label, source_provider
                 FROM important_dates",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |r| {
                let prov: String = r.get(7)?;
                Ok(ImportantDate {
                    id: r.get(0)?,
                    person_id: None,
                    person_name: r.get(1)?,
                    kind: r.get(2)?,
                    month: r.get::<_, i64>(3)? as u32,
                    day: r.get::<_, i64>(4)? as u32,
                    year_optional: r.get(5)?,
                    label: r.get(6)?,
                    source_provider: ProviderKind::from_str(&prov),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for item in rows {
            let d = item.map_err(|e| e.to_string())?;
            if let Some((days_until, target_year)) = days_until_month_day(today, d.month, d.day) {
                if days_until <= window_days {
                    let turning_age = d
                        .year_optional
                        .filter(|y| *y > 1900 && *y < target_year)
                        .map(|y| target_year - y);
                    let title = if !d.person_name.is_empty() && d.kind == "birthday" {
                        format!("Aniversário de {}", d.person_name)
                    } else {
                        d.label.clone()
                    };
                    list.push(UpcomingDate {
                        id: d.id,
                        title,
                        person_name: d.person_name,
                        kind: d.kind,
                        month: d.month,
                        day: d.day,
                        days_until,
                        turning_age,
                        source_provider: d.source_provider,
                    });
                }
            }
        }
        list.sort_by_key(|x| (x.days_until, x.kind != "birthday"));
        Ok(list)
    }

    pub fn upsert_phone(&self, dev: &PhoneDevice) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO phone_devices
                 (id, name, platform, connected, active_transport, battery_pct, charging, last_seen_ts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(id) DO UPDATE SET
                   name=excluded.name,
                   platform=excluded.platform,
                   connected=excluded.connected,
                   active_transport=excluded.active_transport,
                   battery_pct=COALESCE(excluded.battery_pct, phone_devices.battery_pct),
                   charging=excluded.charging,
                   last_seen_ts=excluded.last_seen_ts",
                params![
                    dev.id,
                    dev.name,
                    dev.platform,
                    dev.connected as i64,
                    dev.active_transport.as_str(),
                    dev.battery_pct.map(|v| v as i64),
                    dev.charging as i64,
                    dev.last_seen_ts
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_phones(&self) -> Result<Vec<PhoneDevice>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, name, platform, connected, active_transport, battery_pct, charging, last_seen_ts
                 FROM phone_devices ORDER BY connected DESC, last_seen_ts DESC",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| {
                let tr: String = r.get(4)?;
                let bat: Option<i64> = r.get(5)?;
                Ok(PhoneDevice {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    platform: r.get(2)?,
                    connected: r.get::<_, i64>(3)? != 0,
                    active_transport: PhoneTransport::from_str(&tr),
                    battery_pct: bat.map(|v| v.clamp(0, 100) as u8),
                    charging: r.get::<_, i64>(6)? != 0,
                    last_seen_ts: r.get(7)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn record_phone_signal(&self, sig: &PhoneSignalRecord) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO phone_signals (id, device_id, kind, title, body, app_name, ts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    sig.id,
                    sig.device_id,
                    sig.kind,
                    sig.title,
                    sig.body,
                    sig.app_name,
                    sig.ts
                ],
            )
            .map_err(|e| e.to_string())?;
        // Mantém somente os 40 sinais mais recentes para não crescer indefinidamente
        self.conn
            .execute(
                "DELETE FROM phone_signals WHERE id NOT IN (
                   SELECT id FROM phone_signals ORDER BY ts DESC LIMIT 40
                 )",
                [],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn list_recent_signals(&self, limit: usize) -> Result<Vec<PhoneSignalRecord>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, device_id, kind, title, body, app_name, ts
                 FROM phone_signals ORDER BY ts DESC LIMIT ?1",
            )
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(params![limit as i64], |r| {
                Ok(PhoneSignalRecord {
                    id: r.get(0)?,
                    device_id: r.get(1)?,
                    kind: r.get(2)?,
                    title: r.get(3)?,
                    body: r.get(4)?,
                    app_name: r.get(5)?,
                    ts: r.get(6)?,
                })
            })
            .map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for item in rows {
            out.push(item.map_err(|e| e.to_string())?);
        }
        Ok(out)
    }

    pub fn was_notified_recently(&self, key: &str, since_ts: i64) -> Result<bool, String> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM notification_log WHERE key = ?1 AND fired_at >= ?2",
                params![key, since_ts],
                |r| r.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(count > 0)
    }

    pub fn mark_notified(&self, key: &str, fired_at: i64, action_taken: &str) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO notification_log (key, fired_at, action_taken)
                 VALUES (?1, ?2, ?3)",
                params![key, fired_at, action_taken],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn today_local() -> NaiveDate {
        Local::now().date_naive()
    }
}

pub fn days_until_month_day(today: NaiveDate, month: u32, day: u32) -> Option<(i64, i32)> {
    let cur_year = today.year();
    let candidate = safe_date(cur_year, month, day)?;
    if candidate >= today {
        return Some(((candidate - today).num_days(), cur_year));
    }
    let next = safe_date(cur_year + 1, month, day)?;
    Some(((next - today).num_days(), cur_year + 1))
}

fn safe_date(year: i32, month: u32, day: u32) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day).or_else(|| {
        if month == 2 && day == 29 {
            NaiveDate::from_ymd_opt(year, 2, 28)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::normalizer::event_fingerprint;

    #[test]
    fn deduplicates_identical_events_across_providers() {
        let db = PersonalMemoryDb::open_in_memory().unwrap();
        let fp = event_fingerprint("2026-11-14", "10:00", "Daily Técnico");
        let e1 = CanonicalEvent {
            id: "g-1".into(),
            fingerprint: fp.clone(),
            title: "Daily Técnico".into(),
            category: "meeting".into(),
            date: "2026-11-14".into(),
            start_time: "10:00".into(),
            end_time: "10:30".into(),
            start_ts: 1794650400,
            end_ts: 1794652200,
            all_day: false,
            join_url: "".into(),
            source_provider: ProviderKind::Google,
            account_id: "google:main".into(),
        };
        let mut e2 = e1.clone();
        e2.id = "w-1".into();
        e2.source_provider = ProviderKind::Windows;
        e2.account_id = "windows:local".into();
        e2.join_url = "https://teams.microsoft.com/meet/123".into();

        db.upsert_event(&e1).unwrap();
        db.upsert_event(&e2).unwrap();

        let list = db.list_events_for_date("2026-11-14").unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].join_url, "https://teams.microsoft.com/meet/123");
    }

    #[test]
    fn calculates_upcoming_birthdays_and_age() {
        let db = PersonalMemoryDb::open_in_memory().unwrap();
        db.upsert_person(&PersonRecord {
            id: "p1".into(),
            name: "Amanda Silva".into(),
            relation: "Amiga".into(),
            email: "amanda@example.com".into(),
            phone: "".into(),
            source_provider: ProviderKind::Google,
        })
        .unwrap();
        db.upsert_important_date(&ImportantDate {
            id: "bday-p1".into(),
            person_id: Some("p1".into()),
            person_name: "Amanda Silva".into(),
            kind: "birthday".into(),
            month: 11,
            day: 15,
            year_optional: Some(1996),
            label: "Aniversário da Amanda".into(),
            source_provider: ProviderKind::Google,
        })
        .unwrap();

        let today = NaiveDate::from_ymd_opt(2026, 11, 14).unwrap();
        let up = db.list_upcoming_dates(today, 30).unwrap();
        let amanda = up.iter().find(|u| u.id == "bday-p1").unwrap();
        assert_eq!(amanda.days_until, 1);
        assert_eq!(amanda.turning_age, Some(30));
    }
}
