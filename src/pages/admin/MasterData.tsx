import { Edit2, Lock, Plus, RefreshCw, X } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { FormEvent, ReactNode } from 'react';

import {
  createCategory,
  createFolder,
  createOffice,
  listCategories,
  listFolders,
  listOffices,
  updateCategory,
  updateFolder,
  updateOffice
} from '../../lib/invoke';
import { getErrorMessage } from '../../lib/errors';
import { useSessionStore } from '../../store/sessionStore';
import type { CategoryItem, FolderItem, OfficeItem } from '../../types';

type Tab = 'categories' | 'folders' | 'offices';

const emptyCategory = {
  categoryName: '',
  description: '',
  colorCode: '#2563EB',
  icon: ''
};

const emptyFolder = {
  categoryId: '',
  folderName: '',
  description: '',
  folderColor: '#0F766E'
};

const emptyOffice = {
  officeName: '',
  description: ''
};

export const MasterData = () => {
  const { sessionId } = useSessionStore();
  const [tab, setTab] = useState<Tab>('categories');
  const [categories, setCategories] = useState<CategoryItem[]>([]);
  const [folders, setFolders] = useState<FolderItem[]>([]);
  const [offices, setOffices] = useState<OfficeItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState('');
  const [notice, setNotice] = useState('');

  const [categoryForm, setCategoryForm] = useState(emptyCategory);
  const [editingCategory, setEditingCategory] = useState<CategoryItem | null>(null);
  const [categoryActive, setCategoryActive] = useState(true);

  const [folderForm, setFolderForm] = useState(emptyFolder);
  const [editingFolder, setEditingFolder] = useState<FolderItem | null>(null);
  const [folderActive, setFolderActive] = useState(true);

  const [officeForm, setOfficeForm] = useState(emptyOffice);
  const [editingOffice, setEditingOffice] = useState<OfficeItem | null>(null);
  const [officeActive, setOfficeActive] = useState(true);

  const editableCategories = useMemo(
    () => categories.filter((category) => !category.is_system && category.is_active),
    [categories]
  );

  const reload = async () => {
    if (!sessionId) return;
    setLoading(true);
    setError('');
    try {
      const [nextCategories, nextFolders, nextOffices] = await Promise.all([
        listCategories(sessionId, true),
        listFolders(sessionId, null, true),
        listOffices(sessionId, true)
      ]);
      setCategories(nextCategories);
      setFolders(nextFolders);
      setOffices(nextOffices);
      if (!folderForm.categoryId) {
        const first = nextCategories.find((category) => !category.is_system && category.is_active);
        if (first) setFolderForm((current) => ({ ...current, categoryId: String(first.category_id) }));
      }
    } catch (err) {
      setError(getErrorMessage(err, 'Could not load master data.'));
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void reload();
  }, [sessionId]);

  const clearMessage = () => {
    setError('');
    setNotice('');
  };

  const submitCategory = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    clearMessage();
    setSaving(true);
    try {
      const payload = {
        sessionId,
        categoryName: categoryForm.categoryName,
        description: nullable(categoryForm.description),
        colorCode: categoryForm.colorCode,
        icon: nullable(categoryForm.icon)
      };
      if (editingCategory) {
        await updateCategory({
          ...payload,
          categoryId: editingCategory.category_id,
          isActive: categoryActive
        });
        setNotice('Category updated.');
      } else {
        await createCategory(payload);
        setNotice('Category created.');
      }
      cancelCategoryEdit();
      await reload();
    } catch (err) {
      setError(getErrorMessage(err, 'Category save failed.'));
    } finally {
      setSaving(false);
    }
  };

  const submitFolder = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    clearMessage();
    setSaving(true);
    try {
      const categoryId = Number(folderForm.categoryId);
      const payload = {
        sessionId,
        categoryId,
        folderName: folderForm.folderName,
        description: nullable(folderForm.description),
        folderColor: folderForm.folderColor
      };
      if (editingFolder) {
        await updateFolder({
          ...payload,
          folderId: editingFolder.folder_id,
          isActive: folderActive
        });
        setNotice('Folder updated.');
      } else {
        await createFolder(payload);
        setNotice('Folder created.');
      }
      cancelFolderEdit();
      await reload();
    } catch (err) {
      setError(getErrorMessage(err, 'Folder save failed.'));
    } finally {
      setSaving(false);
    }
  };

  const submitOffice = async (event: FormEvent) => {
    event.preventDefault();
    if (!sessionId || saving) return;
    clearMessage();
    setSaving(true);
    try {
      const payload = {
        sessionId,
        officeName: officeForm.officeName,
        description: nullable(officeForm.description)
      };
      if (editingOffice) {
        await updateOffice({
          ...payload,
          officeId: editingOffice.office_id,
          isActive: officeActive
        });
        setNotice('Office updated.');
      } else {
        await createOffice(payload);
        setNotice('Office created.');
      }
      cancelOfficeEdit();
      await reload();
    } catch (err) {
      setError(getErrorMessage(err, 'Office save failed.'));
    } finally {
      setSaving(false);
    }
  };

  const editCategory = (category: CategoryItem) => {
    if (category.is_system) return;
    setTab('categories');
    setEditingCategory(category);
    setCategoryActive(category.is_active);
    setCategoryForm({
      categoryName: category.category_name,
      description: category.description ?? '',
      colorCode: category.color_code,
      icon: category.icon ?? ''
    });
  };

  const editFolder = (folder: FolderItem) => {
    setTab('folders');
    setEditingFolder(folder);
    setFolderActive(folder.is_active);
    setFolderForm({
      categoryId: String(folder.category_id),
      folderName: folder.folder_name,
      description: folder.description ?? '',
      folderColor: folder.folder_color
    });
  };

  const editOffice = (office: OfficeItem) => {
    setTab('offices');
    setEditingOffice(office);
    setOfficeActive(office.is_active);
    setOfficeForm({
      officeName: office.office_name,
      description: office.description ?? ''
    });
  };

  const cancelCategoryEdit = () => {
    setEditingCategory(null);
    setCategoryActive(true);
    setCategoryForm(emptyCategory);
  };

  const cancelFolderEdit = () => {
    setEditingFolder(null);
    setFolderActive(true);
    setFolderForm((current) => ({
      ...emptyFolder,
      categoryId: current.categoryId || editableCategories[0]?.category_id.toString() || ''
    }));
  };

  const cancelOfficeEdit = () => {
    setEditingOffice(null);
    setOfficeActive(true);
    setOfficeForm(emptyOffice);
  };

  return (
    <div className="mx-auto max-w-6xl space-y-5">
      <div className="flex flex-wrap items-center justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold text-secondary">Master Data</h1>
          <p className="text-sm text-muted">Categories, folders, and offices</p>
        </div>
        <button
          className="focus-ring inline-flex h-10 items-center gap-2 rounded border border-border bg-surface px-3 text-sm font-medium text-secondary hover:bg-background"
          disabled={loading}
          onClick={() => void reload()}
          type="button"
        >
          <RefreshCw size={16} />
          Refresh
        </button>
      </div>

      <div className="flex gap-2 border-b border-border">
        {(['categories', 'folders', 'offices'] as Tab[]).map((item) => (
          <button
            className={[
              'focus-ring h-10 border-b-2 px-3 text-sm font-medium capitalize',
              tab === item
                ? 'border-primary text-primary'
                : 'border-transparent text-muted hover:text-secondary'
            ].join(' ')}
            key={item}
            onClick={() => setTab(item)}
            type="button"
          >
            {item}
          </button>
        ))}
      </div>

      {error && <div className="rounded border border-primary/30 bg-primary/10 px-4 py-3 text-sm text-primary">{error}</div>}
      {notice && <div className="rounded border border-success/30 bg-success/10 px-4 py-3 text-sm text-success">{notice}</div>}

      {tab === 'categories' && (
        <section className="grid gap-5 lg:grid-cols-[1fr_20rem]">
          <DataTable
            headers={['Name', 'Color', 'Status', 'Documents', 'Action']}
            loading={loading}
            rows={categories.map((category) => [
              <NameCell
                description={category.description}
                key="name"
                locked={category.is_system}
                name={category.category_name}
              />,
              <Swatch key="color" value={category.color_code} />,
              <Status key="status" active={category.is_active} />,
              category.document_count.toString(),
              category.is_system ? (
                <span className="inline-flex h-9 items-center gap-2 rounded border border-border px-3 text-xs text-muted">
                  <Lock size={14} />
                  Locked
                </span>
              ) : (
                <IconButton key="edit" label="Edit category" onClick={() => editCategory(category)} />
              )
            ])}
          />
          <form className="space-y-3 rounded border border-border bg-surface p-4" onSubmit={submitCategory}>
            <FormTitle editing={Boolean(editingCategory)} label="Category" onCancel={cancelCategoryEdit} />
            <TextField label="Name" value={categoryForm.categoryName} onChange={(value) => setCategoryForm({ ...categoryForm, categoryName: value })} required />
            <TextField label="Description" value={categoryForm.description} onChange={(value) => setCategoryForm({ ...categoryForm, description: value })} />
            <ColorField label="Color" value={categoryForm.colorCode} onChange={(value) => setCategoryForm({ ...categoryForm, colorCode: value })} />
            <TextField label="Icon" value={categoryForm.icon} onChange={(value) => setCategoryForm({ ...categoryForm, icon: value })} />
            {editingCategory && <ActiveToggle checked={categoryActive} onChange={setCategoryActive} />}
            <SubmitButton disabled={saving} editing={Boolean(editingCategory)} />
          </form>
        </section>
      )}

      {tab === 'folders' && (
        <section className="grid gap-5 lg:grid-cols-[1fr_20rem]">
          <DataTable
            headers={['Folder', 'Category', 'Color', 'Status', 'Action']}
            loading={loading}
            rows={folders.map((folder) => [
              <NameCell description={folder.description} key="name" name={folder.folder_name} />,
              folder.category_name,
              <Swatch key="color" value={folder.folder_color} />,
              <Status key="status" active={folder.is_active} />,
              <IconButton key="edit" label="Edit folder" onClick={() => editFolder(folder)} />
            ])}
          />
          <form className="space-y-3 rounded border border-border bg-surface p-4" onSubmit={submitFolder}>
            <FormTitle editing={Boolean(editingFolder)} label="Folder" onCancel={cancelFolderEdit} />
            <label className="block text-sm font-medium text-secondary">
              Category
              <select
                className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm"
                onChange={(event) => setFolderForm({ ...folderForm, categoryId: event.target.value })}
                required
                value={folderForm.categoryId}
              >
                <option value="">Select category</option>
                {editableCategories.map((category) => (
                  <option key={category.category_id} value={category.category_id}>
                    {category.category_name}
                  </option>
                ))}
              </select>
            </label>
            <TextField label="Name" value={folderForm.folderName} onChange={(value) => setFolderForm({ ...folderForm, folderName: value })} required />
            <TextField label="Description" value={folderForm.description} onChange={(value) => setFolderForm({ ...folderForm, description: value })} />
            <ColorField label="Color" value={folderForm.folderColor} onChange={(value) => setFolderForm({ ...folderForm, folderColor: value })} />
            {editingFolder && <ActiveToggle checked={folderActive} onChange={setFolderActive} />}
            <SubmitButton disabled={saving || editableCategories.length === 0} editing={Boolean(editingFolder)} />
          </form>
        </section>
      )}

      {tab === 'offices' && (
        <section className="grid gap-5 lg:grid-cols-[1fr_20rem]">
          <DataTable
            headers={['Office', 'Status', 'Action']}
            loading={loading}
            rows={offices.map((office) => [
              <NameCell description={office.description} key="name" name={office.office_name} />,
              <Status key="status" active={office.is_active} />,
              <IconButton key="edit" label="Edit office" onClick={() => editOffice(office)} />
            ])}
          />
          <form className="space-y-3 rounded border border-border bg-surface p-4" onSubmit={submitOffice}>
            <FormTitle editing={Boolean(editingOffice)} label="Office" onCancel={cancelOfficeEdit} />
            <TextField label="Name" value={officeForm.officeName} onChange={(value) => setOfficeForm({ ...officeForm, officeName: value })} required />
            <TextField label="Description" value={officeForm.description} onChange={(value) => setOfficeForm({ ...officeForm, description: value })} />
            {editingOffice && <ActiveToggle checked={officeActive} onChange={setOfficeActive} />}
            <SubmitButton disabled={saving} editing={Boolean(editingOffice)} />
          </form>
        </section>
      )}
    </div>
  );
};

