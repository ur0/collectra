ALTER TABLE devices
ADD COLUMN num_checkins INTEGER NOT NULL DEFAULT 1;
ALTER TABLE devices
ADD COLUMN last_checkin BIGINT NOT NULL DEFAULT 0;