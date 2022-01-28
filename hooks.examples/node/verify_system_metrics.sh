#!/bin/bash
#
# Bash script to fetch system metrics from a polkadot node using sysstat
#
# > Make a file executable
# chmod +x ./verify_system_metrics.sh
# 
# > Positional arguments:
# 1st - Username
# 2nd - Password
#
# > run with the following parameters e.g.:
# ./verify_system_metrics.sh username ip_address
# 

FILENAME="$(basename $0)"
REMOTE_FILENAME="$(dirname $0)/remote/system_metrics.sh"

printf "> /$FILENAME $1 $2 \n"

# Verify $1
if [ -z "$1" ]
then
  exit
else
  USER=$1
fi

if [ -z "$2" ]
then
  exit
else
  IP_ADDRESS=$2
fi

# Run script on remote machine and print
SYSTEM_METRICS=$( ssh  -o StrictHostKeyChecking=no $USER@$IP_ADDRESS 'bash -s' < $REMOTE_FILENAME )
printf "$SYSTEM_METRICS \n"