import React, { useState, useEffect, useMemo, useCallback } from 'react';
import {
    Plus, Search, Edit3, Trash2, ChevronDown, ChevronUp,
    Package, Calendar, AlertTriangle, MapPin
} from 'lucide-react';
import * as itemsApi from '../api/items';
import * as categoriesApi from '../api/categories';
import * as locationsApi from '../api/locations';
import * as batchesApi from '../api/batches';
import type { Item, Category, Location, Batch, CreateItemRequest, CreateBatchRequest } from '../types';
import Modal from '../components/Modal';
import ConfirmDialog from '../components/ConfirmDialog';
import { useToast } from '../components/Toast';

function formatDate(dateStr: string | null): string {
    if (!dateStr) return '无过期日期';
    return new Date(dateStr).toLocaleDateString('zh-CN');
}

function getExpiryStatus(dateStr: string | null): 'expired' | 'expiring-soon' | 'normal' {
    if (!dateStr) return 'normal';
    const now = new Date();
    const expiry = new Date(dateStr);
    const diffDays = (expiry.getTime() - now.getTime()) / (1000 * 60 * 60 * 24);
    if (diffDays < 0) return 'expired';
    if (diffDays <= 30) return 'expiring-soon';
    return 'normal';
}

function getTotalQuantity(item: Item): number {
    return item.batches.reduce((sum, b) => sum + b.quantity, 0);
}

function getNearestExpiry(item: Item): string | null {
    const withDates = item.batches
        .filter((b) => b.expiry_date)
        .sort((a, b) => new Date(a.expiry_date!).getTime() - new Date(b.expiry_date!).getTime());
    return withDates.length > 0 ? withDates[0].expiry_date : null;
}

