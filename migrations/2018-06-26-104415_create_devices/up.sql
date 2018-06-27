CREATE TABLE devices (
	id SERIAL PRIMARY KEY,
	udid_sha256 VARCHAR UNIQUE NOT NULL,
	ios_version VARCHAR NOT NULL,
	electra_version INTEGER NOT NULL,
	device_model VARCHAR NOT NULL
)