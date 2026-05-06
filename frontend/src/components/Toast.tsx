import { useState, useEffect, useCallback, createContext, useContext, type ReactNode } from 'react';
import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-react';

type ToastType = 'success' | 'error' | 'info' | 'warning';

interface ToastItem {
  id: number;
  message: string;
  type: ToastType;
}

interface ToastContextType {
  showToast: (message: string, type?: ToastType) => void;
}

const ToastContext = createContext<ToastContextType | null>(null);

let nextId = 0;

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<ToastItem[]>([]);

  const showToast = useCallback((message: string, type: ToastType = 'info') => {
    const id = nextId++;
    setToasts((prev) => [...prev, { id, message, type }]);
  }, []);

  const removeToast = useCallback((id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <div className="toast-container">
        {toasts.map((toast) => (
          <Toast key={toast.id} toast={toast} onClose={() => removeToast(toast.id)} />
        ))}
      </div>
    </ToastContext.Provider>
  );
}

export function useToast() {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error('useToast must be used within a ToastProvider');
  }
  return context;
}

function Toast({ toast, onClose }: { toast: ToastItem; onClose: () => void }) {
  const [isExiting, setIsExiting] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => {
      setIsExiting(true);
      setTimeout(onClose, 300);
    }, 3000);
    return () => clearTimeout(timer);
  }, [onClose]);

  const icons = {
    success: <CheckCircle size={18} style={{ color: 'var(--color-success)' }} />,
    error: <AlertCircle size={18} style={{ color: 'var(--color-danger)' }} />,
    info: <Info size={18} style={{ color: 'var(--color-info)' }} />,
    warning: <AlertTriangle size={18} style={{ color: 'var(--color-warning)' }} />,
  };

  return (
    <div
      className={`toast toast-${toast.type}`}
      style={isExiting ? { animation: 'toastOut 0.3s ease forwards' } : undefined}
    >
      {icons[toast.type]}
      <span style={{ flex: 1 }}>{toast.message}</span>
      <button className="toast-close" onClick={() => { setIsExiting(true); setTimeout(onClose, 300); }}>
        <X size={14} />
      </button>
    </div>
  );
}
