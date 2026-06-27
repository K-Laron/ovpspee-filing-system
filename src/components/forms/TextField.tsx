import { FieldError } from './FieldError';

export const TextField = ({
  label,
  onChange,
  required = false,
  type = 'text',
  value,
  error,
}: {
  label: string;
  onChange: (value: string) => void;
  required?: boolean;
  type?: string;
  value: string;
  error?: string | null;
}) => {
  const id = `field-${label.toLowerCase().replace(/\s+/g, '-')}`;
  return (
    <div>
      <label htmlFor={id} className="block text-sm font-medium text-secondary">
        {label}
        {required && <span className="text-red-500 ml-0.5">*</span>}
      </label>
      <input
        id={id}
        className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm"
        maxLength={120}
        onChange={(event) => onChange(event.target.value)}
        required={required}
        type={type}
        value={value}
        aria-invalid={!!error}
        aria-describedby={error ? `${id}-error` : undefined}
        aria-required={required}
      />
      <FieldError message={error} id={error ? `${id}-error` : undefined} />
    </div>
  );
};
