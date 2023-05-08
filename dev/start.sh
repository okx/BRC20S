#!/bin/sh

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" > /dev/null 2>&1 && pwd)"

BTC_RPC_USER=$1
BTC_RPC_PASS=$2
BTC_NODE_URL=$3

if [ "" == "$BTC_RPC_USER" ]; then
  echo "need bitcoin rpc user"
  exit 1
fi

if [ "" == "$BTC_RPC_PASS" ]; then
  echo "need bitcoin rpc password"
  exit 1
fi

if [ "" == "$BTC_NODE_URL" ]; then
  BTC_NODE_URL='10.254.22.166:7011'
  echo "using default btc rpc url $BTC_NODE_URL"
fi


cargo run -- --rpc-url $BTC_NODE_URL --bitcoin-rpc-user $BTC_RPC_USER --bitcoin-rpc-pass $BTC_RPC_PASS --data-dir $DIR/data --first-inscription-height=779832  server
