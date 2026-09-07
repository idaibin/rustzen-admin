CREATE TABLE notification_outbox (
    event_id TEXT PRIMARY KEY NOT NULL CHECK(length(CAST(event_id AS BLOB)) BETWEEN 1 AND 128),
    topic TEXT NOT NULL CHECK(length(CAST(topic AS BLOB)) BETWEEN 1 AND 128),
    subject_kind TEXT NOT NULL CHECK(length(CAST(subject_kind AS BLOB)) BETWEEN 1 AND 64),
    subject_id TEXT NOT NULL CHECK(length(CAST(subject_id AS BLOB)) BETWEEN 1 AND 128),
    subject_revision INTEGER NOT NULL CHECK(subject_revision > 0),
    payload_json TEXT NOT NULL CHECK(length(CAST(payload_json AS BLOB)) BETWEEN 2 AND 16384),
    payload_sha256 TEXT NOT NULL CHECK(length(payload_sha256) = 64),
    occurred_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    state TEXT NOT NULL CHECK(state IN ('pending','reconciling','quarantined')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts >= 0),
    next_attempt_at TEXT NOT NULL,
    lease_until TEXT,
    lease_token TEXT,
    reconcile_until TEXT,
    last_error_code TEXT CHECK(last_error_code IS NULL OR length(CAST(last_error_code AS BLOB)) <= 64),
    charged_bytes INTEGER NOT NULL CHECK(charged_bytes >= 512),
    CHECK((lease_until IS NULL) = (lease_token IS NULL)),
    UNIQUE(subject_kind, subject_id, subject_revision)
);
CREATE INDEX idx_notification_outbox_claim
ON notification_outbox(state, next_attempt_at, occurred_at);
CREATE INDEX idx_notification_outbox_subject
ON notification_outbox(subject_kind, subject_id, subject_revision);

CREATE TABLE notification_delivery_status (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    pending_count INTEGER NOT NULL DEFAULT 0 CHECK(pending_count >= 0),
    pending_bytes INTEGER NOT NULL DEFAULT 0 CHECK(pending_bytes >= 0),
    quarantine_count INTEGER NOT NULL DEFAULT 0 CHECK(quarantine_count >= 0),
    quarantine_bytes INTEGER NOT NULL DEFAULT 0 CHECK(quarantine_bytes >= 0),
    omitted_count INTEGER NOT NULL DEFAULT 0 CHECK(omitted_count >= 0),
    expired_count INTEGER NOT NULL DEFAULT 0 CHECK(expired_count >= 0),
    unconfirmed_count INTEGER NOT NULL DEFAULT 0 CHECK(unconfirmed_count >= 0),
    quarantined_count INTEGER NOT NULL DEFAULT 0 CHECK(quarantined_count >= 0),
    quarantine_evicted_count INTEGER NOT NULL DEFAULT 0 CHECK(quarantine_evicted_count >= 0),
    first_gap_at TEXT,
    last_gap_at TEXT,
    last_success_at TEXT
);
INSERT INTO notification_delivery_status(id) VALUES(1);

CREATE TRIGGER notification_outbox_account_insert AFTER INSERT ON notification_outbox BEGIN
  UPDATE notification_delivery_status SET
    pending_count=pending_count+CASE WHEN NEW.state!='quarantined' THEN 1 ELSE 0 END,
    pending_bytes=pending_bytes+CASE WHEN NEW.state!='quarantined' THEN NEW.charged_bytes ELSE 0 END,
    quarantine_count=quarantine_count+CASE WHEN NEW.state='quarantined' THEN 1 ELSE 0 END,
    quarantine_bytes=quarantine_bytes+CASE WHEN NEW.state='quarantined' THEN NEW.charged_bytes ELSE 0 END
  WHERE id=1;
END;
CREATE TRIGGER notification_outbox_account_delete AFTER DELETE ON notification_outbox BEGIN
  UPDATE notification_delivery_status SET
    pending_count=pending_count-CASE WHEN OLD.state!='quarantined' THEN 1 ELSE 0 END,
    pending_bytes=pending_bytes-CASE WHEN OLD.state!='quarantined' THEN OLD.charged_bytes ELSE 0 END,
    quarantine_count=quarantine_count-CASE WHEN OLD.state='quarantined' THEN 1 ELSE 0 END,
    quarantine_bytes=quarantine_bytes-CASE WHEN OLD.state='quarantined' THEN OLD.charged_bytes ELSE 0 END
  WHERE id=1;
END;
CREATE TRIGGER notification_outbox_account_state AFTER UPDATE OF state ON notification_outbox BEGIN
  UPDATE notification_delivery_status SET
    pending_count=pending_count-CASE WHEN OLD.state!='quarantined' THEN 1 ELSE 0 END+CASE WHEN NEW.state!='quarantined' THEN 1 ELSE 0 END,
    pending_bytes=pending_bytes-CASE WHEN OLD.state!='quarantined' THEN OLD.charged_bytes ELSE 0 END+CASE WHEN NEW.state!='quarantined' THEN NEW.charged_bytes ELSE 0 END,
    quarantine_count=quarantine_count-CASE WHEN OLD.state='quarantined' THEN 1 ELSE 0 END+CASE WHEN NEW.state='quarantined' THEN 1 ELSE 0 END,
    quarantine_bytes=quarantine_bytes-CASE WHEN OLD.state='quarantined' THEN OLD.charged_bytes ELSE 0 END+CASE WHEN NEW.state='quarantined' THEN NEW.charged_bytes ELSE 0 END
  WHERE id=1;
END;
CREATE TRIGGER notification_outbox_charge_immutable BEFORE UPDATE OF charged_bytes ON notification_outbox BEGIN
  SELECT RAISE(ABORT, 'notification outbox charge is immutable');
END;
