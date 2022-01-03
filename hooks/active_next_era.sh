#!/bin/bash
#
# > make a file executable
# chmod +x ./active_next_era.sh
#
# > positional arguments:
# 1st - Validator Stash
# 2nd - Era
# 3rd - Session 
# 4th - Next Era
# 5th - Next Session 
#
# > Special character '!' controls message visibility on Matrix (Element)
# Any message that starts with '!' will be sent to Matrix, to the user private room
# 
# echo "! This message will be sent to Matrix"
# echo "This message will NOT be sent to Matrix"
# 
# ***** START *****
#
echo "🔵 $1 -> ACTIVE Next Era $4"
#
# ***** END *****