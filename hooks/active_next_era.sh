#!/bin/bash
#
# > make a file executable
# chmod +x ./active_next_era.sh
#
# > positional arguments:
# 1st - Stash
# 2nd - Identity
# 3rd - Next Era
# 4th - Next Session 
#
# > Special character '!' controls message visibility on Matrix (Element)
# Any message that starts with '!' will be sent to Matrix, to the user private room
# 
# echo "! This message will be sent to Matrix"
# echo "This message will NOT be sent to Matrix"
# 
# ***** START *****
#
echo "! e.g. write your own script here"
echo "! --------------------------------"
echo "! Positional arguments:"
echo "! 1st - Stash -> $1" 
echo "! 2nd - Identity -> $2"
echo "! 3rd - Next Era -> $3"
echo "! 4th - Next Session -> $4"
echo "! -------------------------------"
echo "! e.g. 🏃 Warm up! $2 will be 🟢 next era $3"
#
# ***** END *****