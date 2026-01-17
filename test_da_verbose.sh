#!/bin/bash

# Verbose test script for Device Attributes functionality
# Shows what queries are sent and captures responses

echo "=== Device Attributes Test Suite ==="
echo "This tests whether AgTerm responds to terminal queries"
echo ""

# Function to send query and show result
test_query() {
    local name="$1"
    local query="$2"
    local expected="$3"

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Test: $name"
    echo "Query: $query"
    echo "Expected response: $expected"
    echo ""
    echo -n "Sending query... "
    printf "$query"
    sleep 0.1
    echo "✓"
    echo ""
}

# Test 1: Primary Device Attributes
test_query \
    "Primary DA (DA1)" \
    "\033[c" \
    "CSI ? 1 ; 2 c (VT100 with Advanced Video)"

# Test 2: Secondary Device Attributes
test_query \
    "Secondary DA (DA2)" \
    "\033[>c" \
    "CSI > 0 ; 0 ; 0 c (VT100 compatible)"

# Test 3: Device Status Report
test_query \
    "Device Status Report (DSR)" \
    "\033[5n" \
    "CSI 0 n (Terminal OK)"

# Test 4: Cursor Position Report (at current position)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test: Cursor Position Report (CPR)"
echo "Query: CSI 6 n"
echo "Expected response: CSI <row> ; <col> R"
echo ""
echo "Current position:"
echo -n "Sending CPR query... "
printf "\033[6n"
sleep 0.1
echo "✓"
echo ""

# Test 5: CPR at specific position
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Test: CPR at position (10,20)"
echo "Moving cursor to row 10, col 20..."
printf "\033[10;20H"
sleep 0.05
echo "Sending CPR query... "
printf "\033[6n"
sleep 0.1
echo "✓"
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ All tests completed!"
echo ""
echo "Note: Responses are sent back to the shell, but may not be visible."
echo "If apps like vim, htop, or neofetch work properly, DA is functioning."
echo ""
echo "To verify responses are being sent, you can:"
echo "1. Run: cat | xxd"
echo "2. Press Ctrl+V then Esc then [ then c"
echo "3. You should see the hex response from the terminal"
