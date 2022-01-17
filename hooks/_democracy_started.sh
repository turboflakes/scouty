#!/bin/bash
#
# > make a file executable
# chmod +x ./_democracy_started.sh
#
# > positional arguments:
# 1st - Referendum index
# 2nd - Vote Threshold
#
# The following arguments depend on exposed flags
# 3r - Network name (--expose-network flag must be set)
# 4th - Network token symbol (--expose-network flag must be set)
# 5th - Network token decimals (--expose-network flag must be set)
#
# > Special character '!' controls message visibility on Matrix (Element)
# Any message that starts with '!' will be sent to Matrix, to the user private room
# 
# echo "! This message will be sent to Matrix"
# echo "This message will NOT be sent to Matrix"
# 
# ***** START *****
#
echo "! e.g. Write your own script here"
echo "! --------------------------------"
echo "! Positional arguments:"
echo "! 1st - Referendum index -> $1" 
echo "! 2nd - Vote Threshold -> $2"
echo "! -------------------------------"
#
# ***** END *****

