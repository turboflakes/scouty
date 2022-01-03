# The MIT License (MIT)
# Copyright © 2021 Aukbit Ltd.
# 
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
# 
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
# 
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

#!/bin/bash
#
# > make a file executable
# chmod +x ./skipper-update.sh

DIRNAME="/opt/skipper-cli"
FILENAME="$DIRNAME/skipper"

read -p "Enter the Skipper version that you would like to download (e.g.: 0.1.1): " INPUT_VERSION
if [ "$INPUT_VERSION" = "" ]; then
                    INPUT_VERSION="0.1.1"
fi

URI="https://github.com/turboflakes/skipper/releases/download/v$INPUT_VERSION/skipper"
URI_SHA256="https://github.com/turboflakes/skipper/releases/download/v$INPUT_VERSION/skipper.sha256"
wget $URI && wget $URI_SHA256

if sha256sum -c skipper.sha256 2>&1 | grep -q 'OK'
then
        if [ ! -d "$DIRNAME" ]
        then
                mkdir $DIRNAME
        fi
        if [[ -f "$FILENAME" ]]
        then
                mv "$FILENAME" "$FILENAME.backup"
        fi
        rm skipper.sha256
        chmod +x skipper
        mv skipper "$FILENAME"
        echo "** skipper v$INPUT_VERSION successfully downloaded and verified $FILENAME **"
else
        echo "Error: SHA256 doesn't match!"
        rm "$FILENAME*"
fi