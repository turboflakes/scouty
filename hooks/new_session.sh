#!/bin/bash
#
# > make a file executable
# chmod +x ./new_session.sh
#
# > positional arguments:
# 1st - Validator Stash
# 2nd - Stash Active/Inactive (true/false)
# 3rd - Era
# 4th - Session
# 5th - Era session index
#
if [ "$2" = "true" ]
then
  echo "🟢 $1 -> Session $4 ($5) | Era $3"
else
  echo "🔴 $1 -> Session $4 ($5) | Era $3"
fi
