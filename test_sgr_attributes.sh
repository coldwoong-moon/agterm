#!/bin/bash

# Test script for new SGR text attributes in AgTerm
# Usage: ./test_sgr_attributes.sh

echo "Testing SGR Text Attributes in AgTerm"
echo "======================================"
echo ""

# Test dim (SGR 2)
echo -e "\x1b[2mThis text is DIM\x1b[0m"
echo -e "\x1b[2;31mDim red text\x1b[0m"
echo ""

# Test italic (SGR 3)
echo -e "\x1b[3mThis text is ITALIC\x1b[0m"
echo -e "\x1b[3;32mItalic green text\x1b[0m"
echo ""

# Test strikethrough (SGR 9)
echo -e "\x1b[9mThis text has STRIKETHROUGH\x1b[0m"
echo -e "\x1b[9;34mStrikethrough blue text\x1b[0m"
echo ""

# Test combined attributes
echo -e "\x1b[1;3;4;9mBold + Italic + Underline + Strikethrough\x1b[0m"
echo -e "\x1b[2;3mDim + Italic\x1b[0m"
echo -e "\x1b[1;9;31mBold + Strikethrough + Red\x1b[0m"
echo ""

# Test attribute resets
echo -e "\x1b[2mDim text \x1b[22mnormal intensity\x1b[0m"
echo -e "\x1b[3mItalic text \x1b[23mnot italic\x1b[0m"
echo -e "\x1b[9mStrikethrough text \x1b[29mno strikethrough\x1b[0m"
echo ""

# Test with colors
echo -e "\x1b[2;31mDim Red\x1b[0m | \x1b[2;32mDim Green\x1b[0m | \x1b[2;34mDim Blue\x1b[0m"
echo -e "\x1b[3;91mItalic Bright Red\x1b[0m | \x1b[3;92mItalic Bright Green\x1b[0m"
echo -e "\x1b[9;33mStrikethrough Yellow\x1b[0m | \x1b[9;35mStrikethrough Magenta\x1b[0m"
echo ""

echo "Test complete!"