export default function DashboardPage() {
    const [items, setItems] = useState<Item[]>([]);
    const [categories, setCategories] = useState<Category[]>([]);
    const [locations, setLocations] = useState<Location[]>([]);
    const [loading, setLoading] = useState(true);
    const [search, setSearch] = useState('');
    const [filterCategory, setFilterCategory] = useState('');
    const [filterLocation, setFilterLocation] = useState('');
    const [expandedItem, setExpandedItem] = useState<string | null>(null);

    // Item modal
    const [itemModalOpen, setItemModalOpen] = useState(false);
    const [editingItem, setEditingItem] = useState<Item | null>(null);
    const [itemName, setItemName] = useState('');
    const [itemCategoryId, setItemCategoryId] = useState('');
    const [itemLocationId, setItemLocationId] = useState('');
    const [itemSubmitting, setItemSubmitting] = useState(false);
    // 多批次支持
    interface ItemBatch {
        id?: string;
        quantity: string;
        expiry_date: string;
    }
    const [itemBatches, setItemBatches] = useState<ItemBatch[]>([]);

    // Batch modal
    const [batchModalOpen, setBatchModalOpen] = useState(false);
    const [batchItemId, setBatchItemId] = useState('');
    const [editingBatch, setEditingBatch] = useState<Batch | null>(null);
    const [batchQuantity, setBatchQuantity] = useState('');
    const [batchExpiry, setBatchExpiry] = useState('');
    const [batchSubmitting, setBatchSubmitting] = useState(false);

    // Delete
    const [deleteTarget, setDeleteTarget] = useState<{ type: 'item' | 'batch'; id: string; name: string } | null>(null);

    const { showToast } = useToast();

    const fetchData = useCallback(async () => {
        try {
            const [itemsData, catsData, locsData] = await Promise.all([
                itemsApi.getItems(),
                categoriesApi.getCategories(),
                locationsApi.getLocations(),
            ]);
            setItems(itemsData);
            setCategories(catsData);
            setLocations(locsData);
        } catch {
            showToast('加载数据失败', 'error');
        } finally {
            setLoading(false);
        }
    }, [showToast]);

    useEffect(() => {
        fetchData();
    }, [fetchData]);

    const filteredItems = useMemo(() => {
        return items.filter((item) => {
            const matchName = !search || item.name.toLowerCase().includes(search.toLowerCase());
            const matchCat = !filterCategory || item.category_id === Number(filterCategory);
            const matchLoc = !filterLocation || item.location_id === Number(filterLocation);
            return matchName && matchCat && matchLoc;
        });
    }, [items, search, filterCategory, filterLocation]);

    // Item CRUD
    const openAddItem = () => {
        setEditingItem(null);
        setItemName('');
        setItemCategoryId(categories[0]?.id?.toString() || '');
        setItemLocationId(locations[0]?.id?.toString() || '');
        // 初始化一个空批次
        setItemBatches([{ quantity: '', expiry_date: '' }]);
        setItemModalOpen(true);
    };

    const openEditItem = (item: Item) => {
        setEditingItem(item);
        setItemName(item.name);
        setItemCategoryId(item.category_id.toString());
        setItemLocationId(item.location_id.toString());
        // 将现有批次转换为表单格式
        if (item.batches.length > 0) {
            setItemBatches(item.batches.map(batch => ({
                id: batch.id,
                quantity: batch.quantity.toString(),
                expiry_date: batch.expiry_date || ''
            })));
        } else {
            setItemBatches([{ quantity: '', expiry_date: '' }]);
        }
        setItemModalOpen(true);
    };

    // 添加批次行
    const addItemBatch = () => {
        setItemBatches([...itemBatches, { quantity: '', expiry_date: '' }]);
    };

    // 删除批次行
    const removeItemBatch = (index: number) => {
        if (itemBatches.length > 1) {
            setItemBatches(itemBatches.filter((_, i) => i !== index));
        }
    };

    // 更新批次字段
    const updateItemBatchField = (index: number, field: 'quantity' | 'expiry_date', value: string) => {
        const newBatches = [...itemBatches];
        newBatches[index] = { ...newBatches[index], [field]: value };
        setItemBatches(newBatches);
    };

    const handleSaveItem = async () => {
        if (!itemName.trim()) {
            showToast('请输入物品名称', 'warning');
            return;
        }
        if (!itemCategoryId || !itemLocationId) {
            showToast('请选择分类和地点', 'warning');
            return;
        }

        // 验证批次信息
        const validBatches = itemBatches.filter(batch => batch.quantity && parseInt(batch.quantity) > 0);
        if (validBatches.length === 0) {
            showToast('请至少添加一个有效的批次（数量必须大于0）', 'warning');
            return;
        }

        setItemSubmitting(true);
        try {
            const data: CreateItemRequest = {
                name: itemName.trim(),
                category_id: Number(itemCategoryId),
                location_id: Number(itemLocationId),
            };

            if (editingItem) {
                await itemsApi.updateItem(editingItem.id, data);

                // 更新批次：先删除不再存在的批次，再更新或创建批次
                const existingBatchIds = editingItem.batches.map(b => b.id);
                const newBatchIds = validBatches.filter(b => b.id).map(b => b.id!);

                // 删除已移除的批次
                for (const batchId of existingBatchIds) {
                    if (!newBatchIds.includes(batchId)) {
                        await batchesApi.deleteBatch(batchId);
                    }
                }

                // 更新或创建批次
                for (const batch of validBatches) {
                    const batchData = {
                        quantity: parseInt(batch.quantity),
                        expiry_date: batch.expiry_date || null
                    };
                    if (batch.id) {
                        await batchesApi.updateBatch(batch.id, batchData);
                    } else {
                        await batchesApi.createBatch(editingItem.id, batchData);
                    }
                }
                showToast('物品已更新', 'success');
            } else {
                const createdItem = await itemsApi.createItem(data);
                // 创建所有批次
                for (const batch of validBatches) {
                    await batchesApi.createBatch(
                        createdItem.id,
                        { quantity: parseInt(batch.quantity), expiry_date: batch.expiry_date || null }
                    );
                }
                showToast('物品已添加', 'success');
            }
            setItemModalOpen(false);
            fetchData();
        } catch {
            showToast('操作失败，请重试', 'error');
        } finally {
            setItemSubmitting(false);
        }
    };

    const handleDeleteItem = async () => {
        if (!deleteTarget || deleteTarget.type !== 'item') return;
        try {
            await itemsApi.deleteItem(deleteTarget.id);
            showToast('物品已删除', 'success');
            fetchData();
        } catch {
            showToast('删除失败', 'error');
        }
    };

    // Batch CRUD
    const openAddBatch = (itemId: string) => {
        setBatchItemId(itemId);
        setEditingBatch(null);
        setBatchQuantity('');
        setBatchExpiry('');
        setBatchModalOpen(true);
    };

    const openEditBatch = (batch: Batch, itemId: string) => {
        setBatchItemId(itemId);
        setEditingBatch(batch);
        setBatchQuantity(batch.quantity.toString());
        setBatchExpiry(batch.expiry_date ? batch.expiry_date.split('T')[0] : '');
        setBatchModalOpen(true);
    };

    const handleSaveBatch = async () => {
        if (!batchQuantity || Number(batchQuantity) <= 0) {
            showToast('请输入有效数量', 'warning');
            return;
        }

        setBatchSubmitting(true);
        try {
            const data: CreateBatchRequest = {
                quantity: Number(batchQuantity),
                expiry_date: batchExpiry || null,
            };

            if (editingBatch) {
                await batchesApi.updateBatch(editingBatch.id, data);
                showToast('批次已更新', 'success');
            } else {
                await batchesApi.createBatch(batchItemId, data);
                showToast('批次已添加', 'success');
            }
            setBatchModalOpen(false);
            fetchData();
        } catch {
            showToast('操作失败，请重试', 'error');
        } finally {
            setBatchSubmitting(false);
        }
    };

    const handleDeleteBatch = async () => {
        if (!deleteTarget || deleteTarget.type !== 'batch') return;
        try {
            await batchesApi.deleteBatch(deleteTarget.id);
            showToast('批次已删除', 'success');
            fetchData();
        } catch {
            showToast('删除失败', 'error');
        }
    };

    if (loading) {
        return (
            <div className="page-container">
                <div className="loading">
                    <div className="spinner" />
                </div>
            </div>
        );
    }

    return (
        <div className="page-container">
            <div className="page-header">
                <h1>我的物品</h1>
                <button className="btn btn-primary" onClick={openAddItem}>
                    <Plus size={18} />
                    添加物品
                </button>
            </div>

            {/* Filter Bar */}
            <div className="filter-bar">
                <div style={{ position: 'relative', flex: 1, maxWidth: 300 }}>
                    <Search size={16} style={{ position: 'absolute', left: 12, top: '50%', transform: 'translateY(-50%)', color: 'var(--text-light)' }} />
                    <input
                        type="text"
                        placeholder="搜索物品名称..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        style={{ paddingLeft: 36 }}
                    />
                </div>
                <select value={filterCategory} onChange={(e) => setFilterCategory(e.target.value)}>
                    <option value="">全部分类</option>
                    {categories.map((c) => (
                        <option key={c.id} value={c.id}>{c.name}</option>
                    ))}
                </select>
                <select value={filterLocation} onChange={(e) => setFilterLocation(e.target.value)}>
                    <option value="">全部地点</option>
                    {locations.map((l) => (
                        <option key={l.id} value={l.id}>{l.name}</option>
                    ))}
                </select>
            </div>

            {/* Desktop Table */}
            {filteredItems.length > 0 ? (
                <>
                    <div className="table-container">
                        <table>
                            <thead>
                                <tr>
                                    <th>名称</th>
                                    <th>分类</th>
                                    <th>地点</th>
                                    <th>总数量</th>
                                    <th>最近过期</th>
                                    <th>操作</th>
                                </tr>
                            </thead>
                            <tbody>
                                {filteredItems.map((item) => {
                                    const isExpanded = expandedItem === item.id;
                                    const totalQty = getTotalQuantity(item);
                                    const nearestExpiry = getNearestExpiry(item);
                                    const expiryStatus = getExpiryStatus(nearestExpiry);

                                    return (
                                        <React.Fragment key={item.id}>
                                            <tr>
                                                <td>
                                                    <button
                                                        className="btn btn-ghost btn-sm"
                                                        onClick={() => setExpandedItem(isExpanded ? null : item.id)}
                                                        style={{ display: 'flex', alignItems: 'center', gap: 6, padding: '4px 8px' }}
                                                    >
                                                        {isExpanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
                                                        <span style={{ fontWeight: 600 }}>{item.name}</span>
                                                    </button>
                                                </td>
                                                <td>{item.category_name}</td>
                                                <td>{item.location_name}</td>
                                                <td>
                                                    <span style={{ fontWeight: 600 }}>{totalQty}</span>
                                                </td>
                                                <td>
                                                    <span className={`batch-expiry ${expiryStatus}`}>
                                                        {formatDate(nearestExpiry)}
                                                    </span>
                                                </td>
                                                <td>
                                                    <div className="flex gap-8">
                                                        <button className="btn btn-ghost btn-sm" onClick={() => openEditItem(item)}>
                                                            <Edit3 size={15} />
                                                        </button>
                                                        <button
                                                            className="btn btn-ghost btn-sm"
                                                            style={{ color: 'var(--color-danger)' }}
                                                            onClick={() => setDeleteTarget({ type: 'item', id: item.id, name: item.name })}
                                                        >
                                                            <Trash2 size={15} />
                                                        </button>
                                                    </div>
                                                </td>
                                            </tr>
                                            {isExpanded && (
                                                <tr>
                                                    <td colSpan={6} style={{ padding: '8px 16px 16px', backgroundColor: '#FAF8F5' }}>
                                                        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                                                            <span style={{ fontWeight: 600, fontSize: '0.875rem', color: 'var(--text-secondary)' }}>
                                                                批次明细
                                                            </span>
                                                            <button className="btn btn-primary btn-sm" onClick={() => openAddBatch(item.id)}>
                                                                <Plus size={14} />
                                                                添加批次
                                                            </button>
                                                        </div>
                                                        {item.batches.length === 0 ? (
                                                            <p style={{ color: 'var(--text-light)', fontSize: '0.875rem', padding: '12px 0' }}>
                                                                暂无批次记录
                                                            </p>
                                                        ) : (
                                                            <div className="batch-list">
                                                                {item.batches.map((batch) => {
                                                                    const bStatus = getExpiryStatus(batch.expiry_date);
                                                                    return (
                                                                        <div key={batch.id} className="batch-item">
                                                                            <div className="batch-info">
                                                                                <span className="batch-quantity">数量: {batch.quantity}</span>
                                                                                <span className={`batch-expiry ${bStatus}`}>
                                                                                    {formatDate(batch.expiry_date)}
                                                                                </span>
                                                                            </div>
                                                                            <div className="batch-actions">
                                                                                <button className="btn btn-ghost btn-sm" onClick={() => openEditBatch(batch, item.id)}>
                                                                                    <Edit3 size={14} />
                                                                                </button>
                                                                                <button
                                                                                    className="btn btn-ghost btn-sm"
                                                                                    style={{ color: 'var(--color-danger)' }}
                                                                                    onClick={() => setDeleteTarget({ type: 'batch', id: batch.id, name: `${item.name} 的批次` })}
                                                                                >
                                                                                    <Trash2 size={14} />
                                                                                </button>
                                                                            </div>
                                                                        </div>
                                                                    );
                                                                })}
                                                            </div>
                                                        )}
                                                    </td>
                                                </tr>
                                            )}
                                        </React.Fragment>
                                    );
                                })}
                            </tbody>
                        </table>
                    </div>

                    {/* Mobile Card List */}
                    <div className="mobile-item-list">
                        {filteredItems.map((item) => {
                            const totalQty = getTotalQuantity(item);
                            const nearestExpiry = getNearestExpiry(item);
                            const expiryStatus = getExpiryStatus(nearestExpiry);
                            const isExpanded = expandedItem === item.id;

                            return (
                                <div key={item.id} className="item-card">
                                    <div className="item-card-header">
                                        <span className="item-card-name">{item.name}</span>
                                        <div className="item-card-actions">
                                            <button className="btn btn-ghost btn-sm" onClick={() => openEditItem(item)}>
                                                <Edit3 size={15} />
                                            </button>
                                            <button
                                                className="btn btn-ghost btn-sm"
                                                style={{ color: 'var(--color-danger)' }}
                                                onClick={() => setDeleteTarget({ type: 'item', id: item.id, name: item.name })}
                                            >
                                                <Trash2 size={15} />
                                            </button>
                                        </div>
                                    </div>
                                    <div className="item-card-meta">
                                        <span><Package size={14} /> {item.category_name}</span>
                                        <span><MapPin size={14} /> {item.location_name}</span>
                                        <span style={{ fontWeight: 600 }}>数量: {totalQty}</span>
                                    </div>
                                    {nearestExpiry && (
                                        <div style={{ marginBottom: 8 }}>
                                            <span className={`batch-expiry ${expiryStatus}`} style={{ fontSize: '0.8125rem' }}>
                                                <Calendar size={12} style={{ marginRight: 4, verticalAlign: 'middle' }} />
                                                {formatDate(nearestExpiry)}
                                            </span>
                                        </div>
                                    )}
                                    <button
                                        className="btn btn-ghost btn-sm w-full"
                                        onClick={() => setExpandedItem(isExpanded ? null : item.id)}
                                        style={{ justifyContent: 'center' }}
                                    >
                                        {isExpanded ? <ChevronUp size={16} /> : <ChevronDown size={16} />}
                                        {isExpanded ? '收起批次' : '查看批次'}
                                    </button>
                                    {isExpanded && (
                                        <div style={{ marginTop: 8 }}>
                                            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
                                                <span style={{ fontWeight: 600, fontSize: '0.875rem', color: 'var(--text-secondary)' }}>
                                                    批次明细
                                                </span>
                                                <button className="btn btn-primary btn-sm" onClick={() => openAddBatch(item.id)}>
                                                    <Plus size={14} />
                                                    添加
                                                </button>
                                            </div>
                                            {item.batches.length === 0 ? (
                                                <p style={{ color: 'var(--text-light)', fontSize: '0.875rem', padding: '12px 0' }}>
                                                    暂无批次记录
                                                </p>
                                            ) : (
                                                <div className="batch-list">
                                                    {item.batches.map((batch) => {
                                                        const bStatus = getExpiryStatus(batch.expiry_date);
                                                        return (
                                                            <div key={batch.id} className="batch-item">
                                                                <div className="batch-info">
                                                                    <span className="batch-quantity">数量: {batch.quantity}</span>
                                                                    <span className={`batch-expiry ${bStatus}`}>
                                                                        {formatDate(batch.expiry_date)}
                                                                    </span>
                                                                </div>
                                                                <div className="batch-actions">
                                                                    <button className="btn btn-ghost btn-sm" onClick={() => openEditBatch(batch, item.id)}>
                                                                        <Edit3 size={14} />
                                                                    </button>
                                                                    <button
                                                                        className="btn btn-ghost btn-sm"
                                                                        style={{ color: 'var(--color-danger)' }}
                                                                        onClick={() => setDeleteTarget({ type: 'batch', id: batch.id, name: `${item.name} 的批次` })}
                                                                    >
                                                                        <Trash2 size={14} />
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </div>
                            );
                        })}
                    </div>
                </>
            ) : (
                <div className="empty-state">
                    <Package size={48} />
                    <h3>{search || filterCategory || filterLocation ? '没有找到匹配的物品' : '还没有物品'}</h3>
                    <p>{search || filterCategory || filterLocation ? '试试调整搜索条件' : '点击上方按钮添加你的第一个物品'}</p>
                    {!search && !filterCategory && !filterLocation && (
                        <button className="btn btn-primary" onClick={openAddItem}>
                            <Plus size={18} />
                            添加物品
                        </button>
                    )}
                </div>
            )}

            {/* Item Modal */}
            <Modal
                isOpen={itemModalOpen}
                onClose={() => setItemModalOpen(false)}
                title={editingItem ? '编辑物品' : '添加物品'}
                footer={
                    <>
                        <button className="btn btn-secondary" onClick={() => setItemModalOpen(false)}>取消</button>
                        <button className="btn btn-primary" onClick={handleSaveItem} disabled={itemSubmitting}>
                            {itemSubmitting ? '保存中...' : '保存'}
                        </button>
                    </>
                }
            >
                <div className="form-group">
                    <label>物品名称</label>
                    <input
                        type="text"
                        placeholder="请输入物品名称"
                        value={itemName}
                        onChange={(e) => setItemName(e.target.value)}
                        autoFocus
                    />
                </div>
                <div className="form-row">
                    <div className="form-group">
                        <label>分类</label>
                        <select value={itemCategoryId} onChange={(e) => setItemCategoryId(e.target.value)}>
                            <option value="">请选择分类</option>
                            {categories.map((c) => (
                                <option key={c.id} value={c.id}>{c.name}</option>
                            ))}
                        </select>
                    </div>
                    <div className="form-group">
                        <label>存放地点</label>
                        <select value={itemLocationId} onChange={(e) => setItemLocationId(e.target.value)}>
                            <option value="">请选择地点</option>
                            {locations.map((l) => (
                                <option key={l.id} value={l.id}>{l.name}</option>
                            ))}
                        </select>
                    </div>
                </div>

                {/* 多批次区域 */}
                <div className="form-group">
                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 12 }}>
                        <label>批次信息</label>
                        <button
                            className="btn btn-primary btn-sm"
                            onClick={addItemBatch}
                            style={{ padding: '4px 12px', fontSize: '0.8125rem' }}
                        >
                            <Plus size={14} />
                            添加批次
                        </button>
                    </div>

                    {/* 批次列表 */}
                    <div className="batch-form-list">
                        {itemBatches.map((batch, index) => (
                            <div key={index} className="batch-form-row">
                                <div className="form-group" style={{ flex: 1, marginRight: 12 }}>
                                    <label>数量</label>
                                    <input
                                        type="number"
                                        min="1"
                                        placeholder="请输入数量"
                                        value={batch.quantity}
                                        onChange={(e) => updateItemBatchField(index, 'quantity', e.target.value)}
                                    />
                                </div>
                                <div className="form-group" style={{ flex: 1, marginRight: itemBatches.length > 1 ? 12 : 0 }}>
                                    <label>保质期（可选）</label>
                                    <input
                                        type="date"
                                        value={batch.expiry_date}
                                        onChange={(e) => updateItemBatchField(index, 'expiry_date', e.target.value)}
                                    />
                                </div>
                                {itemBatches.length > 1 && (
                                    <button
                                        className="btn btn-ghost btn-sm"
                                        style={{ color: 'var(--color-danger)', alignSelf: 'flex-end', marginBottom: 8 }}
                                        onClick={() => removeItemBatch(index)}
                                    >
                                        <Trash2 size={16} />
                                    </button>
                                )}
                            </div>
                        ))}
                    </div>
                </div>
            </Modal>

            {/* Batch Modal */}
            <Modal
                isOpen={batchModalOpen}
                onClose={() => setBatchModalOpen(false)}
                title={editingBatch ? '编辑批次' : '添加批次'}
                footer={
                    <>
                        <button className="btn btn-secondary" onClick={() => setBatchModalOpen(false)}>取消</button>
                        <button className="btn btn-primary" onClick={handleSaveBatch} disabled={batchSubmitting}>
                            {batchSubmitting ? '保存中...' : '保存'}
                        </button>
                    </>
                }
            >
                <div className="form-group">
                    <label>数量</label>
                    <input
                        type="number"
                        min="1"
                        placeholder="请输入数量"
                        value={batchQuantity}
                        onChange={(e) => setBatchQuantity(e.target.value)}
                        autoFocus
                    />
                </div>
                <div className="form-group">
                    <label>过期日期（可选）</label>
                    <input
                        type="date"
                        value={batchExpiry}
                        onChange={(e) => setBatchExpiry(e.target.value)}
                    />
                </div>
            </Modal>

            {/* Delete Confirm */}
            <ConfirmDialog
                isOpen={!!deleteTarget}
                onClose={() => setDeleteTarget(null)}
                onConfirm={deleteTarget?.type === 'item' ? handleDeleteItem : handleDeleteBatch}
                title="确认删除"
                message={`确定要删除「${deleteTarget?.name}」吗？此操作不可撤销。`}
                confirmText="删除"
                variant="danger"
                icon={<AlertTriangle size={32} style={{ color: 'var(--color-danger)' }} />}
            />
        </div>
    );
}


