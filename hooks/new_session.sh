#!/bin/bash
#
# > make a file executable
# chmod +x ./new_session.sh
#
# > positional arguments:
# 1st - Stash
# 2nd - Identity
# 3rd - Is active?x (true/false)
# 4th - Session keys queued? (true/false)
# 5th - Era
# 6th - Session
# 7th - Era session index
#
# > Special character '!' controls message visibility on Matrix (Element)
# Any message that starts with '!' will be sent to Matrix, to the user private room
# 
# echo "! This message will be sent to Matrix"
# echo "This message will NOT be sent to Matrix"
# 
# ***** START *****
#
echo "! e.g. write your own script here"
echo "! --------------------------------"
echo "! Positional arguments:"
echo "! 1st - Stash -> $1" 
echo "! 2nd - Identity -> $2"
echo "! 3rd - Is active? -> $3"
echo "! 4th - Session keys queued? -> $4"
echo "! 5th - Era -> $5"
echo "! 6th - Session -> $6"
echo "! 7th - Eras session index -> $7"
echo "! -------------------------------"
if [ "$3" = "true" ]
then
  echo "! 🟢 -> 😎"
else
  echo "! 🔴 -> 😤"
fi
#
# ***** END *****

