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

printf "> $FILENAME $1 $2 \n"

if [ -z "$1" ]
then
  printf "! ⚠️ Positional argument 1 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators' are set \n"
  exit 1;
else
  IS_ACTIVE=$1
fi

if [ -z "$2" ]
then
  printf "! ⚠️ Positional argument 2 not defined \n"
  printf "! ⚠️ Make sure flags '--expose-network --expose-nominators' are set \n"
  exit 1;
else
  NOMINATORS=$2
fi

# Write 1kv nominators endpoint to a file
curl -sS --fail -X GET -G -H 'Accept: application/json' $NOMINATORS_1KV_ENDPOINT -o $NOMINATORS_1KV_RAW_FILENAME 
if [ $? -ne 0 ]; 
then
  printf "! ⚠️ 1kv endpoint is down ($NOMINATORS_1KV_ENDPOINT) \n"
fi

# Read file and check if one of the 1kv nominators is currently in the list of the validator nominators
for row in $( cat $NOMINATORS_1KV_RAW_FILENAME | jq -r '.[] | @base64' ); do
    _jq() {
     echo ${row} | base64 --decode | jq -r ${1}
    }
    NOMINATOR=$(_jq '.stash')
    if [[ "$NOMINATORS" == *"$NOMINATOR"* ]]; then
        printf "! 🎒 <b>1KV</b> $NOMINATOR \n"
        exit 0
    fi
done

# 1KV nominators not found and validator active
if [ "$IS_ACTIVE" == "true" ]; 
then
  printf "! 🥳 Running Independent 🚀 \n"
fi