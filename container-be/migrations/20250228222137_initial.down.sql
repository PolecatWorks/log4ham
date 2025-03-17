-- Add down migration script here

DROP TABLE logs;

DROP TABLE users;

DROP TRIGGER update_contacts_timestamp ON contacts;
DROP FUNCTION update_modified_column;

DROP TABLE qsl_cards;
DROP TABLE station_setup;

DROP TABLE contacts;


DROP TYPE band_type;
DROP TYPE mode_type;
