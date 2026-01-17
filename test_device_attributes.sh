#!/bin/bash

# Test script for Device Attributes (DA) functionality in AgTerm
# This script sends various terminal query sequences to test if the terminal responds correctly

echo "Testing Device Attributes (DA) functionality..."
echo ""

# Test 1: Primary DA (DA1)
echo "Test 1: Primary DA (CSI c)"
echo -n "Sending CSI c... "
printf "\033[c"
sleep 0.1
echo "Done"
echo ""

# Test 2: Secondary DA (DA2)
echo "Test 2: Secondary DA (CSI > c)"
echo -n "Sending CSI > c... "
printf "\033[>c"
sleep 0.1
echo "Done"
echo ""

# Test 3: Device Status Report (DSR)
echo "Test 3: Device Status Report (CSI 5 n)"
echo -n "Sending CSI 5 n... "
printf "\033[5n"
sleep 0.1
echo "Done"
echo ""

# Test 4: Cursor Position Report (CPR)
echo "Test 4: Cursor Position Report (CSI 6 n)"
echo -n "Sending CSI 6 n... "
printf "\033[6n"
sleep 0.1
echo "Done"
echo ""

echo "All tests sent. The terminal should have responded to these queries."
echo "If responses are being sent correctly, apps like vim, htop, etc. will work better."
