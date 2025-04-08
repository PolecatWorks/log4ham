-- Add up migration script here

CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
    forename VARCHAR ( 50 ) NOT NULL,
    surname VARCHAR ( 50 ) NOT NULL,
    password VARCHAR ( 50 ) NOT NULL
);

CREATE TABLE logs (
    id BIGSERIAL PRIMARY KEY,
    description VARCHAR ( 1000 ) NOT NULL,
    user_id BIGSERIAL REFERENCES users ON DELETE RESTRICT,
    contacttime TIMESTAMP(0)
);

-- Create enum types for common fields
CREATE TYPE band AS ENUM ('B160m', 'B80m', 'B60m', 'B40m', 'B30m', 'B20m', 'B17m', 'B15m', 'B12m', 'B10m', 'B6m', 'B2m', 'B70cm', 'B23cm', 'Other');
CREATE TYPE mode AS ENUM ('Ssb', 'Am', 'FM', 'Cw', 'RTTY', 'PSK31', 'FT8', 'FT4', 'JS8', 'SSTV', 'EME', 'SATELLITE', 'Other');

-- Main contacts table
CREATE TABLE contacts (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGSERIAL REFERENCES users(id) ON DELETE RESTRICT,
    qso_date DATE NOT NULL,
    qso_time TIME NOT NULL,
    callsign VARCHAR(10) NOT NULL,
    operator_callsign VARCHAR(10) NOT NULL,
    band band NOT NULL,
    frequency NUMERIC(10, 3) NOT NULL,  -- in MHz with 3 decimal precision
    mode mode NOT NULL,
    rst_sent VARCHAR(3),  -- RST report sent
    rst_received VARCHAR(3),  -- RST report received
    name_received VARCHAR(50),
    qth_received VARCHAR(100),  -- Location of contact
    grid_square VARCHAR(6),  -- Maidenhead grid locator
    country VARCHAR(50),
    state_province VARCHAR(50),
    county VARCHAR(50),
    notes TEXT,
    is_confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- -- QSL card tracking
CREATE TABLE qsl_cards (
    id BIGSERIAL PRIMARY KEY,
    contact_id BIGSERIAL REFERENCES contacts(id) ON DELETE RESTRICT,
    qsl_sent_date DATE,
    qsl_sent_via VARCHAR(20), -- e.g., 'direct', 'bureau', 'eQSL', 'LOTW'
    qsl_received_date DATE,
    qsl_received_via VARCHAR(20),
    qsl_message TEXT
);

-- -- Station equipment used
CREATE TABLE station_setup (
    id BIGSERIAL PRIMARY KEY,
    contact_id BIGSERIAL REFERENCES contacts(id) ON DELETE RESTRICT,
    radio_model VARCHAR(100),
    antenna_type VARCHAR(100),
    power_output NUMERIC(6, 1),  -- in watts
    other_equipment TEXT
);

-- -- -- Create indexes for faster searching
CREATE INDEX idx_contacts_callsign ON contacts(callsign);
CREATE INDEX idx_contacts_date ON contacts(qso_date);
CREATE INDEX idx_contacts_band ON contacts(band);
CREATE INDEX idx_contacts_mode ON contacts(mode);
CREATE INDEX idx_contacts_grid ON contacts(grid_square);
CREATE INDEX idx_contacts_country ON contacts(country);

-- -- Function to update timestamp on record changes
CREATE OR REPLACE FUNCTION update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE 'plpgsql';

-- -- Trigger to automatically update timestamp
CREATE TRIGGER update_contacts_timestamp
BEFORE UPDATE ON contacts
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();
