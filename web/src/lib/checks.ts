// Small checks for what kids and guardians type on the kiosk forms.

/** A full name has a first and a last name: at least two words. */
export function isFullName(name: string): boolean {
  return name.trim().split(/\s+/).length >= 2;
}

/** A phone number has ten digits, or eleven starting with 1. Spaces, dashes and brackets are fine. */
export function isPhone(phone: string): boolean {
  const digits = phone.replace(/\D/g, '');
  return digits.length === 10 || (digits.length === 11 && digits.startsWith('1'));
}
