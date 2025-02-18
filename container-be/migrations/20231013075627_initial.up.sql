-- Add up migration script here
CREATE SEQUENCE validation_good START WITH 1;
CREATE SEQUENCE validation_fail START WITH 1;


CREATE TABLE lists (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR ( 50 ) NOT NULL UNIQUE,
    active BIGINT
);

CREATE TABLE list_versions (
    id BIGSERIAL PRIMARY KEY,
    version VARCHAR (50) NOT NULL,
    schema JSON NOT NULL,
    list BIGINT REFERENCES lists ON DELETE RESTRICT
);


ALTER TABLE lists
ADD CONSTRAINT lists_active_fkey
FOREIGN KEY (active) REFERENCES list_versions ON DELETE RESTRICT;


CREATE FUNCTION check_list_update() RETURNS trigger AS $check_list_update$
    BEGIN
        -- Check if active is provided then it is valid
        IF NEW.active IS NOT NULL AND NOT EXISTS(select null from list_versions WHERE NEW.active = list_versions.id AND NEW.id = list_versions.list) THEN
            RAISE EXCEPTION 'Non-null activate must be a valid list_versions id';
        END IF;

        RETURN NEW;
    END;
$check_list_update$ LANGUAGE plpgsql;

CREATE TRIGGER check_list_update
    BEFORE UPDATE ON lists
    FOR EACH ROW
    EXECUTE FUNCTION check_list_update();

CREATE TRIGGER check_list_insert
    BEFORE INSERT ON lists
    FOR EACH ROW
    EXECUTE FUNCTION check_list_update();

CREATE TABLE list_files (
    id BIGSERIAL PRIMARY KEY,
    version BIGINT NOT NULL REFERENCES list_versions ON DELETE RESTRICT,
    validated BOOLEAN NOT NULL
);
