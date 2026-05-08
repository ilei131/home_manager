import { useState, useEffect } from 'react';
import { Users, Package, FolderOpen, MapPin, Layers, Calendar, Shield } from 'lucide-react';
import * as statsApi from '../api/stats';
import type { SystemStats, User } from '../types';
import { useToast } from '../components/Toast';

interface StatCardData {
    label: string;
    value: number;
    icon: React.ReactNode;
    color: string;
    bgColor: string;
}

export default function StatsPage() {
    const [stats, setStats] = useState<SystemStats | null>(null);
    const [users, setUsers] = useState<User[]>([]);
    const [loading, setLoading] = useState(true);
    const { showToast } = useToast();

    useEffect(() => {
        async function loadData() {
            try {
                const [systemStats, userList] = await Promise.all([
                    statsApi.getSystemStats(),
                    statsApi.getUsers(),
                ]);
                setStats(systemStats);
                setUsers(userList);
            } catch (error) {
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

    if (!stats) {
        return (
            <div className="page-container">
                <div className="empty-state">
                    <Layers size={48} />
                    <h3>无法加载统计数据</h3>
                    <p>请稍后重试</p>
                </div>
            </div>
        );
    }

    const statCards: StatCardData[] = [
        {
            label: '总用户数',
            value: stats.total_users,
            icon: <Users size={24} />,
            color: '#4A90D9',
            bgColor: 'rgba(74, 144, 217, 0.1)',
        },
        {
            label: '总物品数',
            value: stats.total_items,
            icon: <Package size={24} />,
            color: '#4CAF50',
            bgColor: 'rgba(76, 175, 80, 0.1)',
        },
        {
            label: '总分类数',
            value: stats.total_categories,
            icon: <FolderOpen size={24} />,
            color: '#F39C12',
            bgColor: 'rgba(243, 156, 18, 0.1)',
        },
        {
            label: '总地点数',
            value: stats.total_locations,
            icon: <MapPin size={24} />,
            color: '#E74C3C',
            bgColor: 'rgba(231, 76, 60, 0.1)',
        },
        {
            label: '总批次数',
            value: stats.total_batches,
            icon: <Layers size={24} />,
            color: '#9B59B6',
            bgColor: 'rgba(155, 89, 182, 0.1)',
        },
    ];

    const formatDate = (dateString: string) => {
        return new Date(dateString).toLocaleDateString('zh-CN', {
            year: 'numeric',
            month: '2-digit',
            day: '2-digit',
            hour: '2-digit',
            minute: '2-digit',
        });
    };

    return (
        <div className="page-container">
            <div className="page-header">
                <h1>系统管理</h1>
                <p className="page-subtitle">查看系统统计信息和用户列表</p>
            </div>

            {/* 统计卡片 */}
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

            {/* 用户列表 */}
            <div className="section">
                <div className="section-header">
                    <h2>用户列表</h2>
                    <span className="section-badge">{users.length} 位用户</span>
                </div>

                <div className="table-container">
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th>用户名</th>
                                <th>创建时间</th>
                            </tr>
                        </thead>
                        <tbody>
                            {users.length > 0 ? (
                                users.map((user) => (
                                    <tr key={user.id}>
                                        <td className="user-info">
                                            <Users size={18} className="user-icon" />
                                            <span>{user.username}</span>
                                        </td>
                                        <td className="date-cell">
                                            <Calendar size={14} />
                                            <span>{formatDate(user.created_at)}</span>
                                        </td>
                                    </tr>
                                ))
                            ) : (
                                <tr>
                                    <td colSpan={2} className="empty-table">
                                        <Users size={32} />
                                        <p>暂无用户</p>
                                    </td>
                                </tr>
                            )}
                        </tbody>
                    </table>
                </div>

                <div className="mobile-user-list">
                    {users.length > 0 ? (
                        users.map((user) => (
                            <div key={user.id} className="user-card">
                                <div className="user-card-header">
                                    <div className="user-card-name">
                                        <Users size={18} />
                                        <span>{user.username}</span>
                                    </div>
                                    <span className={`role-badge ${user.role === 'admin' ? 'admin' : 'user'}`}>
                                        <Shield size={14} />
                                        {user.role === 'admin' ? '管理员' : '普通用户'}
                                    </span>
                                </div>
                                <div className="user-card-date">
                                    <Calendar size={14} />
                                    <span>注册时间：{formatDate(user.created_at)}</span>
                                </div>
                            </div>
                        ))
                    ) : (
                        <div className="empty-state">
                            <Users size={32} />
                            <p>暂无用户</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
