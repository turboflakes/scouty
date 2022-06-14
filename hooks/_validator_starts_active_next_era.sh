#!/bin/bash
#
# > make a file executable
# chmod +x ./_validator_starts_active_next_era.sh
#
# > positional arguments:
# 1st - Stash
# 2nd - Identity
# 3rd - Queued session keys (0x..)
# 4th - Next Era
# 5th - Next Session 
#
# The following arguments depend on exposed flags
# 6th - Network name (--expose-network flag must be set)
# 7th - Network token symbol (--expose-network flag must be set)
# 8th - Network token decimals (--expose-network flag must be set)
#
# > Special character '!' controls message visibility on Matrix (Element)
# Any message that starts with '!' will be sent to Matrix, to the user private room
# 
# echo "! This message will be sent to Matrix"
# echo "This message will NOT be sent to Matrix"
# 
# ***** START *****
#
# echo "! e.g. Write your own script here"
# echo "! --------------------------------"
# echo "! Positional arguments:"
# echo "! 1st - Stash -> $1" 
# echo "! 2nd - Identity -> $2"
# echo "! 3rd - Queued session keys -> ${3:0:16}.."
# echo "! 4th - Next Era -> $4"
# echo "! 5th - Next Session -> $5"
# echo "! (6th) - Network name -> $6"
# echo "! (7th) - Network token symbol -> $7"
# echo "! (8th) - Network token decimals -> $8"
# echo "! -------------------------------"
echo "! 🏃 Warm up! $2 will be 🟢 next era $4"
#
# ***** END *****