export interface IContact {
  id?: number;
  user_id: number;
  qso_date: string;
  qso_time: string;
  callsign: string;
  operator_callsign: string;
  band: string;
  frequency: string;
  mode: string;
  rst_sent?: string;
  rst_received?: string;
  name_received?: string;
  qth_received?: string;
  grid_square?: string;
  country?: string;
  state_province?: string;
  county?: string;
  notes?: string;
  is_confirmed?: boolean;
  created_at?: string;
  updated_at?: string;
}

export class Contact implements IContact {
  id?: number;
  user_id: number;
  qso_date: string;
  qso_time: string;
  callsign: string;
  operator_callsign: string;
  band: string;
  frequency: string;
  mode: string;
  rst_sent?: string;
  rst_received?: string;
  name_received?: string;
  qth_received?: string;
  grid_square?: string;
  country?: string;
  state_province?: string;
  county?: string;
  notes?: string;
  is_confirmed?: boolean;
  created_at?: string;
  updated_at?: string;

  constructor(
    id: number,
    user_id: number,
    qso_date: string,
    qso_time: string,
    callsign: string,
    operator_callsign: string,
    band: string,
    frequency: string,
    mode: string,
    rst_sent?: string,
    rst_received?: string,
    name_received?: string,
    qth_received?: string,
    grid_square?: string,
    country?: string,
    state_province?: string,
    county?: string,
    notes?: string,
    is_confirmed?: boolean,
    created_at?: string,
    updated_at?: string
  ) {
    this.id = id;
    this.user_id = user_id;
    this.qso_date = qso_date;
    this.qso_time = qso_time;
    this.callsign = callsign;
    this.operator_callsign = operator_callsign;
    this.band = band;
    this.frequency = frequency;
    this.mode = mode;
    this.rst_sent = rst_sent;
    this.rst_received = rst_received;
    this.name_received = name_received;
    this.qth_received = qth_received;
    this.grid_square = grid_square;
    this.country = country;
    this.state_province = state_province;
    this.county = county;
    this.notes = notes;
    this.is_confirmed = is_confirmed;
    this.created_at = created_at;
    this.updated_at = updated_at;
  }
}