const nullable = (value: string) => {
  const trimmed = value.trim();
  return trimmed ? trimmed : null;
};

const DataTable = ({
  headers,
  loading,
  rows
}: {
  headers: string[];
  loading: boolean;
  rows: Array<Array<ReactNode>>;
}) => (
  <div className="overflow-hidden rounded border border-border bg-surface">
    <table className="w-full table-fixed text-left text-sm">
      <thead className="bg-background text-xs uppercase text-muted">
        <tr>
          {headers.map((header) => (
            <th className="px-4 py-3 font-semibold" key={header}>
              {header}
            </th>
          ))}
        </tr>
      </thead>
      <tbody className="divide-y divide-border">
        {loading && (
          <tr>
            <td className="px-4 py-6 text-center text-muted" colSpan={headers.length}>
              Loading...
            </td>
          </tr>
        )}
        {!loading && rows.length === 0 && (
          <tr>
            <td className="px-4 py-6 text-center text-muted" colSpan={headers.length}>
              No records.
            </td>
          </tr>
        )}
        {!loading &&
          rows.map((row, index) => (
            <tr className="align-middle" key={index}>
              {row.map((cell, cellIndex) => (
                <td className="px-4 py-3" key={cellIndex}>
                  {cell}
                </td>
              ))}
            </tr>
          ))}
      </tbody>
    </table>
  </div>
);

