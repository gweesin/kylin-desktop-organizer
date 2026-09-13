#!/bin/bash
# 麒麟桌面整理助手 一键安装脚本（银河麒麟 V10）
set -e

APP_NAME="kylin-desktop-organizer"
BIN="/usr/local/bin/${APP_NAME}"
DESKTOP="/usr/share/applications/${APP_NAME}.desktop"

echo "==> 1/4 安装依赖（需要 sudo）"
sudo apt update
sudo apt install -y build-essential pkg-config libgtk-3-dev libcairo2-dev \
    libgdk-pixbuf2.0-dev libglib2.0-dev

echo "==> 2/4 编译（首次约 5~10 分钟）"
cargo build --release

echo "==> 3/4 安装二进制与启动器"
sudo cp "target/release/${APP_NAME}" "${BIN}"
sudo cp "packaging/${APP_NAME}.desktop" "${DESKTOP}"
sudo chmod +x "${BIN}"

echo "==> 4/4 完成"
echo ""
echo "已安装：${BIN}"
echo ""
echo "运行：${BIN}"
echo "（首次建议先隐藏系统桌面图标，见 README「隐藏系统桌面图标」章节）"
