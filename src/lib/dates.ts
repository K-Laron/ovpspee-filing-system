type DateValue = string | null | undefined;

const unknownDate = 'Unknown';
const displayTimeZone = 'Asia/Manila';

const dateOnlyFormatter = new Intl.DateTimeFormat('en-US', {
  day: 'numeric',
  month: 'long',
  timeZone: 'UTC',
  year: 'numeric'
});

const dateTimeDateFormatter = new Intl.DateTimeFormat('en-US', {
  day: 'numeric',
  month: 'long',
  timeZone: displayTimeZone,
  year: 'numeric'
});

const dateTimeTimeFormatter = new Intl.DateTimeFormat('en-US', {
  hour: 'numeric',
  hour12: true,
  minute: '2-digit',
  timeZone: displayTimeZone
});

const dateInputFormatter = new Intl.DateTimeFormat('en-US', {
  day: '2-digit',
  month: '2-digit',
  timeZone: displayTimeZone,
  year: 'numeric'
});

export const formatDateOnly = (value: DateValue) => {
  if (!value) return unknownDate;

  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value);
  if (!match) return value;

  const year = Number(match[1]);
  const month = Number(match[2]);
  const day = Number(match[3]);
  const date = new Date(Date.UTC(year, month - 1, day));

  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  ) {
    return value;
  }

  return dateOnlyFormatter.format(date);
};

export const formatDateTime = (value: DateValue) => {
  if (!value) return unknownDate;

  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return value;

  return `${dateTimeDateFormatter.format(date)}, ${dateTimeTimeFormatter.format(date)}`;
};

export const formatDateInputValue = (value = new Date()) => {
  const parts = dateInputFormatter.formatToParts(value);
  const part = (type: Intl.DateTimeFormatPartTypes) => parts.find((item) => item.type === type)?.value ?? '';
  return `${part('year')}-${part('month')}-${part('day')}`;
};
