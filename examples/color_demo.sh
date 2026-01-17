#!/bin/bash
# AgTerm 256-color and TrueColor Demo Script

echo "=== AgTerm Color Support Demo ==="
echo

# Test 16 basic colors
echo "1. Basic 16 Colors (SGR 30-37, 40-47, 90-97, 100-107)"
echo "Foreground colors:"
for i in {30..37}; do
    echo -ne "\e[${i}m Color $i \e[0m"
done
echo
for i in {90..97}; do
    echo -ne "\e[${i}m Color $i \e[0m"
done
echo
echo

# Test 256-color palette
echo "2. 256-Color Palette (SGR 38;5;N and 48;5;N)"
echo "Standard colors (0-15):"
for i in {0..15}; do
    echo -ne "\e[38;5;${i}m█\e[0m"
done
echo
echo

echo "6x6x6 Color Cube (16-231) - Sample:"
for r in {0..5}; do
    for g in {0..5}; do
        for b in {0..5}; do
            i=$((16 + r*36 + g*6 + b))
            echo -ne "\e[38;5;${i}m█\e[0m"
        done
    done
    echo
done
echo

echo "Grayscale (232-255):"
for i in {232..255}; do
    echo -ne "\e[38;5;${i}m█\e[0m"
done
echo
echo

# Test TrueColor (24-bit RGB)
echo "3. TrueColor (24-bit RGB) - SGR 38;2;R;G;B and 48;2;R;G;B"
echo "RGB Gradient:"
for r in {0..255..8}; do
    echo -ne "\e[38;2;${r};0;0m█\e[0m"
done
echo " Red gradient"

for g in {0..255..8}; do
    echo -ne "\e[38;2;0;${g};0m█\e[0m"
done
echo " Green gradient"

for b in {0..255..8}; do
    echo -ne "\e[38;2;0;0;${b}m█\e[0m"
done
echo " Blue gradient"
echo

# Test combining colors with text attributes
echo "4. Colors + Text Attributes"
echo -e "\e[1;38;2;255;100;50mBold Orange (TrueColor)\e[0m"
echo -e "\e[3;38;5;220mItalic Yellow (256-color)\e[0m"
echo -e "\e[4;31mUnderline Red (16-color)\e[0m"
echo -e "\e[1;3;4;38;2;100;200;255mBold Italic Underline Cyan\e[0m"
echo

# Test background colors
echo "5. Background Colors"
echo -e "\e[48;5;196m  Red Background (256)  \e[0m"
echo -e "\e[48;2;100;200;150m  Teal Background (RGB)  \e[0m"
echo -e "\e[38;2;255;255;255;48;2;50;50;50m  White on Dark Gray  \e[0m"
echo

echo "=== Demo Complete ==="
