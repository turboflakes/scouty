#!/bin/bash
#
# > make a file executable
# chmod +x ./_new_session.sh
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
# 9th - Event block number
#
# The following arguments depend on exposed flags
# 10th - Network name (--expose-network flag must be set)
# 11th - Network token symbol (--expose-network flag must be set)
# 12th - Network token decimals (--expose-network flag must be set)

# 13th - Projected APR (same calculation as in polkadot.js) (--expose-nominators flag must be set)
# 14th - Validator Total stake (--expose-nominators flag must be set)
# 15th - Validator Own stake (--expose-nominators flag must be set)
# 16th - Active Nominators stashes [stash_1, stash_2, ..] (--expose-nominators flag must be set)
# 17th - Active Nominators stakes [stake_1, stake_2, ..] (--expose-nominators flag must be set)
#
# 18th - Number of Authored blocks in previous Session (--expose-authored-blocks flag must be set)
# 19th - Number of Authored blocks in previous 6 Sessions (--expose-authored-blocks flag must be set)
#
# 20th - Total Nominators stashes [stash_1, stash_2, ..] (--expose-total-nominators flag must be set)
# 21th - Not applicable
#
# 22th - Is Para validator? (true/false) (--expose-para-validator flag must be set)
# 23th - Number of Para validator times in previous 6 Sessions (--expose-para-validator flag must be set)
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
# echo "! 9th - Event block number -> $9"
# echo "! (10th) - Network name -> ${10}"
# echo "! (11th) - Network token symbol -> ${11}"
# echo "! (12th) - Network token decimals -> ${12}"
# echo "! (13th) - Projected APR -> ${13}"
# echo "! (14th) - Validator total stake -> ${14}"
# echo "! (15th) - Validator own stake -> ${15}"
# echo "! (16th) - Active Nominators -> ${16}"
# echo "! (17th) - Active Nominators Stake -> ${17}"
# echo "! (18th) - Number of Authored blocks in previous Session -> ${18}"
# echo "! (19th) - Number of Authored blocks in previous 6 Sessions -> ${19}"
# echo "! (20th) - Total Nominators -> ${20}"
# echo "! (21th) - NA"
# echo "! (22th) - Is Para Validator? -> ${22}"
# echo "! (23th) - Number of Para Validator times in previous 6 Sessions -> ${23}"
# echo "! -------------------------------"
#
# NOTE: this example requires the following flags to be present when runing scouty cli
# /opt/scouty-cli/scouty --config-path /opt/scouty-cli/.env --expose-authored-blocks
#
if [ "$4" = "true" ]
then
  # Nominators and Stake
  # Convert nominators string "stash_1,stash_2" to an array ("stash_1" "stash_2")
  NOMINATORS=(${16//,/ })
  TOTAL_NOMINATORS=(${20//,/ })
  echo "! 🦸 Nominators: ${#NOMINATORS[@]}/${#TOTAL_NOMINATORS[@]}"
  # Para Validator
  if [ "${22}" = "true" ]
  then
    echo "! 🪂 Para validator 💯"
  fi
  # Latest Authored Blocks
  echo "! 📦 Latest authored blocks: ${18}/${19}"
  #
else 
  # Nominators and Stake
  TOTAL_NOMINATORS=(${20//,/ })
  echo "! 🦸 Inactive Nominators: ${#TOTAL_NOMINATORS[@]}"
  # 1kv nominators check
  FILENAME="$(dirname $0)/1kv/check_1kv_nominators.sh"
  $FILENAME $4 "-" ${20}
  #
fi
# System metrics
# echo "! ----"
# USERNAME="USERNAME"
# e.g the IP address could be stored in a file and be loaded based depending on the validator stash
# IPADDRESS_FILENAME="$(dirname $0)/node/stashes/$1"
# IPADDRESS=$( <$IPADDRESS_FILENAME )
# VERIFY_SYSTEM_METRICS="$( $(dirname $0)/node/verify_system_metrics.sh $USERNAME $IPADDRESS )"
# echo "$VERIFY_SYSTEM_METRICS"
#
# ***** END *****

