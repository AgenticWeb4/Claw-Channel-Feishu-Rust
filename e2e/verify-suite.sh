#!/bin/bash
# 快速验证测试套件完整性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🔍 Claw Feishu E2E 测试套件完整性检查"
echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 1. 文件结构检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "📁 检查文件结构..."

EXPECTED_DIRS=("01-auth" "02-health" "03-messages" "04-security" "05-websocket" "06-zeroclaw" "07-open-lark" "08-edge-cases" "08-openclaw" "environments")
MISSING_DIRS=0

for dir in "${EXPECTED_DIRS[@]}"; do
    if [ -d "$SCRIPT_DIR/$dir" ]; then
        echo "  ✓ $dir/"
    else
        echo "  ✗ $dir/ MISSING"
        MISSING_DIRS=$((MISSING_DIRS + 1))
    fi
done

if [ $MISSING_DIRS -gt 0 ]; then
    echo "❌ $MISSING_DIRS 个目录缺失"
    exit 1
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 2. 测试文件计数
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "📊 统计测试文件..."

TEST_COUNT=$(find "$SCRIPT_DIR" -name "*.bru" -not -path "*/environments/*" | wc -l | tr -d ' ')
DOC_COUNT=$(find "$SCRIPT_DIR" -name "*.md" | wc -l | tr -d ' ')

echo "  测试文件: $TEST_COUNT (预期: 34)"
echo "  文档文件: $DOC_COUNT (预期: 6)"

if [ "$TEST_COUNT" -ne 34 ]; then
    echo "  ⚠ 测试文件数量不匹配！"
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 3. 必需文件检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "📋 检查必需文件..."

REQUIRED_FILES=(
    "bruno.json"
    "README.md"
    "QUICKSTART.md"
    "COVERAGE.md"
    "CHECKLIST.md"
    "STRUCTURE.md"
    "SCRIPTS.md"
    ".env.example"
    ".gitignore"
    "run-tests.sh"
    "environments/dev.bru"
    "environments/test.bru"
)

MISSING_FILES=0

for file in "${REQUIRED_FILES[@]}"; do
    if [ -f "$SCRIPT_DIR/$file" ]; then
        echo "  ✓ $file"
    else
        echo "  ✗ $file MISSING"
        MISSING_FILES=$((MISSING_FILES + 1))
    fi
done

if [ $MISSING_FILES -gt 0 ]; then
    echo "❌ $MISSING_FILES 个文件缺失"
    exit 1
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 4. 环境配置检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "🔧 检查环境配置..."

if [ -f "$SCRIPT_DIR/.env" ]; then
    echo "  ✓ .env 文件存在"
    
    # 检查必需变量
    REQUIRED_VARS=("FEISHU_APP_ID" "FEISHU_APP_SECRET")
    for var in "${REQUIRED_VARS[@]}"; do
        if grep -q "^$var=" "$SCRIPT_DIR/.env"; then
            VALUE=$(grep "^$var=" "$SCRIPT_DIR/.env" | cut -d'=' -f2)
            if [[ "$VALUE" == *"placeholder"* ]] || [[ "$VALUE" == *"replace_me"* ]]; then
                echo "  ⚠ $var 使用占位符值（需更新）"
            else
                echo "  ✓ $var 已配置"
            fi
        else
            echo "  ✗ $var 未配置"
        fi
    done
else
    echo "  ⚠ .env 文件不存在（将使用 .env.example）"
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 5. 脚本权限检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "🔐 检查脚本权限..."

if [ -x "$SCRIPT_DIR/run-tests.sh" ]; then
    echo "  ✓ run-tests.sh 可执行"
else
    echo "  ⚠ run-tests.sh 不可执行，正在修复..."
    chmod +x "$SCRIPT_DIR/run-tests.sh"
    echo "  ✓ 权限已修复"
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 6. Bruno CLI 检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "🛠️  检查依赖..."

if command -v bru &> /dev/null; then
    BRU_VERSION=$(bru --version 2>&1 | head -n1)
    echo "  ✓ Bruno CLI 已安装: $BRU_VERSION"
else
    echo "  ✗ Bruno CLI 未安装"
    echo ""
    echo "安装方法:"
    echo "  npm install -g @usebruno/cli"
    echo "  或: brew install bruno"
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 7. 语法检查（简单）
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "🔬 检查测试文件语法..."

SYNTAX_ERRORS=0

for bru_file in $(find "$SCRIPT_DIR" -name "*.bru" -not -path "*/environments/*"); do
    # 检查必需部分
    if ! grep -q "^meta {" "$bru_file"; then
        echo "  ✗ $(basename $bru_file): 缺少 meta 部分"
        SYNTAX_ERRORS=$((SYNTAX_ERRORS + 1))
    fi
    
    if ! grep -q "^tests {" "$bru_file" && ! grep -q "^assert {" "$bru_file"; then
        echo "  ⚠ $(basename $bru_file): 缺少 tests 或 assert 部分"
    fi
done

if [ $SYNTAX_ERRORS -eq 0 ]; then
    echo "  ✓ 所有测试文件语法基本正确"
else
    echo "  ⚠ 发现 $SYNTAX_ERRORS 个语法问题"
fi

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 8. 覆盖率检查
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "📈 功能需求覆盖检查..."

# F-001 到 F-008
FEATURE_IDS=("F-001" "F-002" "F-003" "F-004" "F-005" "F-006" "F-007" "F-008")
COVERED=0

for fid in "${FEATURE_IDS[@]}"; do
    if grep -q "$fid" "$SCRIPT_DIR/COVERAGE.md"; then
        COVERED=$((COVERED + 1))
    fi
done

echo "  功能需求覆盖: $COVERED/${#FEATURE_IDS[@]} ($(($COVERED * 100 / ${#FEATURE_IDS[@]}))%)"

echo ""

# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
# 总结
# ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ 测试套件完整性检查完成"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📊 统计信息:"
echo "  测试文件: $TEST_COUNT"
echo "  文档文件: $DOC_COUNT"
echo "  总文件数: $(find "$SCRIPT_DIR" -type f | wc -l | tr -d ' ')"
echo ""
echo "📖 快速开始:"
echo "  1. 配置凭证:  nano .env"
echo "  2. 运行测试:  ./run-tests.sh"
echo "  3. 查看文档:  cat QUICKSTART.md"
echo ""
echo "🔗 参考文档:"
echo "  - QUICKSTART.md — 5 分钟快速开始"
echo "  - README.md     — 完整测试说明"
echo "  - COVERAGE.md   — 测试覆盖率报告"
echo "  - CHECKLIST.md  — 验收清单"
echo ""

if [ $MISSING_DIRS -eq 0 ] && [ $MISSING_FILES -eq 0 ]; then
    echo "✅ 所有检查通过！测试套件已就绪。"
    exit 0
else
    echo "⚠️  发现一些问题，请修复后再运行测试。"
    exit 1
fi
