import { Link } from 'react-router-dom';

interface BreadcrumbSegment {
  label: string;
  href?: string;
}

interface BreadcrumbsProps {
  segments: BreadcrumbSegment[];
}

export function Breadcrumbs({ segments }: BreadcrumbsProps) {
  return (
    <nav className="mb-4 text-sm text-muted" aria-label="Breadcrumb">
      {segments.map((segment, i) => (
        <span key={i}>
          {i > 0 && <span className="mx-2 text-muted">/</span>}
          {segment.href ? (
            <Link to={segment.href} className="hover:text-secondary transition-colors">
              {segment.label}
            </Link>
          ) : (
            <span className="text-secondary">{segment.label}</span>
          )}
        </span>
      ))}
    </nav>
  );
}
