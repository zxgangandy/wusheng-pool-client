use std::collections::VecDeque;
use std::{net::SocketAddr,
          sync::{atomic::Ordering, Arc},
          time::Duration,
};
use std::sync::atomic::AtomicU32;
use tokio::{
    net::TcpStream,
    net::TcpListener,
    task,
    time::{sleep, timeout},
    sync::{mpsc::channel},
};
use anyhow::Result;
use anyhow::{anyhow, bail};
use log::{debug, info};
use ansi_term::Colour::{Cyan, Green, Red};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

pub struct Stats{
    total_proofs: Arc<AtomicU32>,
    valid_shares: Arc<AtomicU32>,
    invalid_shares: Arc<AtomicU32>,
    stats_sender: Sender<StatsEvent>,
    handlers: Vec<JoinHandle<()>>,
}

#[derive(Debug)]
pub enum StatsEvent {
    Prove(bool, u32),
    SubmitResult(bool, Option<String>),
    Exit(oneshot::Sender<()>),
}

impl Stats {

    pub fn new() -> Self {
        let (tx, rx) = channel(256);
        let mut stats = Stats {
            total_proofs: Arc::new(Default::default()),
            valid_shares: Arc::new(Default::default()),
            invalid_shares: Arc::new(Default::default()),
            stats_sender: tx,
            handlers: vec![]
        };

        //stats.start_receiver(rx);
        //stats.start_calculator();

        stats
    }

    pub fn sender(&self) -> Sender<StatsEvent> {
        self.stats_sender.clone()
    }

    fn start_receiver(mut self, mut rx: Receiver<StatsEvent>) {
        let handler = task::spawn(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    StatsEvent::Prove(valid, weight) => {
                        self.update_total_proofs();
                    }
                    StatsEvent::SubmitResult(is_valid, msg) => {
                        //self.print_shares(is, msg).await;
                    }
                    StatsEvent::Exit(responder) => {
                        for handler in self.handlers {
                            handler.abort();
                        }
                        responder.send(()).expect("Failed to respond exit msg");
                        debug!("Statistic exited");
                        return
                    },
                }
            }
        });

        //self.handlers.push(handler);
    }

    pub fn update_total_proofs(&self) {
        self.total_proofs.fetch_add(1, Ordering::SeqCst);
    }

    /// start calculator
    pub async fn start_calculator(&mut self) ->Result<()> {
        let total_proofs = self.total_proofs.clone();
        let handler = task::spawn(async move {
            let mut log = VecDeque::<u32>::from(vec![0; 60]);
            loop {
                sleep(Duration::from_secs(60)).await;
                let proofs = total_proofs.load(Ordering::SeqCst);
                log.push_back(proofs);
                let m1 = *log.get(59).unwrap_or(&0);
                let m5 = *log.get(55).unwrap_or(&0);
                let m15 = *log.get(45).unwrap_or(&0);
                let m30 = *log.get(30).unwrap_or(&0);
                let m60 = log.pop_front().unwrap_or_default();
                info!(
                    "{}",
                    Cyan.normal().paint(format!(
                        "Total proofs: {} (1m: {} p/s, 5m: {} p/s, 15m: {} p/s, 30m: {} p/s, 60m: {} p/s)",
                        proofs,
                        Self::calculate_proof_rate(proofs, m1, 1),
                        Self::calculate_proof_rate(proofs, m5, 5),
                        Self::calculate_proof_rate(proofs, m15, 15),
                        Self::calculate_proof_rate(proofs, m30, 30),
                        Self::calculate_proof_rate(proofs, m60, 60),
                    ))
                );
            }
        });

        self.handlers.push(handler);

        debug!("Created proof rate calculator");

        Ok(())
    }

    fn calculate_proof_rate(now: u32, past: u32, interval: u32) -> Box<str> {
        if interval < 1 {
            return Box::from("---");
        }
        if now <= past || past == 0 {
            return Box::from("---");
        }
        let rate = (now - past) as f64 / (interval * 60) as f64;
        Box::from(format!("{:.2}", rate))
    }

    pub async fn print_shares(&self, success: bool, msg: Option<String>) {
        if success {
            let valid_minus_1 = self.valid_shares.fetch_add(1, Ordering::SeqCst);
            let valid = valid_minus_1 + 1;
            let invalid = self.invalid_shares.load(Ordering::SeqCst);

            if let Some(msg) = msg {
                info!(
                    "{}",
                    Green.normal().paint(format!(
                        "Share accepted: {}  {} / {} ({:.2}%)",
                        msg,
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            } else {
                info!(
                    "{}",
                    Green.normal().paint(format!(
                        "Share accepted  {} / {} ({:.2}%)",
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            }
        } else {
            let invalid_minus_1 = self.invalid_shares.fetch_add(1, Ordering::SeqCst);
            let invalid = invalid_minus_1 + 1;
            let valid = self.valid_shares.load(Ordering::SeqCst);
            if let Some(msg) = msg {
                info!(
                    "{}",
                    Red.normal().paint(format!(
                        "Share rejected: {}  {} / {} ({:.2}%)",
                        msg,
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            } else {
                info!(
                    "{}",
                    Red.normal().paint(format!(
                        "Share rejected  {} / {} ({:.2}%)",
                        valid,
                        valid + invalid,
                        (valid as f64 / (valid + invalid) as f64) * 100.0
                    ))
                );
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;
    use crate::stats::Stats;

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
