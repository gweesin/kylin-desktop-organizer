#!/usr/bin/env bash
#
# 麒麟桌面整理助手 - 发布脚本
#
# 作用：一条命令完成「改版本号 -> 更新 Cargo.lock -> 提交 -> 打 tag -> 推送」，
#       推送 tag 后会触发 .github/workflows/release.yml 自动构建并发布 Release。
#
# 用法：
#   ./scripts/release.sh 1.1.0                # 交互确认后发布
#   ./scripts/release.sh v1.1.0 --yes         # 跳过确认
#   ./scripts/release.sh 1.1.0 --dry-run      # 只演示改动，不提交/不打 tag
#   ./scripts/release.sh 1.1.0 --no-push      # 本地提交并打 tag，但不推送
#
# 选项：
#   -y, --yes            不询问，直接执行
#   -n, --dry-run        预演：修改文件后展示 diff，然后还原，不提交
#   --no-push            不推送到远端（只做本地提交 + tag）
#   --skip-lock          不通过 cargo 刷新 Cargo.lock
#   --allow-branch       允许在非 main/master 分支发布
#   -h, --help           显示帮助

set -euo pipefail

APP_NAME="kylin-desktop-organizer"
MAIN_BRANCHES="main master"

# ---------- 输出helpers ----------
info()  { printf '\033[1;34m==>\033[0m %s\n' "$*"; }
warn()  { printf '\033[1;33m[!]\033[0m %s\n' "$*" >&2; }
error() { printf '\033[1;31m[x]\033[0m %s\n' "$*" >&2; }
die()   { error "$*"; exit 1; }

usage() {
    awk 'NR==1 { next }
         /^#/ { sub(/^# ?/, ""); print; next }
         { exit }' "$0"
    exit 0
}

# ---------- 参数解析 ----------
VERSION_RAW=""
ASSUME_YES=0
DRY_RUN=0
PUSH=1
REFRESH_LOCK=1
ALLOW_BRANCH=0

while [ $# -gt 0 ]; do
    case "$1" in
        -y|--yes)        ASSUME_YES=1 ;;
        -n|--dry-run)    DRY_RUN=1 ;;
        --no-push)       PUSH=0 ;;
        --skip-lock)     REFRESH_LOCK=0 ;;
        --allow-branch)  ALLOW_BRANCH=1 ;;
        -h|--help)       usage ;;
        -*)              die "未知选项：$1（用 --help 查看用法）" ;;
        *)
            [ -z "$VERSION_RAW" ] || die "只能指定一个版本号（多余参数：$1）"
            VERSION_RAW="$1"
            ;;
    esac
    shift
done

[ -n "$VERSION_RAW" ] || die "缺少版本号。用法：./scripts/release.sh <版本号>，例如 ./scripts/release.sh 1.1.0"

# 去掉可选的 v 前缀
VERSION="${VERSION_RAW#v}"
TAG="v${VERSION}"

# 语义化版本校验
if ! printf '%s' "$VERSION" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-+][0-9A-Za-z.-]+)?$'; then
    die "版本号格式不合法：${VERSION_RAW}（应形如 1.1.0 或 1.1.0-rc.1）"
fi

# ---------- 定位仓库根目录 ----------
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "当前目录不是 git 仓库"
cd "$REPO_ROOT"

CARGO_TOML="Cargo.toml"
CARGO_LOCK="Cargo.lock"
[ -f "$CARGO_TOML" ] || die "未找到 $CARGO_TOML"

BACKUP_TOML="$(mktemp)"
BACKUP_LOCK="$(mktemp)"
cp "$CARGO_TOML" "$BACKUP_TOML"
[ -f "$CARGO_LOCK" ] && cp "$CARGO_LOCK" "$BACKUP_LOCK"

rollback() {
    cp "$BACKUP_TOML" "$CARGO_TOML"
    [ -f "$CARGO_LOCK" ] && cp "$BACKUP_LOCK" "$CARGO_LOCK"
}
cleanup() {
    rm -f "$BACKUP_TOML" "$BACKUP_LOCK"
}
trap cleanup EXIT

# ---------- 前置检查 ----------
info "检查工作区状态"
if [ -n "$(git status --porcelain)" ]; then
    die "工作区有未提交的改动，请先提交或 stash 后再发布"
fi

CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [ "$ALLOW_BRANCH" -eq 0 ]; then
    case " $MAIN_BRANCHES " in
        *" $CURRENT_BRANCH "*) ;;
        *) die "当前分支为 '${CURRENT_BRANCH}'，非 main/master。如确需发布请加 --allow-branch" ;;
    esac
fi