const NameCell = ({
  description,
  locked = false,
  name
}: {
  description?: string | null;
  locked?: boolean;
  name: string;
}) => (
  <div className="min-w-0">
    <div className="flex items-center gap-2 font-medium text-secondary">
      <span className="truncate">{name}</span>
      {locked && <Lock className="shrink-0 text-muted" size={14} />}
    </div>
    {description && <div className="truncate text-xs text-muted">{description}</div>}
  </div>
);

const Swatch = ({ value }: { value: string }) => (
  <div className="flex items-center gap-2">
    <span className="h-5 w-5 shrink-0 rounded border border-border" style={{ backgroundColor: value }} />
    <span className="font-mono text-xs text-muted">{value}</span>
  </div>
);

const Status = ({ active }: { active: boolean }) => (
  <span
    className={[
      'inline-flex h-7 items-center rounded px-2 text-xs font-medium',
      active ? 'bg-success/10 text-success' : 'bg-warning/10 text-warning'
    ].join(' ')}
  >
    {active ? 'Active' : 'Inactive'}
  </span>
);

const IconButton = ({ label, onClick }: { label: string; onClick: () => void }) => (
  <button
    aria-label={label}
    className="focus-ring inline-flex h-9 w-9 items-center justify-center rounded border border-border text-secondary hover:bg-background"
    onClick={onClick}
    title={label}
    type="button"
  >
    <Edit2 size={15} />
  </button>
);

