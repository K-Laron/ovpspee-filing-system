import { FileText, Folder, Search } from 'lucide-react';
import { FormEvent, useEffect, useState } from 'react';

import {
  getAttachmentFilePath,
  getPublicDocument,
  listPublicCategories,
  listPublicDocuments,
  listPublicFolders
} from '../lib/invoke';
import type { CategoryItem, DocumentDetail, DocumentItem, FolderItem } from '../types';

export const GuestLanding = () => {
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [documents, setDocuments] = useState<DocumentItem[]>([]);
  const [detail, setDetail] = useState<DocumentDetail | null>(null);
  const [search, setSearch] = useState('');
  const [categoryId, setCategoryId] = useState('');
  const [folderId, setFolderId] = useState('');
  const [message, setMessage] = useState('');

  const load = async () => {
    setCategories(await listPublicCategories());
    const rows = await listPublicDocuments({
      search: search || null,
      categoryId: categoryId ? Number(categoryId) : null,
      folderId: folderId ? Number(folderId) : null
    });
    setDocuments(rows);
    if (!detail && rows[0]) setDetail(await getPublicDocument(rows[0].document_id));
  };

  useEffect(() => {
    void load().catch((err) => setMessage(String(err)));
  }, []);

  const selectCategory = async (id: number) => {
    setCategoryId(String(id));
    setFolderId('');
    setFolders(await listPublicFolders(id));
    setDocuments(await listPublicDocuments({ search: search || null, categoryId: id }));
    setDetail(null);
  };

  const submitSearch = (event: FormEvent) => {
    event.preventDefault();
    void load().catch((err) => setMessage(String(err)));
  };

  const openDocument = async (documentId: number) => {
    setDetail(await getPublicDocument(documentId));
  };

  const showPath = async (attachmentId: number) => {
    setMessage(await getAttachmentFilePath(attachmentId));
  };

  return (
    <section className="space-y-5">
      <div className="rounded border border-border bg-surface p-5 shadow-sm">
        <div className="flex items-center justify-between gap-4">
          <div>
            <h1 className="text-2xl font-bold text-secondary">Browse Documents</h1>
            <p className="mt-1 text-sm text-muted">Read-only Staff/Head Viewer. Hidden, confidential, and trashed records are excluded.</p>
          </div>
          <form className="flex min-w-[360px] items-end gap-2" onSubmit={submitSearch}>
            <label className="flex-1">
              <span className="form-label">Search</span>
              <input className="input" value={search} onChange={(e) => setSearch(e.target.value)} />
            </label>
            <button className="btn btn-primary" type="submit"><Search size={16} />Search</button>
          </form>
        </div>
      </div>

      {message && <div className="rounded border border-border bg-surface p-3 text-sm text-secondary">{message}</div>}

      <div className="grid gap-5 xl:grid-cols-[280px_0.9fr_1.1fr]">
        <aside className="space-y-4">
          <div className="rounded border border-border bg-surface p-4 shadow-sm">
            <h2 className="mb-3 font-semibold text-secondary">Categories</h2>
            <div className="space-y-2">
              {categories.map((category) => (
                <button
                  className={`flex w-full items-center justify-between rounded border px-3 py-2 text-left text-sm ${categoryId === String(category.category_id) ? 'border-primary bg-red-50 text-secondary' : 'border-border bg-white text-muted hover:bg-background'}`}
                  key={category.category_id}
                  onClick={() => void selectCategory(category.category_id).catch((err) => setMessage(String(err)))}
                  type="button"
                >
                  <span className="flex items-center gap-2"><Folder size={15} />{category.category_name}</span>
                  <span>{category.document_count}</span>
                </button>
              ))}
            </div>
          </div>
          <div className="rounded border border-border bg-surface p-4 shadow-sm">
            <h2 className="mb-3 font-semibold text-secondary">Folders</h2>
            <button className="mb-2 w-full rounded border border-border px-3 py-2 text-left text-sm text-muted hover:bg-background" onClick={() => { setFolderId(''); void load(); }} type="button">Category root</button>
            {folders.map((folder) => (
              <button
                className={`mb-2 w-full rounded border px-3 py-2 text-left text-sm ${folderId === String(folder.folder_id) ? 'border-primary bg-red-50 text-secondary' : 'border-border text-muted hover:bg-background'}`}
                key={folder.folder_id}
                onClick={() => {
                  setFolderId(String(folder.folder_id));
                  void listPublicDocuments({ search: search || null, categoryId: Number(categoryId), folderId: folder.folder_id }).then(setDocuments).catch((err) => setMessage(String(err)));
                }}
                type="button"
              >
                {folder.folder_name}
              </button>
            ))}
          </div>
        </aside>

        <div className="overflow-hidden rounded border border-border bg-surface shadow-sm">
          <table className="w-full text-left text-sm">
            <thead className="border-b border-border bg-background text-xs uppercase text-muted">
              <tr><th className="p-3">Document</th><th className="p-3">Filed</th></tr>
            </thead>
            <tbody>
              {documents.map((doc) => (
                <tr className="cursor-pointer border-b border-border hover:bg-background" key={doc.document_id} onClick={() => void openDocument(doc.document_id).catch((err) => setMessage(String(err)))}>
                  <td className="p-3"><p className="font-semibold text-secondary">{doc.document_name}</p><p className="text-xs text-muted">{doc.category_name}{doc.folder_name ? ` / ${doc.folder_name}` : ''}</p></td>
                  <td className="p-3 text-muted">{doc.date_received}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        <div className="rounded border border-border bg-surface p-5 shadow-sm">
          {!detail ? <p className="text-sm text-muted">Select document.</p> : (
            <div>
              <div className="mb-4 flex items-start gap-3">
                <FileText className="mt-1 text-primary" size={24} />
                <div>
                  <h2 className="text-xl font-bold text-secondary">{detail.document.document_name}</h2>
                  <p className="text-sm text-muted">{detail.document.category_name}{detail.document.folder_name ? ` / ${detail.document.folder_name}` : ''}</p>
                </div>
              </div>
              <dl className="grid gap-3 text-sm md:grid-cols-2">
                <div><dt className="text-muted">Date received</dt><dd className="font-medium text-secondary">{detail.document.date_received}</dd></div>
                <div><dt className="text-muted">Sender office</dt><dd className="font-medium text-secondary">{detail.document.office_name ?? 'Not specified'}</dd></div>
                <div className="md:col-span-2"><dt className="text-muted">Remarks</dt><dd className="font-medium text-secondary">{detail.document.remarks ?? 'No remarks'}</dd></div>
              </dl>
              <div className="mt-5 border-t border-border pt-4">
                <h3 className="mb-3 font-semibold text-secondary">Attachments</h3>
                <div className="space-y-2">
                  {detail.attachments.map((file) => (
                    <button className="flex w-full items-center justify-between rounded border border-border p-3 text-left text-sm hover:bg-background" key={file.attachment_id} onClick={() => void showPath(file.attachment_id).catch((err) => setMessage(String(err)))} type="button">
                      <span>{file.original_file_name}</span>
                      <span className="text-xs text-muted">Show path</span>
                    </button>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
};
