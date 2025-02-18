-- Add down migration script here
DROP TABLE list_files;

DROP TRIGGER check_list_insert on lists;
DROP TRIGGER check_list_update on lists;

DROP FUNCTION check_list_update;

ALTER TABLE lists DROP CONSTRAINT lists_active_fkey;

DROP TABLE list_versions;
DROP TABLE lists;

DROP SEQUENCE validation_fail;
DROP SEQUENCE validation_good;