PREV_VERSION="$(grep -m1 -E '^version[[:space:]]*=' "$CARGO_TOML" | sed -E 's/.*"([^"]+)".*/\1/')"
if [ "$PREV_VERSION" = "$VERSION" ]; then
    die "$CARGO_TOML 中的版本已经是 ${VERSION}，无需发布"
fi

info "检查 tag 是否已存在"
git rev-parse -q --verify "refs/tags/${TAG}" >/dev/null && die "本地已存在 tag ${TAG}"
if git ls-remote --exit-code --tags origin "refs/tags/${TAG}" >/dev/null 2>&1; then
    die "远端 origin 已存在 tag ${TAG}"
fi

if [ "$REFRESH_LOCK" -eq 1 ] && ! command -v cargo >/dev/null 2>&1; then
    die "未找到 cargo，无法刷新 ${CARGO_LOCK}（CI 使用 --locked，lock 不同步会导致构建失败）。请先安装 Rust，或用 --skip-lock 自行确认 lock 已同步。"
fi

info "发布计划：${PREV_VERSION} -> ${VERSION}（tag: ${TAG}，分支: ${CURRENT_BRANCH}）"
if [ "$DRY_RUN" -eq 1 ]; then warn "DRY-RUN 模式：不会提交、不会打 tag、不会推送"; fi
if [ "$PUSH" -eq 0 ]; then warn "--no-push：仅本地提交与打 tag"; fi

# ---------- 更新版本号 ----------
info "更新 ${CARGO_TOML} 中的版本号"

# 保留原文件的换行风格（Windows 下检出的文件可能是 CRLF）
if head -n 1 "$CARGO_TOML" | grep -q $'\r'; then
    EOL=$'\r\n'
else
    EOL=$'\n'
fi

if ! awk -v ver="$VERSION" -v ors="$EOL" '
    { sub(/\r$/, "") }
    /^\[package\]/            { in_pkg = 1 }
    in_pkg && !done && /^version[[:space:]]*=/ {
        sub(/"[^"]*"/, "\"" ver "\"")
        done = 1
    }
    { printf "%s%s", $0, ors }
    END { if (!done) exit 1 }
' "$CARGO_TOML" > "${CARGO_TOML}.tmp"; then
    rm -f "${CARGO_TOML}.tmp"
    die "未能在 [package] 段找到 version 字段"
fi
mv "${CARGO_TOML}.tmp" "$CARGO_TOML"

WRITTEN_VERSION="$(grep -m1 -E '^version[[:space:]]*=' "$CARGO_TOML" | sed -E 's/.*"([^"]+)".*/\1/')"
if [ "$WRITTEN_VERSION" != "$VERSION" ]; then
    rollback
    die "写入版本号失败（当前为 '${WRITTEN_VERSION}'）"
fi

# ---------- 刷新 Cargo.lock ----------
if [ "$REFRESH_LOCK" -eq 1 ]; then
    info "刷新 ${CARGO_LOCK}（cargo metadata）"
    if ! cargo metadata --format-version 1 >/dev/null; then
        rollback
        die "刷新 ${CARGO_LOCK} 失败，已还原改动。请先修复依赖解析问题，或加 --skip-lock 跳过。"
    fi
fi

# ---------- 展示 diff / 确认 ----------
info "改动如下："
git --no-pager diff -- "$CARGO_TOML" "$CARGO_LOCK" || true

if [ "$DRY_RUN" -eq 1 ]; then
    rollback
    info "DRY-RUN 结束，已还原改动。"
    exit 0
fi

if [ "$ASSUME_YES" -eq 0 ]; then
    printf '\n确认发布 %s 并触发 CI？[y/N] ' "$TAG"
    read -r REPLY
    case "$REPLY" in
        y|Y|yes|YES) ;;
        *) rollback; info "已取消，改动已还原。"; exit 1 ;;
    esac
fi

# ---------- 提交 ----------
info "提交改动"
git add "$CARGO_TOML"
[ -f "$CARGO_LOCK" ] && git add "$CARGO_LOCK"
git commit -m "chore(release): ${TAG}"

# ---------- 打 tag ----------
info "创建附注 tag ${TAG}"
git tag -a "$TAG" -m "Release ${TAG}"

# ---------- 推送 ----------
if [ "$PUSH" -eq 1 ]; then
    info "推送分支 ${CURRENT_BRANCH} 到 origin"
    git push origin "$CURRENT_BRANCH"
    info "推送 tag ${TAG}（将触发 Release 工作流）"
    git push origin "$TAG"
else
    warn "已跳过推送。手动推送命令：git push origin ${CURRENT_BRANCH} && git push origin ${TAG}"
fi

# ---------- 输出仓库链接 ----------
REMOTE_URL="$(git remote get-url origin 2>/dev/null || true)"
case "$REMOTE_URL" in
    git@github.com:*)   WEB="https://github.com/${REMOTE_URL#git@github.com:}" ;;
    https://github.com/*) WEB="$REMOTE_URL" ;;
    *)                  WEB="" ;;
esac
WEB="${WEB%.git}"

echo
info "发布流程已启动：${TAG}"
if [ -n "$WEB" ]; then
    echo "  Actions : ${WEB}/actions"
    echo "  Release : ${WEB}/releases/tag/${TAG}"
fi
echo "  （构建约需数分钟，完成后产物会自动挂到该 Release 下）"