const FormTitle = ({
  editing,
  label,
  onCancel
}: {
  editing: boolean;
  label: string;
  onCancel: () => void;
}) => (
  <div className="flex items-center justify-between gap-2">
    <h2 className="text-base font-semibold text-secondary">{editing ? `Edit ${label}` : `New ${label}`}</h2>
    {editing && (
      <button
        aria-label="Cancel edit"
        className="focus-ring inline-flex h-8 w-8 items-center justify-center rounded border border-border text-muted hover:text-secondary"
        onClick={onCancel}
        title="Cancel edit"
        type="button"
      >
        <X size={15} />
      </button>
    )}
  </div>
);

const TextField = ({
  label,
  onChange,
  required = false,
  value
}: {
  label: string;
  onChange: (value: string) => void;
  required?: boolean;
  value: string;
}) => (
  <label className="block text-sm font-medium text-secondary">
    {label}
    <input
      className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm"
      maxLength={100}
      onChange={(event) => onChange(event.target.value)}
      required={required}
      type="text"
      value={value}
    />
  </label>
);

const ColorField = ({
  label,
  onChange,
  value
}: {
  label: string;
  onChange: (value: string) => void;
  value: string;
}) => (
  <label className="block text-sm font-medium text-secondary">
    {label}
    <input
      className="focus-ring mt-1 h-10 w-full rounded border border-border bg-white px-3 text-sm"
      onChange={(event) => onChange(event.target.value.toUpperCase())}
      pattern="^#[0-9A-Fa-f]{6}$"
      required
      type="text"
      value={value}
    />
  </label>
);

const ActiveToggle = ({
  checked,
  onChange
}: {
  checked: boolean;
  onChange: (checked: boolean) => void;
}) => (
  <label className="flex items-center gap-2 text-sm font-medium text-secondary">
    <input
      checked={checked}
      className="h-4 w-4 rounded border-border"
      onChange={(event) => onChange(event.target.checked)}
      type="checkbox"
    />
    Active
  </label>
);

const SubmitButton = ({ disabled, editing }: { disabled: boolean; editing: boolean }) => (
  <button
    className="focus-ring inline-flex h-10 w-full items-center justify-center gap-2 rounded bg-primary px-3 text-sm font-semibold text-white hover:bg-secondary disabled:cursor-not-allowed disabled:opacity-60"
    disabled={disabled}
    type="submit"
  >
    <Plus size={16} />
    {editing ? 'Save' : 'Create'}
  </button>
);
