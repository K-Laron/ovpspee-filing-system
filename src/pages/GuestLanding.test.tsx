import { render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { GuestLanding } from './GuestLanding';
import type { DocumentDetail, DocumentItem } from '../types';

vi.mock('@tauri-apps/plugin-dialog', () => ({
  save: vi.fn()
}));

vi.mock('../components/AttachmentPreview', () => ({
  AttachmentPreview: () => <div>Attachment preview</div>
}));

vi.mock('../lib/invoke', () => ({
  exportDocumentPdf: vi.fn(),
  getPublicDocument: vi.fn(),
  listPrintPrinters: vi.fn(),
  listPublicCategories: vi.fn(),
  listPublicDocuments: vi.fn(),
  listPublicFolders: vi.fn(),
  printDocumentPdf: vi.fn()
}));

import {
  getPublicDocument,
  listPrintPrinters,
  listPublicCategories,
  listPublicDocuments
} from '../lib/invoke';

const documentRow: DocumentItem = {
  document_id: 1,
  document_name: 'Public memo',
  category_id: 1,
  category_name: 'General',
  folder_id: null,
  folder_name: null,
  office_id: null,
  office_name: null,
  date_received: '2026-05-18',
  date_added: '2026-05-18T00:00:00Z',
  remarks: null,
  status: 'Filed',
  is_hidden: false,
  is_trashed: false,
  attachment_count: 0,
  created_by: 1,
  created_by_name: 'Admin User',
  updated_at: '2026-05-18T00:00:00Z'
};

const documentDetail: DocumentDetail = {
  document: documentRow,
  attachments: []
};

describe('GuestLanding printer loading', () => {
  beforeEach(() => {
    vi.mocked(listPublicCategories).mockResolvedValue([]);
    vi.mocked(listPublicDocuments).mockResolvedValue([documentRow]);
    vi.mocked(getPublicDocument).mockResolvedValue(documentDetail);
    vi.mocked(listPrintPrinters).mockReset();
  });

  it.each([null, undefined])('keeps public documents usable when printer list is %s', async (value) => {
    vi.mocked(listPrintPrinters).mockResolvedValue(value as never);

    render(<GuestLanding />);

    expect(await screen.findAllByText('Public memo')).toHaveLength(2);
    expect(screen.getAllByText('Printers are not available right now. You can still view and export public documents.').length).toBeGreaterThan(0);
    await waitFor(() => expect(screen.getByRole('combobox', { name: 'Printer' })).toBeDisabled());
    expect(screen.getByRole('button', { name: 'Export PDF' })).toBeEnabled();
  });

  it('keeps public documents usable when printer loading fails', async () => {
    vi.mocked(listPrintPrinters).mockRejectedValue(new Error('C:\\driver\\printer failed'));

    render(<GuestLanding />);

    expect(await screen.findAllByText('Public memo')).toHaveLength(2);
    expect(screen.getAllByText('Printers are not available right now. You can still view and export public documents.').length).toBeGreaterThan(0);
    expect(screen.queryByText('C:\\driver\\printer failed')).not.toBeInTheDocument();
    await waitFor(() => expect(screen.getByRole('combobox', { name: 'Printer' })).toBeDisabled());
  });
});
