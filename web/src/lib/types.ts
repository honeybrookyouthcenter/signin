// The shapes of the JSON the server sends and receives.

export interface Rule {
  title: string;
  text: string;
}

export interface KioskConfig {
  name: string;
  short_name: string;
  tagline: string;
  accent_color: string;
  rules_intro: string;
  rules: Rule[];
  waiver_name: string;
  waiver_text: string;
  grades: string[];
  use_keyboard: boolean;
  /** When the server started. */
  started: string;
}

/** One kid in the kiosk's search list. */
export interface KioskPerson {
  id: string;
  first_name: string;
  last_name: string;
  grade: string;
  info_expired: boolean;
  /** Set while the kid is signed in. */
  visit_id: string | null;
  signed_in_at: string | null;
}

/** A kid's saved info with most of it hidden, for the "still right?" check. */
export interface MaskedInfo {
  grade: string;
  address: string;
  guardian: string;
  phone: string;
}

export interface InfoChanges {
  grade?: string;
  street?: string;
  city?: string;
  state?: string;
  guardian_name?: string;
  guardian_phone?: string;
  guardian_email?: string;
}

export interface SignUpForm {
  first_name: string;
  last_name: string;
  birthday: string;
  grade: string;
  street: string;
  city: string;
  state: string;
  guardian_name: string;
  guardian_phone: string;
  guardian_email: string;
  signature: string;
}

export interface Visit {
  id: string;
  person_id: string;
  person_name: string;
  signed_in_at: string;
  signed_out_at: string | null;
  sign_out_kind: string | null;
}

/** A kid with everything staff can see. */
export interface Person {
  id: string;
  first_name: string;
  last_name: string;
  birthday: string | null;
  grade: string;
  street: string;
  city: string;
  state: string;
  guardian_name: string;
  guardian_phone: string;
  guardian_email: string;
  info_updated: string | null;
  notes: string;
  archived: boolean;
  created_at: string;
  /** Their info is old enough that the kiosk will ask about it. */
  info_expired: boolean;
  visit_count: number;
  last_visit: string | null;
  has_waiver: boolean;
}

export interface ReportRow {
  period: string;
  visits: number;
  people: number;
}

export interface Report {
  days: ReportRow[];
  months: ReportRow[];
  total_visits: number;
  total_people: number;
  new_people: number;
}

export interface Status {
  version: string;
  outlook: {
    configured: boolean;
    connected: boolean;
    account: string | null;
    last_sync: string | null;
    last_error: string | null;
    pending: number;
    login: { user_code: string; verification_uri: string } | null;
  };
  backup: { last_backup: string | null; folder: string; in_onedrive: boolean };
  kiosk: { seconds_since_seen: number | null };
  settings: { auto_sign_out_time: string; use_keyboard: boolean; pin_set: boolean };
}
