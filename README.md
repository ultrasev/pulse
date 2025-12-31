# Pulse

一个基于 Tauri 的轻量级系统监控桌面应用。

## 安装

```bash
brew install --cask ultrasev/tap/pulse
```

## 功能

- **系统监控** - 实时查看 CPU、内存、磁盘和网络使用情况
- **IP 信息** - 查询当前网络 IP 信息
- **图片上传** - 支持剪贴板图片上传与历史记录
- **Claude Models** - 管理 `~/.claude` 目录下的 Git 分支切换

## 技术栈

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [Astro](https://astro.build/) - 前端框架
- [Svelte](https://svelte.dev/) - UI 组件
- [Tailwind CSS](https://tailwindcss.com/) - 样式

## 开发

```bash
# 安装依赖
bun install

# 启动开发服务器
make dev

# 构建生产版本
make build
```

## 安装

```bash
# 构建后安装到 /Applications
make install
```

## 项目结构

```
├── app/
│   ├── components/     # Svelte 组件
│   ├── layouts/        # 页面布局
│   ├── pages/          # Astro 页面
│   └── styles/         # 样式文件
├── tauri/              # Tauri 后端 (Rust)
└── public/             # 静态资源
```

## 许可证

MIT
