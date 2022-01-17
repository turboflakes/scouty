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
# 9th - Network name (--expose-network flag must be set)
# 10th - Network token symbol (--expose-network flag must be set)
# 11th - Network token decimals (--expose-network flag must be set)
#
# 12th - Validator Total stake (--expose-nominators flag must be set)
# 13th - Validator Own stake (--expose-nominators flag must be set)
# 14th - Nominator stashes [stash_1, stash_2, ..] (--expose-nominators flag must be set)
# 15th - Nominator stakes [stake_1, stake_2, ..] (--expose-nominators flag must be set)
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
echo "! (9th) - Network name -> ${9}"
echo "! (10th) - Network token symbol -> ${10}"
echo "! (11th) - Network token decimals -> ${11}"
echo "! (12th) - Validator total stake -> ${12}"
echo "! (13th) - Validator own stake -> ${13}"
echo "! (14th) - Nominators -> ${14}"
echo "! (15th) - Nominators Stake -> ${15}"
echo "! -------------------------------"
#
# ***** END *****

