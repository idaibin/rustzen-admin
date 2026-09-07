-- Optional Admin notification schema for fresh compositions selecting notifications.

CREATE TABLE notification_accounting (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    message_count INTEGER NOT NULL DEFAULT 0 CHECK (message_count >= 0),
    recipient_count INTEGER NOT NULL DEFAULT 0 CHECK (recipient_count >= 0),
    receipt_count INTEGER NOT NULL DEFAULT 0 CHECK (receipt_count >= 0),
    charged_bytes INTEGER NOT NULL DEFAULT 0 CHECK (charged_bytes >= 0)
);
INSERT INTO notification_accounting (id) VALUES (1);

CREATE TABLE notification_receipts (
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL CHECK (length(CAST(event_id AS BLOB)) BETWEEN 1 AND 128),
    payload_sha256 TEXT NOT NULL CHECK (
        length(payload_sha256) = 64 AND payload_sha256 NOT GLOB '*[^0-9a-f]*'
    ),
    accepted_at DATETIME NOT NULL,
    expires_at DATETIME NOT NULL,
    retain_until DATETIME NOT NULL,
    result TEXT NOT NULL CHECK (result IN ('stored', 'no-recipients')),
    charged_bytes INTEGER NOT NULL DEFAULT 512 CHECK (charged_bytes >= 512),
    PRIMARY KEY (producer, event_id)
);
CREATE INDEX idx_notification_receipts_retain_until
    ON notification_receipts(retain_until);

CREATE TABLE notifications (
    inbox_seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE CHECK (length(CAST(id AS BLOB)) BETWEEN 1 AND 64),
    producer TEXT NOT NULL CHECK (producer IN ('monitor', 'reports')),
    event_id TEXT NOT NULL CHECK (length(CAST(event_id AS BLOB)) BETWEEN 1 AND 128),
    topic TEXT NOT NULL CHECK (length(CAST(topic AS BLOB)) BETWEEN 1 AND 128),
    subject_kind TEXT NOT NULL CHECK (length(CAST(subject_kind AS BLOB)) BETWEEN 1 AND 64),
    subject_id TEXT NOT NULL CHECK (length(CAST(subject_id AS BLOB)) BETWEEN 1 AND 256),
    subject_revision INTEGER NOT NULL CHECK (subject_revision > 0),
    occurred_at DATETIME NOT NULL,
    accepted_at DATETIME NOT NULL,
    title TEXT NOT NULL CHECK (length(CAST(title AS BLOB)) BETWEEN 1 AND 256),
    summary TEXT NOT NULL CHECK (length(CAST(summary AS BLOB)) <= 1024),
    required_capability TEXT NOT NULL CHECK (
        length(CAST(required_capability AS BLOB)) BETWEEN 1 AND 128
    ),
    charged_bytes INTEGER NOT NULL DEFAULT 1024 CHECK (charged_bytes >= 1024),
    UNIQUE (producer, event_id),
    FOREIGN KEY (producer, event_id)
        REFERENCES notification_receipts(producer, event_id) ON DELETE CASCADE
);
CREATE INDEX idx_notifications_accepted_order ON notifications(inbox_seq DESC);
CREATE INDEX idx_notifications_accepted_at ON notifications(accepted_at, inbox_seq);

CREATE TABLE notification_recipients (
    notification_id TEXT NOT NULL CHECK (
        length(CAST(notification_id AS BLOB)) BETWEEN 1 AND 64
    ),
    user_id INTEGER NOT NULL,
    read_at DATETIME,
    created_at DATETIME NOT NULL,
    charged_bytes INTEGER NOT NULL DEFAULT 256 CHECK (charged_bytes >= 256),
    PRIMARY KEY (notification_id, user_id),
    FOREIGN KEY (notification_id) REFERENCES notifications(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX idx_notification_recipients_user_read
    ON notification_recipients(user_id, read_at, notification_id);

CREATE TABLE notification_user_state (
    user_id INTEGER PRIMARY KEY,
    revision INTEGER NOT NULL DEFAULT 0 CHECK (revision >= 0),
    charged_bytes INTEGER NOT NULL DEFAULT 128 CHECK (charged_bytes >= 128),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TRIGGER notification_receipts_account_insert
AFTER INSERT ON notification_receipts BEGIN
    UPDATE notification_accounting SET
        receipt_count = receipt_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_receipts_account_delete
AFTER DELETE ON notification_receipts BEGIN
    UPDATE notification_accounting SET
        receipt_count = receipt_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notifications_account_insert
AFTER INSERT ON notifications BEGIN
    UPDATE notification_accounting SET
        message_count = message_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notifications_account_delete
AFTER DELETE ON notifications BEGIN
    UPDATE notification_accounting SET
        message_count = message_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_recipients_account_insert
AFTER INSERT ON notification_recipients BEGIN
    UPDATE notification_accounting SET
        recipient_count = recipient_count + 1,
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_recipients_account_delete
AFTER DELETE ON notification_recipients BEGIN
    UPDATE notification_accounting SET
        recipient_count = recipient_count - 1,
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_user_state_account_insert
AFTER INSERT ON notification_user_state BEGIN
    UPDATE notification_accounting SET
        charged_bytes = charged_bytes + NEW.charged_bytes WHERE id = 1;
END;
CREATE TRIGGER notification_user_state_account_delete
AFTER DELETE ON notification_user_state BEGIN
    UPDATE notification_accounting SET
        charged_bytes = charged_bytes - OLD.charged_bytes WHERE id = 1;
END;

CREATE TRIGGER notification_receipts_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_receipts BEGIN
    SELECT RAISE(ABORT, 'notification receipt charge is immutable');
END;
CREATE TRIGGER notifications_charge_immutable
BEFORE UPDATE OF charged_bytes ON notifications BEGIN
    SELECT RAISE(ABORT, 'notification message charge is immutable');
END;
CREATE TRIGGER notification_recipients_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_recipients BEGIN
    SELECT RAISE(ABORT, 'notification recipient charge is immutable');
END;
CREATE TRIGGER notification_user_state_charge_immutable
BEFORE UPDATE OF charged_bytes ON notification_user_state BEGIN
    SELECT RAISE(ABORT, 'notification user-state charge is immutable');
END;
