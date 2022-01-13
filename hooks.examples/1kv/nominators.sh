#!/bin/bash
#
# Bash script to verify if Validator stash is being backed by 1KV nominators or not
# 
# > Prerequisisites
# apt install jq
#
# > Make a file executable
# chmod +x ./nominators.sh
# 
# > Positional arguments:
# 1st - Validator is active? (true/false)
# 2nd - Validator nominators
#
# > run with the following parameters e.g.:
# ./nominators.sh true stash_1,stash_2
# 

FILENAME="$(basename $0)"

printf "> /$FILENAME $1 $2 \n"

for row in $( curl 'https://kusama.w3f.community/nominators' | jq -r '.[] | @base64' ); do
    _jq() {
     echo ${row} | base64 --decode | jq -r ${1}
    }
    NOMINATOR=$(_jq '.stash')
    if [[ "$2" == *"$NOMINATOR"* ]]; then
        printf "! 1KV -> $NOMINATOR"
        exit 0
    fi
done

# 1KV nominators not found and validator active
if [ "$1" == "true" ]; 
then
  printf "! 🥳 Running Independent 🚀 \n"
fi