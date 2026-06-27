export const Status = ({ active }: { active: boolean }) => (
  <span
    className={[
      'inline-flex h-7 items-center rounded px-2 text-xs font-medium',
      active ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning',
    ].join(' ')}
  >
    {active ? 'Active' : 'Inactive'}
  </span>
);
