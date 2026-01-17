#!/bin/bash
# Test script for alternate screen buffer support

echo "=== Alternate Screen Buffer Test ==="
echo ""
echo "This test demonstrates the alternate screen buffer functionality."
echo "The alternate screen is used by programs like vim, less, htop, etc."
echo ""
echo "Current screen content (normal buffer)"
echo "Line 1"
echo "Line 2"
echo "Line 3"
echo ""
echo "Press Enter to switch to alternate screen..."
read

# Switch to alternate screen (CSI ?1049h)
echo -ne "\x1b[?1049h"

# Clear screen and write some content
echo -ne "\x1b[2J\x1b[H"
echo "=== ALTERNATE SCREEN ACTIVE ==="
echo ""
echo "You are now in the alternate screen buffer."
echo "This is like what vim or less would show."
echo ""
echo "Features:"
echo "  - Separate buffer from main screen"
echo "  - Cursor position saved before switching"
echo "  - Scrollback is separate"
echo ""
echo "Press Enter to return to normal screen..."
read

# Switch back to normal screen (CSI ?1049l)
echo -ne "\x1b[?1049l"

echo ""
echo "Back to normal screen!"
echo "The previous content should still be visible above."
echo ""
echo "=== Test Complete ==="
