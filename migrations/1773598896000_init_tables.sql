PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS user (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task (
    id INTEGER PRIMARY KEY,
    name TEXT,
    command TEXT NOT NULL,
    working_dir TEXT NOT NULL,
    cron_expr TEXT NOT NULL ,
    is_active BOOLEAN NOT NULL
);

CREATE TABLE IF NOT EXISTS task_run (
    id INTEGER PRIMARY KEY,
    task_id INTEGER NOT NULL,

    -- Execution Timing
    created_dt DATETIME NOT NULL,
    finish_dt DATETIME,

    -- Status & Resolution
    status TEXT NOT NULL,

    -- Process Metadata
    max_ram_bytes INTEGER,
    max_cpu_percent REAL,

    FOREIGN KEY(task_id) REFERENCES task(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS runlog (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id INTEGER NOT NULL,
    created_dt DATETIME DEFAULT CURRENT_TIMESTAMP,
    level TEXT NOT NULL,
    message TEXT,
    raw_payload TEXT NOT NULL,
    FOREIGN KEY(run_id) REFERENCES task_run(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS ip_whitelist (
    address TEXT PRIMARY KEY
);

CREATE VIRTUAL TABLE IF NOT EXISTS runlog_fts USING fts5(
    message,
    raw_payload,
    run_id UNINDEXED,
    content='runlog',
    content_rowid='id'
);

CREATE TRIGGER IF NOT EXISTS runlog_ai AFTER INSERT ON runlog BEGIN
    INSERT INTO runlog_fts(rowid, message, raw_payload, run_id)
    VALUES (new.id, new.message, new.raw_payload, new.run_id);
END;

CREATE TRIGGER IF NOT EXISTS runlog_ad AFTER DELETE ON runlog BEGIN
    INSERT INTO runlog_fts(runlog_fts, rowid, message, raw_payload, run_id)
    VALUES('delete', old.id, old.message, old.raw_payload, old.run_id);
END;

CREATE TRIGGER IF NOT EXISTS runlog_au AFTER UPDATE ON runlog BEGIN
    INSERT INTO runlog_fts(runlog_fts, rowid, message, raw_payload, run_id)
    VALUES('delete', old.id, old.message, old.raw_payload, old.run_id);

    INSERT INTO runlog_fts(rowid, message, raw_payload, run_id)
    VALUES (new.id, new.message, new.raw_payload, new.run_id);
END;
