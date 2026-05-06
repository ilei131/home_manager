import { useState, useEffect } from 'react';
import { Package, FolderOpen, MapPin, Layers, AlertTriangle, Calendar, ChevronRight } from 'lucide-react';
import * as statsApi from '../api/stats';
import * as itemsApi from '../api/items';
import type { SystemStats, Item } from '../types';
import { useToast } from '../components/Toast';

interface StatCardData {
    label: string;
    value: number;
    icon: React.ReactNode;
    color: string;
    bgColor: string;
}

export default function HomePage() {
    const [stats, setStats] = useState<SystemStats | null>(null);
    const [expiringItems, setExpiringItems] = useState<Item[]>([]);
    const [loading, setLoading] = useState(true);
    const { showToast } = useToast();

    useEffect(() => {
        async function loadData() {
            try {
                const [userStats, items] = await Promise.all([
                    statsApi.getUserStats(),
                    itemsApi.getExpiringItems(),
                ]);
                setStats(userStats);
                setExpiringItems(items);
            } catch {
                showToast('加载数据失败', 'error');
            } finally {
                setLoading(false);
            }
        }
        loadData();
    }, [showToast]);

    if (loading) {
        return (
            <div className="page-container">
                <div className="loading"><div className="spinner" /></div>
            </div>
        );
    }

    const statCards: StatCardData[] = [
        {
            label: '我的物品',
            value: stats?.total_items || 0,
            icon: <Package size={24} />,
            color: '#4CAF50',
            bgColor: 'rgba(76, 175, 80, 0.1)',
        },
        {
            label: '分类数量',
            value: stats?.total_categories || 0,
            icon: <FolderOpen size={24} />,
            color: '#F39C12',
            bgColor: 'rgba(243, 156, 18, 0.1)',
        },
        {
            label: '存放地点',
            value: stats?.total_locations || 0,
            icon: <MapPin size={24} />,
            color: '#E74C3C',
            bgColor: 'rgba(231, 76, 60, 0.1)',
        },
        {
            label: '物品批次',
            value: stats?.total_batches || 0,
            icon: <Layers size={24} />,
            color: '#9B59B6',
            bgColor: 'rgba(155, 89, 182, 0.1)',
        },
    ];

    const getDaysUntilExpiry = (expiryDate: string | null): number | null => {
        if (!expiryDate) return null;
        const expiry = new Date(expiryDate);
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        expiry.setHours(0, 0, 0, 0);
        const diffTime = expiry.getTime() - today.getTime();
        return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    };

    const getExpiryStatus = (days: number | null): { text: string; color: string } => {
        if (days === null) return { text: '无保质期', color: '#999' };
        if (days < 0) return { text: '已过期', color: '#E74C3C' };
        if (days === 0) return { text: '今日过期', color: '#E74C3C' };
        if (days <= 3) return { text: `${days}天后过期`, color: '#F39C12' };
        if (days <= 7) return { text: `${days}天后过期`, color: '#F1C40F' };
        return { text: `${days}天后过期`, color: '#4CAF50' };
    };

    return (
        <div className="page-container home-page">
            <div className="page-header">
                <h1>欢迎回来</h1>
                {/* <p className="page-subtitle">查看您的物品概览和临期提醒</p> */}
            </div>

            <div className="stats-grid">
                {statCards.map((card) => (
                    <div key={card.label} className="stat-card">
                        <div
                            className="stat-icon"
                            style={{ backgroundColor: card.bgColor, color: card.color }}
                        >
                            {card.icon}
                        </div>
                        <div className="stat-value">{card.value}</div>
                        <div className="stat-label">{card.label}</div>
                    </div>
                ))}
            </div>

            <div className="section">
                <div className="section-header">
                    <div className="flex items-center gap-2">
                        <AlertTriangle size={20} style={{ color: '#F39C12' }} />
                        <h2>临期物品提醒</h2>
                    </div>
                    <span className="section-badge">{expiringItems.length} 件</span>
                </div>

                {expiringItems.length > 0 ? (
                    <div className="expiring-list">
                        {expiringItems.map((item) => {
                            const latestBatch = item.batches
                                .filter(b => b.expiry_date)
                                .sort((a, b) => new Date(a.expiry_date || '').getTime() - new Date(b.expiry_date || '').getTime())[0];
                            const days = latestBatch ? getDaysUntilExpiry(latestBatch.expiry_date) : null;
                            const status = getExpiryStatus(days);

                            return (
                                <div key={item.id} className="expiring-item">
                                    <div className="expiring-item-info">
                                        <h3 className="expiring-item-name">{item.name}</h3>
                                        <div className="expiring-item-meta">
                                            <span className="meta-tag">{item.category_name}</span>
                                            <span className="meta-tag">{item.location_name}</span>
                                        </div>
                                    </div>
                                    <div className="expiring-item-status">
                                        <div className="flex items-center gap-1" style={{ color: status.color }}>
                                            <Calendar size={14} />
                                            <span style={{ fontWeight: 500 }}>{status.text}</span>
                                        </div>
                                    </div>
                                    <ChevronRight size={18} className="expiring-item-arrow" />
                                </div>
                            );
                        })}
                    </div>
                ) : (
                    <div className="empty-state small">
                        <Calendar size={40} style={{ color: '#ddd' }} />
                        <h3>暂无临期物品</h3>
                        <p>您的物品都在保质期内</p>
                    </div>
                )}
            </div>
        </div>
    );
}
