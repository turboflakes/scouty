#!/bin/bash
#
# > make a file executable
# chmod +x ./_new_era.sh
#
# > positional arguments:
# 1st - Stash
# 2nd - Identity
# 3rd - Queued session keys (0x..)
# 4th - Is active? (true/false)
# 5th - Session keys queued? (true/false)
# 6th - Era
# 7th - Session
# 8th - Eras session index [1,2,3,4,5,6]
#
# The following arguments depend on exposed flags
# 9th - Nominator stashes [stash_1, stash_2, ..]
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
echo "! 1st - Stash -> $1" 
echo "! 2nd - Identity -> $2"
echo "! 3rd - Queued session keys -> ${3:0:16}.."
echo "! 4th - Is active? -> $4"
echo "! 5th - Session keys queued? -> $5"
echo "! 6th - Era -> $6"
echo "! 7th - Session -> $7"
echo "! 8th - Eras session index -> $8"
echo "! -------------------------------"
#
FILENAME="$(dirname $0)/1kv/nominators.sh"
# 
$FILENAME $4 $9
#
# ***** END *****

