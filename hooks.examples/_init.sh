#!/bin/bash
#
# > make a file executable
# chmod +x ./_init.sh
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
# 9th - Current block
#
# The following arguments depend on exposed flags
# 10th - Network name (--expose-network flag must be set)
# 11th - Network token symbol (--expose-network flag must be set)
# 12th - Network token decimals (--expose-network flag must be set)

# 13th - Validator Total stake (--expose-nominators flag must be set)
# 14th - Validator Own stake (--expose-nominators flag must be set)
# 15th - Active Nominators stashes [stash_1, stash_2, ..] (--expose-nominators flag must be set)
# 16th - Active Nominators stakes [stake_1, stake_2, ..] (--expose-nominators flag must be set)
#
# 17th - Number of Authored blocks (Session - 1) (--expose-authored-blocks flag must be set)
# 18th - Not applicable
#
# 19th - Total Nominators stashes [stash_1, stash_2, ..] (--expose-total-nominators flag must be set)
# 20th - Not applicable
#
# 21th - Is Para validator? (true/false) (--expose-para-validator flag must be set)
# 22th - Number of Para validator times in previous 6 Sessions (--expose-para-validator flag must be set)
#
# 23th - Last era validator points (--expose-era-points flag must be set)
# 24th - Last era validators average points (--expose-era-points flag must be set)
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
# echo "! 9th - Current block -> $9"
# echo "! (10th) - Network name -> ${10}"
# echo "! (11th) - Network token symbol -> ${11}"
# echo "! (12th) - Network token decimals -> ${12}"
# echo "! (13th) - Validator total stake -> ${13}"
# echo "! (14th) - Validator own stake -> ${14}"
# echo "! (15th) - Active Nominators -> ${15}"
# echo "! (16th) - Active Nominators Stake -> ${16}"
# echo "! (17th) - Number of Authored blocks in current Session -> ${17}"
# echo "! (18th) - NA"
# echo "! (19th) - Total Nominators -> ${19}"
# echo "! (20th) - NA"
# echo "! (21th) - Is Para Validator? -> ${21}"
# echo "! (22th) - Number of Para Validator times in previous 6 Sessions -> ${22}"
# echo "! (23th) - Last era validator points -> ${23}"
# echo "! (24th) - Last era avg validators points -> ${24}"
# echo "! -------------------------------"
# if [ "$4" = "true" ]
# then
#   echo "! 🟢 -> 😎"
# else
#   echo "! 🔴 -> 😤"
# fi
if [ "$4" = "true" ]
then
  # Nominators and Stake
  # Convert nominators string "stash_1,stash_2" to an array ("stash_1" "stash_2")
  NOMINATORS=(${15//,/ })
  TOTAL_NOMINATORS=(${19//,/ })
  echo "! 🦸 Nominators ${#NOMINATORS[@]}/${#TOTAL_NOMINATORS[@]}"
  # 1kv nominators check
  FILENAME="$(dirname $0)/1kv/check_1kv_nominators.sh"
  $FILENAME $4 ${15} ${19}
  #
  TOTAL_ACTIVE_STAKE=$((${13}/(10**${12})))
  echo "! 💸 Active stake $TOTAL_ACTIVE_STAKE ${11}"
  OWN_STAKE=$((${14}/(10**${12})))
  echo "! 💰 Own stake $OWN_STAKE ${11}"
  # Para Validator
  if [ "${21}" = "true" ]
  then
    echo "! 🪂 Para validator ✔"
  fi
  # Last authored blocks
  echo "! 📦 Latest authored blocks ${17}"
  #
else 
  # Nominators and Stake
  TOTAL_NOMINATORS=(${19//,/ })
  echo "! 🦸 Nominators 0/${#TOTAL_NOMINATORS[@]}"
  #
  # 1kv nominators check
  FILENAME="$(dirname $0)/1kv/check_1kv_nominators.sh"
  $FILENAME $4 "-" ${19}
fi
#
# ***** END *****

