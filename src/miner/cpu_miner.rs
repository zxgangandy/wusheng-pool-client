use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::collections::VecDeque;
use std::sync::atomic::AtomicU32;
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, info};
use rayon::{ThreadPool, ThreadPoolBuilder};

use tokio::{
    net::TcpStream,
    net::TcpListener,
    task,
    time::{sleep, timeout},
    sync::{mpsc::channel},
};


pub struct CpuMiner {
    thread_pools: Arc<Vec<Arc<ThreadPool>>>,
    total_proofs: Arc<AtomicU32>,
}


impl CpuMiner {
    pub fn new(threads: u16,) -> Self {

        let mut thread_pools: Vec<Arc<ThreadPool>> = Vec::new();
        let pool_count;
        let pool_threads;
        if threads % 12 == 0 {
            pool_count = threads / 12;
            pool_threads = 12;
        } else if threads % 10 == 0 {
            pool_count = threads / 10;
            pool_threads = 10;
        } else if threads % 8 == 0 {
            pool_count = threads / 8;
            pool_threads = 8;
        } else {
            pool_count = threads / 6;
            pool_threads = 6;
        }

        for index in 0..pool_count {
            let pool = ThreadPoolBuilder::new()
                .stack_size(8 * 1024 * 1024)
                .num_threads(pool_threads as usize)
                .thread_name(move |idx| format!("cpu-miner-{}-{}", index, idx))
                .build().unwrap();
            thread_pools.push(Arc::new(pool));
        }
        info!("Created {} thread pools with {} threads each", thread_pools.len(), pool_threads);

        CpuMiner {
            thread_pools: Arc::new(thread_pools),
            total_proofs: Arc::new(Default::default()),
        }
    }

}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    #[test]
    fn test_vecdeque() {
        let log = VecDeque::from(vec![0;60]);
        assert!(log.get(0).is_some());
        let mut log = VecDeque::with_capacity(60);
        assert!(log.get(0).is_none());
        let cap = log.capacity();
        for i in 1..=cap {
            log.push_front(i);
        }
        assert_eq!(log.get(0).unwrap(), &cap);
        assert_eq!(log.get(cap - 1).unwrap(), &1);
        log.push_front(1);
        assert_eq!(log.get(cap - 1).unwrap(), &2);
    }
}
