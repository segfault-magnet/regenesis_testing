#!/usr/bin/env bash
#
out_file="$1"

# around 21GB
MAX_REPEAT=10000000
#MAX_REPEAT=2

if [[ -z $out_file ]]; then
	echo "Usage $0 OUT_FILE"
	exit 1
fi

cp template.json "$out_file"

echo '"coins": [' >>"$out_file"

coin="$(cat one_coin.json)"
seq $MAX_REPEAT |
	while read -r; do
		echo "$coin,"
	done >>"$out_file"
echo "$coin]," >>"$out_file"

echo '"contracts": [' >>"$out_file"

contract="$(cat one_contract.json)"
seq $MAX_REPEAT |
	while read -r; do
		echo "$contract,"
	done >>"$out_file"
echo "$contract]" >>"$out_file"

echo '}}' >>"$out_file"
