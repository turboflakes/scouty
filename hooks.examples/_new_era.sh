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
# 9th - To Be Defined (TBD)
#
# The following arguments depend on exposed flags
# 10th - Network name (--expose-network flag must be set)
# 11th - Network token symbol (--expose-network flag must be set)
# 12th - Network token decimals (--expose-network flag must be set)

# 13th - Validator Total stake (--expose-nominators flag must be set)
# 14th - Validator Own stake (--expose-nominators flag must be set)
# 15th - Nominator stashes [stash_1, stash_2, ..] (--expose-nominators flag must be set)
# 16th - Nominator stakes [stake_1, stake_2, ..] (--expose-nominators flag must be set)
#
# 17th - Number of Authored blocks (--expose-authored-blocks flag must be set)
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
# echo "! 4th - Is active? -> $4"
# echo "! 5th - Session keys queued? -> $5"
# echo "! 6th - Era -> $6"
# echo "! 7th - Session -> $7"
# echo "! 8th - Eras session index -> $8"
# echo "! 9th - TBD -> $9"
# echo "! (10th) - Network name -> ${10}"
# echo "! (11th) - Network token symbol -> ${11}"
# echo "! (12th) - Network token decimals -> ${12}"
# echo "! (13th) - Validator total stake -> ${13}"
# echo "! (14th) - Validator own stake -> ${14}"
# echo "! (15th) - Nominators -> ${15}"
# echo "! (16th) - Nominators Stake -> ${16}"
# echo "! (17th) - Number of Authored blocks -> ${17}"
# echo "! -------------------------------"
#
# NOTE: this example requires the following flags to be present when runing scouty cli
# /opt/scouty-cli/scouty --config-path /opt/scouty-cli/.env --expose-network --expose-nominators --expose-authored-blocks
#
if [ "$4" = "true" ]
then
  # Nominators and Stake
  # Convert nominators string "stash_1,stash_2" to an array ("stash_1" "stash_2")
  NOMINATORS=(${15//,/ })
  echo "! 🦸 Total Nominators ${#NOMINATORS[@]}" 
  TOTAL_STAKE=$((${13}/(10**${12})))
  echo "! 💸 Total Stake $TOTAL_STAKE ${11}"
  OWN_STAKE=$((${14}/(10**${12})))
  echo "! 💰 Own Stake $OWN_STAKE ${11}"
  #
  # 1kv nominators check
  FILENAME="$(dirname $0)/1kv/check_1kv_nominators.sh"
  $FILENAME $4 ${15}
  #
  # Authored Blocks
  echo "! 🍫 Authored blocks ${17}"
fi
#
# ***** END *****

