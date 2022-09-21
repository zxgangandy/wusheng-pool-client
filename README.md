# wusheng-pool-client
wusheng-pool-client

# generate account
- run --package wusheng-pool-client --bin wusheng-pool-client
  -- gen-account  --file=config/account-backup.txt

# cpu mining
- run --package wusheng-pool-client --bin wusheng-pool-client
  -- cpu-mining  --address=aleo1x3dm5pzdfs8hxg6xckx5lnndmarhnpe03h7lcg32c63dhc5pdcfqht927g 
  --pool-server=127.0.0.1:8888 --worker=1 --threads=8

# design
- manager->multi miner/one stats/one stratum client
- one miner->one message channel
