docker build -t okexchain/brc20s:0.0.1-rollback --file Dockerfile .
docker build -t okexchain/brc20s:0.0.1-no-rollback --file Dockerfile.norollback .
