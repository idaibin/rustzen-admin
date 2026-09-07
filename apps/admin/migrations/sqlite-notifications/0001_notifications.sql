-- Optional Admin notification schema for fresh compositions selecting notifications.

CREATE TABLE notification_receipts (
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL,
    payload_sha256 TEXT NOT NULL CHECK (length(payload_sha256) = 64),
    accepted_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    retain_until DATETIME NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('stored', 'no-recipients')),
    PRIMARY KEY (producer, event_id)
);

CREATE INDEX idx_notification_receipts_retain_until
    ON notification_receipts(retain_until);

CREATE TABLE notifications (
    inbox_seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL,
    topic TEXT NOT NULL,
    subject_kind TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    subject_revision INTEGER NOT NULL CHECK (subject_revision > 0),
    occurred_at DATETIME NOT NULL,
    accepted_at DATETIME NOT NULL,
    title TEXT NOT NULL CHECK (length(title) BETWEEN 1 AND 256),
    summary TEXT NOT NULL CHECK (length(summary) <= 1024),
    required_capability TEXT NOT NULL CHECK (length(required_capability) BETWEEN 1 AND 128),
    UNIQUE (producer, event_id),
    FOREIGN KEY (producer, event_id)
        REFERENCES notification_receipts(producer, event_id) ON DELETE CASCADE
);

CREATE INDEX idx_notifications_accepted_order
    ON notifications(inbox_seq DESC);

CREATE TABLE notification_recipients (
    notification_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    read_at DATETIME,
    created_at DATETIME NOT NULL,
    PRIMARY KEY (notification_id, user_id),
    FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_notification_recipients_user_read
    ON notification_recipients(user_id, read_at, notification_id);

CREATE TABLE notification_user_state (
    user_id INTEGER PRIMARY KEY,
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
