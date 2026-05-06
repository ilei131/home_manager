import { useState, useEffect, useCallback } from 'react';
import { Plus, Edit3, Trash2, FolderOpen, AlertTriangle } from 'lucide-react';
import * as categoriesApi from '../api/categories';
import type { Category } from '../types';
import Modal from '../components/Modal';
import ConfirmDialog from '../components/ConfirmDialog';
import { useToast } from '../components/Toast';

export default function CategoriesPage() {
  const [categories, setCategories] = useState<Category[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editing, setEditing] = useState<Category | null>(null);
  const [name, setName] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Category | null>(null);

  const { showToast } = useToast();

  const fetchData = useCallback(async () => {
    try {
      const data = await categoriesApi.getCategories();
      setCategories(data);
    } catch {
      showToast('加载分类失败', 'error');
    } finally {
      setLoading(false);
    }
  }, [showToast]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const openAdd = () => {
    setEditing(null);
    setName('');
    setModalOpen(true);
  };

  const openEdit = (cat: Category) => {
    setEditing(cat);
    setName(cat.name);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!name.trim()) {
      showToast('请输入分类名称', 'warning');
      return;
    }

    setSubmitting(true);
    try {
      if (editing) {
        await categoriesApi.updateCategory(editing.id, name.trim());
        showToast('分类已更新', 'success');
      } else {
        await categoriesApi.createCategory(name.trim());
        showToast('分类已添加', 'success');
      }
      setModalOpen(false);
      fetchData();
    } catch {
      showToast('操作失败，请重试', 'error');
    } finally {
      setSubmitting(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      await categoriesApi.deleteCategory(deleteTarget.id);
      showToast('分类已删除', 'success');
      fetchData();
    } catch {
      showToast('删除失败', 'error');
    }
  };

  if (loading) {
    return (
      <div className="page-container">
        <div className="loading"><div className="spinner" /></div>
      </div>
    );
  }

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>分类管理</h1>
        <button className="btn btn-primary" onClick={openAdd}>
          <Plus size={18} />
          添加分类
        </button>
      </div>

      {categories.length > 0 ? (
        <div className="table-container">
          <table>
            <thead>
              <tr>
                <th>名称</th>
                <th>类型</th>
                <th>创建时间</th>
                <th style={{ width: 120 }}>操作</th>
              </tr>
            </thead>
            <tbody>
              {categories.map((cat) => (
                <tr key={cat.id}>
                  <td style={{ fontWeight: 500 }}>{cat.name}</td>
                  <td>
                    {cat.is_system ? (
                      <span className="badge badge-system">系统默认</span>
                    ) : (
                      <span className="badge badge-custom">自定义</span>
                    )}
                  </td>
                  <td className="text-muted" style={{ fontSize: '0.875rem' }}>
                    {new Date(cat.created_at).toLocaleDateString('zh-CN')}
                  </td>
                  <td>
                    <div className="flex gap-8">
                      <button className="btn btn-ghost btn-sm" onClick={() => openEdit(cat)}>
                        <Edit3 size={15} />
                      </button>
                      {!cat.is_system && (
                        <button
                          className="btn btn-ghost btn-sm"
                          style={{ color: 'var(--color-danger)' }}
                          onClick={() => setDeleteTarget(cat)}
                        >
                          <Trash2 size={15} />
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="empty-state">
          <FolderOpen size={48} />
          <h3>还没有分类</h3>
          <p>点击上方按钮添加你的第一个分类</p>
          <button className="btn btn-primary" onClick={openAdd}>
            <Plus size={18} />
            添加分类
          </button>
        </div>
      )}

      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        title={editing ? '编辑分类' : '添加分类'}
        footer={
          <>
            <button className="btn btn-secondary" onClick={() => setModalOpen(false)}>取消</button>
            <button className="btn btn-primary" onClick={handleSave} disabled={submitting}>
              {submitting ? '保存中...' : '保存'}
            </button>
          </>
        }
      >
        <div className="form-group">
          <label>分类名称</label>
          <input
            type="text"
            placeholder="请输入分类名称"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />
        </div>
      </Modal>

      <ConfirmDialog
        isOpen={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        onConfirm={handleDelete}
        title="确认删除"
        message={`确定要删除分类「${deleteTarget?.name}」吗？此操作不可撤销。`}
        confirmText="删除"
        variant="danger"
        icon={<AlertTriangle size={32} style={{ color: 'var(--color-danger)' }} />}
      />
    </div>
  );
}
