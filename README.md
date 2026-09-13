# 麒麟桌面整理助手（Kylin Desktop Organizer）

用 **Rust + GTK3 (gtk-rs)** 完整复刻腾讯桌面整理核心功能的国产桌面整理工具，专为 **银河麒麟 V10（UKUI 桌面 / X11）** 设计。

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange) ![GTK3](https://img.shields.io/badge/GTK-3.24-blue)

## 功能特性

| 功能 | 说明 |
|---|---|
| 🗂️ 一键整理 | 桌面图标自动分类（我的电脑 / 回收站 / 应用 / 文件夹 / 文件 / 图片视频 / 压缩包），每类生成一个盒子，带飞入动画 |
| 📦 盒子管理 | 创建、拖拽移动、折叠/展开、关闭、重命名；拖拽图标进/出盒子；自动网格重排 |
| 🎯 图标吸附 | 拖拽释放后自动吸附对齐网格（可调间距） |
| 🖱️ 双击整理 | 双击桌面空白处快速一键整理（可在设置中关闭） |
| 📋 右键菜单 | 桌面菜单（整理/新建盒子/扫描/设置/退出）、图标菜单（打开/显示位置/移入盒子/重命名/复制/删除/属性）、盒子菜单（重命名/添加/移除） |
| ⚙️ 设置面板 | 开机自启、浅色/深色主题、盒子样式（卡片/实心/简洁）、图标大小（40/48/56）、双击整理开关、显示桌面图标、显示文件 |
| 🖼️ 原生外观 | 半透明圆角盒子、系统图标主题提取、选中高亮、文件/文件夹/应用图标自动识别 |
| 💾 状态持久化 | 图标位置、盒子布局、设置全部保存到 `~/.config/kylin-desktop-organizer/config.json` |

## 界面预览

```
┌────────────────────────────────────────────────────────────┐
│  ┌─我的电脑───┐  ┌─浏览器────┐  ┌─文件夹────┐             │
│  │ (电脑图标)  │  │ (Firefox)  │  │ (文件夹1)  │             │
│  │            │  │            │  │ (文件夹2)  │             │
│  └────────────┘  └────────────┘  └────────────┘             │
│   [图标1] [图标2] [图标3]                                    │
│                                                              │
│   (桌面空白处右键 → 一键整理 / 新建盒子 / 设置)               │
└────────────────────────────────────────────────────────────┘
```

## 系统要求

- 银河麒麟 V10 SP1/SP2（UKUI 或兼容 X11 桌面环境）
- Rust 工具链 1.70+（`rustup` 或系统包）
- GTK3 开发库

## 编译运行（麒麟 V10）

### 1. 安装依赖

```bash
# 安装编译工具与 GTK3 开发库
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libcairo2-dev \
    libgdk-pixbuf2.0-dev libglib2.0-dev librust-gtk3-dev
```

> `libgtk-3-dev` 自带 gtk-rs 需要的系统库，`librust-gtk3-dev` 可选（加速 Rust GTK 绑定编译，没有也能编译，只是首次编译较慢）。

### 2. 获取源码并编译

```bash
git clone <本仓库地址>
cd linux-desktop-assistant

# 首次编译会下载 gtk-rs 等依赖（约 5~10 分钟）
cargo build --release
```

编译产物：`target/release/kylin-desktop-organizer`

### 3. 运行

```bash
./target/release/kylin-desktop-organizer
```

> ⚠️ **首次运行提示**：程序会创建一个覆盖整个屏幕的透明"桌面层"窗口。
> 若看不到效果，请先**隐藏系统自带桌面图标**（见下文），并确认处于 X11 会话：
> ```bash
> echo $XDG_SESSION_TYPE   # 应输出 x11
> ```

### 4. 开机自启（可选）

程序内「设置 → 开机自动启动」即可，或手动：

```bash
# 复制启动器并设为自启
cp packaging/kylin-desktop-organizer.desktop ~/.config/autostart/
chmod +x ~/.config/autostart/kylin-desktop-organizer.desktop
```

### 5. 添加到应用菜单（可选）

```bash
sudo cp packaging/kylin-desktop-organizer.desktop /usr/share/applications/
```

## 隐藏系统桌面图标（关键步骤）

为了让"桌面整理"的图标与盒子完全接管桌面，建议关闭系统自带的桌面图标显示。

### UKUI（麒麟默认）

```bash
# 方式一：图形界面
# 桌面右键 → 桌面设置 → 显示图标 → 关闭

# 方式二：命令（麒麟 V10 常用）
gsettings set org.ukui.peony.desktop show-desktop-icons false
```

### MATE（caja）

```bash
gsettings set org.mate.background show-desktop-icons false
```

### GNOME

```bash
gnome-extensions disable desktop-icons@csoriano
```

隐藏后重启程序或按 F5 刷新即可看到整理效果。

## 使用说明

| 操作 | 效果 |
|---|---|
| 双击图标 | 打开文件/文件夹/应用 |
| 单击图标 | 选中（高亮） |
| 拖拽图标 | 移动位置，释放自动吸附网格 |
| 拖拽图标到盒子 | 移入盒子（自动重排） |
| 拖拽图标出盒子 | 移出盒子回桌面 |
| 拖拽盒子标题栏 | 移动盒子 |
| 盒子 ─ / ▢ 按钮 | 折叠 / 展开 |
| 盒子 ✕ 按钮 | 关闭盒子（图标回到桌面） |
| 双击桌面空白 | 一键整理 |
| 桌面右键 | 整理 / 新建盒子 / 扫描桌面 / 设置 / 退出 |
| 图标右键 | 打开 / 显示位置 / 移入盒子 / 重命名 / 复制 / 删除 / 属性 |
| 盒子标题栏右键 | 重命名 / 添加 / 移除 |

## 配置说明

配置文件：`~/.config/kylin-desktop-organizer/config.json`

```json
{
  "first_run": false,
  "icon_size": 48,
  "grid_gap": 10,
  "double_click_organize": true,
  "auto_organize": false,
  "show_files": true,
  "show_desktop_icons": true,
  "theme": "light",
  "box_style": "card",
  "box_opacity": 0.88,
  "desktop_icons": [{ "path": "/home/user/Desktop/xx.desktop", "x": 20, "y": 16 }],
  "boxes": [{ "id": "auto-1", "title": "应用", "x": 20, "y": 16, "w": 320, "h": 240, "collapsed": false, "icons": [] }]
}
```

## 项目结构

```
src/
├── main.rs         # 入口：单实例检查、GTK 初始化
├── model.rs        # 配置模型与持久化（JSON）
├── window.rs       # 桌面层窗口：图标/盒子/拖拽/动画/菜单/整理
├── icon.rs         # 桌面图标控件（图标+文字，选中/拖拽）
├── icons_util.rs   # 图标渲染：系统主题图标提取、圆角背景
├── categorize.rs   # 分类规则引擎
├── launch.rs       # 打开/删除/显示位置（xdg-open/gio）
├── settings.rs     # 设置面板
└── theme.rs        # 浅色/深色主题调色板
packaging/
├── kylin-desktop-organizer.desktop   # 应用菜单/自启启动器
└── install.sh                        # 一键安装脚本
```

## 常见问题

**Q: 编译报错找不到 `pkg-config` 或 GTK 库？**
```bash
sudo apt install -y pkg-config libgtk-3-dev
```

**Q: 运行后桌面没有图标显示？**
- 确认系统桌面图标已隐藏（见上文）
- 确认在 X11 会话：`echo $XDG_SESSION_TYPE` 应为 `x11`
- 运行 `./target/release/kylin-desktop-organizer 2>&1 | tee /tmp/org.log` 查看日志

**Q: 图标不显示文字背景？**
图标文字背景跟随系统主题图标主题（`~/.local/share/icons` 或 `/usr/share/icons`），麒麟默认主题已内置绝大多数图标。未匹配时显示文件类型占位图标。

**Q: 多显示器支持？**
当前版本使用主显示器（primary screen），多显示器扩展桌面将在后续版本支持。

## 技术说明

- 采用 **GTK3 `WindowTypeHint::Desktop`** 窗口类型：窗口自动置于桌面底层、不抢焦点、不占任务栏，实现"贴在桌面上"的原生桌面层效果
- 窗口启用 **ARGB 透明**（`rgba_visual` + `app_paintable`），背景完全透明，仅绘制盒子与图标
- 图标图标从 **系统图标主题** 实时提取（`XDG_DATA_DIRS` 查找），自动适配麒麟主题风格
- 拖拽采用事件坐标增量计算，支持图标拖入/拖出盒子的完整生命周期
- 状态实时持久化，崩溃/重启后布局不丢失

## License

MIT
