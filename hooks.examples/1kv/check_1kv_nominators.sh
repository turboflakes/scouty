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
# 1st - Validator is active? (true/false)
# 2nd - Validator nominators
#
# > run with the following parameters e.g.:
# ./check_1kv_nominators.sh true stash_1,stash_2
# 

FILENAME="$(basename $0)"
NOMINATORS_1KV_ENDPOINT="https://kusama.w3f.community/nominators"
NOMINATORS_1KV_RAW_FILENAME="$(dirname $0)/1kv_nominators.json"

printf "> $FILENAME $1 $2 $3 \n"

if [ -z "$1" ]
then
  printf "! ⚠️ Positional argument 1 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-total-nominators' are set \n"
  exit 1;
else
  IS_ACTIVE=$1
fi

if [ -z "$2" ]
then
  printf "! ⚠️ Positional argument 2 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-total-nominators' are set \n"
  exit 1;
else
  NOMINATORS=$2
fi

if [ -z "$3" ]
then
  printf "! ⚠️ Positional argument 3 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators --expose-total-nominators' are set \n"
  exit 1;
else
  TOTAL_NOMINATORS=$3
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
        printf "! ↳ 🟢 ${NOMINATOR:0:6}...${NOMINATOR:NOMINATOR_LEN-6:NOMINATOR_LEN} \n"
        IS_1KV_NOMINATOR_BACKING="true"
    elif [[ "$TOTAL_NOMINATORS" == *"$NOMINATOR"* ]]; then
        printf "! ↳ 🔴 ${NOMINATOR:0:6}...${NOMINATOR:NOMINATOR_LEN-6:NOMINATOR_LEN} \n"
        IS_1KV_NOMINATOR_BACKING="true"
    fi
done

# 1KV nominators not found and validator active
if [ "$IS_ACTIVE" == "true" ] && [ "$IS_1KV_NOMINATOR_BACKING" == "false" ]; 
then
  printf "! ↳ Independent nominators 🚀 ✨ \n"
fi