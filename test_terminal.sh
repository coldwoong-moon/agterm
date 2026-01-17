#!/bin/bash
# Test script for terminal emulator ANSI escape code handling

echo "=== Terminal Emulator Test Suite ==="
echo ""

# Test 1: Basic colors
echo "Test 1: Basic ANSI colors"
echo -e "\x1b[31mRed text\x1b[0m"
echo -e "\x1b[32mGreen text\x1b[0m"
echo -e "\x1b[33mYellow text\x1b[0m"
echo -e "\x1b[34mBlue text\x1b[0m"
echo -e "\x1b[35mMagenta text\x1b[0m"
echo -e "\x1b[36mCyan text\x1b[0m"
echo ""

# Test 2: Bright colors
echo "Test 2: Bright colors"
echo -e "\x1b[91mBright red\x1b[0m"
echo -e "\x1b[92mBright green\x1b[0m"
echo -e "\x1b[93mBright yellow\x1b[0m"
echo -e "\x1b[94mBright blue\x1b[0m"
echo ""

# Test 3: Bold text
echo "Test 3: Bold text"
echo -e "\x1b[1mBold text\x1b[0m"
echo -e "\x1b[1;32mBold green\x1b[0m"
echo ""

# Test 4: Background colors
echo "Test 4: Background colors"
echo -e "\x1b[41mRed background\x1b[0m"
echo -e "\x1b[42mGreen background\x1b[0m"
echo -e "\x1b[43mYellow background\x1b[0m"
echo ""

# Test 5: Combined styles
echo "Test 5: Combined styles"
echo -e "\x1b[1;4;32mBold underlined green\x1b[0m"
echo -e "\x1b[7mReverse video\x1b[0m"
echo ""

# Test 6: Cursor movement (clear line)
echo "Test 6: Cursor movement"
echo -n "This line will be cleared..."
sleep 1
echo -e "\r\x1b[KLine cleared!"
echo ""

# Test 7: ls with colors (if supported)
echo "Test 7: ls --color output"
ls --color=auto
echo ""

# Test 8: Korean/CJK text with colors
echo "Test 8: Korean/CJK text"
echo -e "\x1b[32m한글 테스트\x1b[0m"
echo -e "\x1b[33m中文測試\x1b[0m"
echo -e "\x1b[34m日本語テスト\x1b[0m"
echo ""

echo "=== Tests Complete ==="
