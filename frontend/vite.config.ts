import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
    plugins: [react()],
    server: {
        port: 5173,
        proxy: {
            '/api': {
                target: 'http://localhost:3000',
                changeOrigin: true,
            },
        },
    },
    build: {
        // 启用文件哈希，实现长期缓存
        rollupOptions: {
            output: {
                // JS 文件添加哈希
                entryFileNames: 'js/[name]-[hash].js',
                chunkFileNames: 'js/[name]-[hash].js',
                // 资源文件（CSS、图片等）添加哈希
                assetFileNames: (assetInfo) => {
                    const info = assetInfo.name.split('.');
                    const ext = info[info.length - 1];
                    // CSS 文件放在 css 目录
                    if (ext === 'css') {
                        return 'css/[name]-[hash][extname]';
                    }
                    // 图片文件放在 images 目录
                    if (/\.(png|jpe?g|gif|svg|webp|ico)$/.test(assetInfo.name)) {
                        return 'images/[name]-[hash][extname]';
                    }
                    // 字体文件放在 fonts 目录
                    if (/\.(woff2?|eot|ttf|otf)$/.test(assetInfo.name)) {
                        return 'fonts/[name]-[hash][extname]';
                    }
                    // 其他资源放在 assets 目录
                    return 'assets/[name]-[hash][extname]';
                },
                // 代码分割 - 分离第三方库
                manualChunks: {
                    'react-vendor': ['react', 'react-dom'],
                    'router': ['react-router-dom'],
                    'lucide': ['lucide-react'],
                    'axios': ['axios'],
                },
            },
        },
        // 生成 manifest.json，方便后端识别最新资源
        manifest: true,
        // 清空输出目录
        emptyOutDir: true,
        // 生产环境移除 console 和 debugger
        minify: 'terser',
        terserOptions: {
            compress: {
                drop_console: true,
                drop_debugger: true,
            },
        },
        // gzip 压缩（配合 Nginx 启用）
        assetsInlineLimit: 4096,
    },
});
