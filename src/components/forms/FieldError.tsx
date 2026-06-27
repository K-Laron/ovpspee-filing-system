export function FieldError({ message, id }: { message?: string | null; id?: string }) {
  if (!message) return null;
  return (
    <p className="text-red-500 text-sm mt-1" id={id} role="alert">
      {message}
    </p>
  );
}
