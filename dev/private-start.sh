rm -rf ./_cache
../target/release/ord --log-level=INFO --data-dir=./_cache --index-sats --rpc-url=http://18.167.77.79:18443 --regtest --bitcoin-rpc-user bitcoinrpc --bitcoin-rpc-pass bitcoinrpc server
