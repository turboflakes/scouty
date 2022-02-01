#!/bin/bash
#
# Bash script to be executed in the remote server to collect metrics via systat
# Note: sysstat ideally should be collecting data every minute
# please confirm cron job at /etc/cron.d/sysstat
# 
# > Make a file executable
# chmod +x ./system_metrics.sh
# 

FILENAME="$(basename $0)"
SYSSTAT_LOGS="/var/log/sysstat/*"

# --- Get storage usage on /polkadot directory
STORAGE=$( du -hs /polkadot | sort -rh | head -n 1 | awk -F ' ' '{ printf $1 }' )
MSG_STORAGE="! 💾 Usage of /polkadot: ${STORAGE}B \n"
# ---

# Note: CPU, RAM and Network are based on sar metrics logged every minute for an hour
# Get CPU metric
# CPU=$( mpstat | head -n 4 | tail -1 | awk -F ' ' '{ printf $4 }' )
CPU=$( sar -u | tail -1 | awk -F ' ' '{ printf $3 }' )
# Get memory ram metric
# RAM=$( free -m | head -n 2 | tail -1 | awk -F ' ' '{ printf $3 }' )
# RAM=$(awk '{print $1/1000}' <<<"${RAM}")
RAM=$( sar -r | tail -1 | awk -F ' ' '{ printf $4 }' )
RAM=$(awk '{print $1/1000000}' <<<"${RAM}")
MSG_CPU_RAM="! 〽️ Usage of CPU ${CPU}%%%%, RAM ${RAM:0:4}GB \n"
# ---

# --- Fetch node health
# NOTE: system_health response example:
# {
#  "isSyncing": false,
#  "peers": 37,
#  "shouldHavePeers": true
# }
PEERS="$( curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_health", "params":[]}' \
  'http://localhost:9933' \
  | jq '.result.peers' )"

# ---

# --- Fetch RPC `system_syncState`
# NOTE: system_health response example:
# {
#   "currentBlock": 11132625,
#   "highestBlock": 11132625,
#   "startingBlock": 10862594
# }
CURRENT_BLOCK_NUMBER="$( curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "system_syncState", "params":[]}' \
  'http://localhost:9933' \
  | jq '.result.currentBlock' )"
# ---

# --- Fetch Finalized block number
# Get Finalized head
BLOCK_HASH="$( curl -H "Content-Type: application/json" \
  -d '{"id":1, "jsonrpc":"2.0", "method": "chain_getFinalizedHead", "params":[]}' \
  'http://localhost:9933' \
  | jq '.result' )"
BLOCK_HASH=$( echo "$BLOCK_HASH" | awk -F ' ' '{ printf $1 }' )

# Get Header
FINALIZED_BLOCK_NUMBER="$( curl -H Content-Type:application/json \
  -d '{"id":1, "jsonrpc": "2.0", "method": "chain_getHeader", "params": ['$BLOCK_HASH']}' \
  'http://localhost:9933' \
  | jq '.result.number' )"
# Note: To convert hex block number decimal
# we first need to emove "" and 0x from heximal number eg: "0xaa1047" -> aa1047
FINALIZED_BLOCK_NUMBER=${FINALIZED_BLOCK_NUMBER//\"/}
FINALIZED_BLOCK_NUMBER=${FINALIZED_BLOCK_NUMBER//0x/}
# Convert block number hex to decimal
FINALIZED_BLOCK_NUMBER=$(( 16#$FINALIZED_BLOCK_NUMBER ))
BLOCK_DRIFT=$(( $CURRENT_BLOCK_NUMBER-$FINALIZED_BLOCK_NUMBER ))
# ---

# --- Get network metrics rx/tx
NET_RX=$( sar -n DEV --iface=eth0 | tail -1 | awk -F ' ' '{ printf $5 }' )
NET_RX=$(awk '{print $1/1000}' <<<"${NET_RX}")
NET_TX=$( sar -n DEV --iface=eth0 | tail -1 | awk -F ' ' '{ printf $6 }' )
NET_TX=$(awk '{print $1/1000}' <<<"${NET_TX}")
# ---
MSG_IDLE="! 💤 $PEERS Peers ⬇ ${NET_RX:0:4}MiB/s ⬆ ${NET_TX:0:4}MiB/s \n"
MSG_BLOCKS="! 🎭 Best: #${CURRENT_BLOCK_NUMBER}, Finalized: #${FINALIZED_BLOCK_NUMBER} \n"
MSG_BLOCK_DRIFT="! 📏 Block drift: ${BLOCK_DRIFT} \n"

# Note: Delete sysstat logs after collecting metrics (every session)
sudo rm -r $SYSSTAT_LOGS

# Printf metrics
printf "$MSG_STORAGE$MSG_CPU_RAM$MSG_IDLE$MSG_BLOCKS$MSG_BLOCK_DRIFT"