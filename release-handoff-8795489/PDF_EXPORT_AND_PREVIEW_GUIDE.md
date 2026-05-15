# PDF Export And Preview Guide

Release commit: `8795489`

## PDF Export

Supported:

- Staff/Head Viewer can export visible public documents.
- Secretary can export accessible documents, including hidden/confidential documents.
- Export uses native save flow.
- Exported PDF includes letterhead placeholder, document metadata, timestamp, page/footer marker, and certification placeholder.

Restrictions:

- Staff/Head Viewer cannot export hidden/confidential/trashed documents.
- Trashed documents are not exported from normal document detail.
- Admin does not receive normal filing/create/edit/move/status workflows.

## Attachment Preview

Supported:

- PDF preview page count/navigation.
- Image preview.
- Unsupported file state.
- Missing file state.

Deferred:

- Inline rasterizing/merging attachment pages into exported PDF.
- Preview/open audit logging.

## Evidence

- `manual-final-release-audit/final-secretary-export-pdf.png`
- `manual-final-release-audit/final-viewer-public-export.png`
- `manual-final-release-audit/final-attachment-preview.png`

