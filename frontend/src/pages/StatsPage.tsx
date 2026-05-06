import { useState, useEffect } from 'react';
import { Users, Package, FolderOpen, MapPin, Layers } from 'lucide-react';
import * as statsApi from '../api/stats';
import type { SystemStats } from '../types';
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
  const [loading, setLoading] = useState(true);
  const { showToast } = useToast();

  useEffect(() => {
    async function loadStats() {
      try {
        const data = await statsApi.getSystemStats();
        setStats(data);
      } catch {
        showToast('加载统计数据失败', 'error');
      } finally {
        setLoading(false);
      }
    }
    loadStats();
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

  return (
    <div className="page-container">
      <div className="page-header">
        <h1>系统统计</h1>
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
    </div>
  );
}
