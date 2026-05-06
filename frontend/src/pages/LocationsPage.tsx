import { useState, useEffect, useCallback } from 'react';
import { Plus, Edit3, Trash2, MapPin, AlertTriangle } from 'lucide-react';
import * as locationsApi from '../api/locations';
import type { Location } from '../types';
import Modal from '../components/Modal';
import ConfirmDialog from '../components/ConfirmDialog';
import { useToast } from '../components/Toast';

export default function LocationsPage() {
  const [locations, setLocations] = useState<Location[]>([]);
  const [loading, setLoading] = useState(true);
  const [modalOpen, setModalOpen] = useState(false);
  const [editing, setEditing] = useState<Location | null>(null);
  const [name, setName] = useState('');
  const [submitting, setSubmitting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Location | null>(null);

  const { showToast } = useToast();

  const fetchData = useCallback(async () => {
    try {
      const data = await locationsApi.getLocations();
      setLocations(data);
    } catch {
      showToast('加载地点失败', 'error');
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

  const openEdit = (loc: Location) => {
    setEditing(loc);
    setName(loc.name);
    setModalOpen(true);
  };

  const handleSave = async () => {
    if (!name.trim()) {
      showToast('请输入地点名称', 'warning');
      return;
    }

    setSubmitting(true);
    try {
      if (editing) {
        await locationsApi.updateLocation(editing.id, name.trim());
        showToast('地点已更新', 'success');
      } else {
        await locationsApi.createLocation(name.trim());
        showToast('地点已添加', 'success');
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
      await locationsApi.deleteLocation(deleteTarget.id);
      showToast('地点已删除', 'success');
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
        <h1>存放地点管理</h1>
        <button className="btn btn-primary" onClick={openAdd}>
          <Plus size={18} />
          添加地点
        </button>
      </div>

      {locations.length > 0 ? (
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
              {locations.map((loc) => (
                <tr key={loc.id}>
                  <td style={{ fontWeight: 500 }}>{loc.name}</td>
                  <td>
                    {loc.is_system ? (
                      <span className="badge badge-system">系统默认</span>
                    ) : (
                      <span className="badge badge-custom">自定义</span>
                    )}
                  </td>
                  <td className="text-muted" style={{ fontSize: '0.875rem' }}>
                    {new Date(loc.created_at).toLocaleDateString('zh-CN')}
                  </td>
                  <td>
                    <div className="flex gap-8">
                      <button className="btn btn-ghost btn-sm" onClick={() => openEdit(loc)}>
                        <Edit3 size={15} />
                      </button>
                      {!loc.is_system && (
                        <button
                          className="btn btn-ghost btn-sm"
                          style={{ color: 'var(--color-danger)' }}
                          onClick={() => setDeleteTarget(loc)}
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
          <MapPin size={48} />
          <h3>还没有存放地点</h3>
          <p>点击上方按钮添加你的第一个存放地点</p>
          <button className="btn btn-primary" onClick={openAdd}>
            <Plus size={18} />
            添加地点
          </button>
        </div>
      )}

      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        title={editing ? '编辑地点' : '添加地点'}
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
          <label>地点名称</label>
          <input
            type="text"
            placeholder="请输入地点名称"
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
        message={`确定要删除地点「${deleteTarget?.name}」吗？此操作不可撤销。`}
        confirmText="删除"
        variant="danger"
        icon={<AlertTriangle size={32} style={{ color: 'var(--color-danger)' }} />}
      />
    </div>
  );
}
