#!/bin/bash
#
# Bash script to verify if Validator stash is being backed by 1KV nominators or not
# 
# > Prerequisisites
# apt install jq
#
# > Make a file executable
# chmod +x ./check_1kv_nominators.sh
# 
# > Positional arguments:
# 1st - Chain (polkadot/kusama)
# 2nd - Validator is active? (true/false)
# 3rd - Nominators
# 4th - All nominators
#
# > run with the following parameters e.g.:
# ./check_1kv_nominators.sh kusama true stash_1,stash_2
# 

FILENAME="$(basename $0)"

printf "> $FILENAME $1 $2 $3 $4 \n"

if [ -z "$1" ]
then
  printf "! ⚠️ Positional argument 1 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-all-nominators' are set \n"
  exit 1;
else
  CHAIN=$1
fi

# NOTE: change endpoint and cached file according to the chain
NOMINATORS_1KV_ENDPOINT="https://${CHAIN,,}.w3f.community/nominators"
NOMINATORS_1KV_RAW_FILENAME="$(dirname $0)/1kv_${CHAIN,,}_nominators.json"

if [ -z "$2" ]
then
  printf "! ⚠️ Positional argument 2 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-all-nominators' are set \n"
  exit 1;
else
  IS_ACTIVE=$1
fi

if [ -z "$3" ]
then
  printf "! ⚠️ Positional argument 3 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-all-nominators' are set \n"
  exit 1;
else
  NOMINATORS=$3
fi

if [ -z "$4" ]
then
  printf "! ⚠️ Positional argument 4 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-all-nominators' are set \n"
  exit 1;
else
  ALL_NOMINATORS=$4
fi

# Write 1kv nominators endpoint to a file
curl -sS --fail -X GET -G -H 'Accept: application/json' $NOMINATORS_1KV_ENDPOINT -o $NOMINATORS_1KV_RAW_FILENAME 
if [ $? -ne 0 ]; 
then
  printf "! ⚠️ 1KV endpoint is down ($NOMINATORS_1KV_ENDPOINT) \n"
fi

IS_1KV_NOMINATOR_BACKING="false"
# Read file and check if one of the 1kv nominators is currently in the list of the validator nominators
for row in $( cat $NOMINATORS_1KV_RAW_FILENAME | jq -r '.[] | @base64' ); do
    _jq() {
     echo ${row} | base64 --decode | jq -r ${1}
    }
    NOMINATOR=$(_jq '.stash')
    NOMINATOR_LEN=${#NOMINATOR}
    if [[ "$NOMINATORS" == *"$NOMINATOR"* ]]; then
        printf "! ↳ 🟢 ${NOMINATOR:0:6}...${NOMINATOR:NOMINATOR_LEN-6:NOMINATOR_LEN} ⚡ 1KV \n"
        IS_1KV_NOMINATOR_BACKING="true"
    elif [[ "$ALL_NOMINATORS" == *"$NOMINATOR"* ]]; then
        printf "! ↳ 🔴 ${NOMINATOR:0:6}...${NOMINATOR:NOMINATOR_LEN-6:NOMINATOR_LEN} ⚡ 1KV \n"
        IS_1KV_NOMINATOR_BACKING="true"
    fi
done

# 1KV nominators not found and validator active
if [ "$IS_ACTIVE" == "true" ] && [ "$IS_1KV_NOMINATOR_BACKING" == "false" ]; 
then
  printf "! ↳ Only independent nominators 🚀 ✨ \n"
fi