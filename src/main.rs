use futures_util::future::join;
use lowestbins::{error::Result, fetch::fetch_auctions, server::start_server, CONFIG, SOURCE, STORE};
use mimalloc::MiMalloc;
use tokio::{signal, time, time::Duration};

use std::env;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
static LOGO: &str = include_str!(concat!(env!("OUT_DIR"), "/logo.txt"));

pub fn create_basic_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .max_blocking_threads(32)
        .build()
        .unwrap()
}

async fn wait_for_shutdown() -> Result<()> {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to install Ctrl+C handler: {e}"))
    };

    #[cfg(unix)]
    let terminate = async {
        let mut sig = signal::unix::signal(signal::unix::SignalKind::terminate())
            .map_err(|e| anyhow::anyhow!("Failed to install SIGTERM handler: {e}"))?;
        sig.recv().await;
        Ok::<(), anyhow::Error>(())
    };

    #[cfg(windows)]
    let terminate = async {
        let mut ctrl_break = signal::windows::ctrl_break()
            .map_err(|e| anyhow::anyhow!("Failed to install Ctrl+Break: {e}"))?;
        let mut ctrl_close = signal::windows::ctrl_close()
            .map_err(|e| anyhow::anyhow!("Failed to install Ctrl+Close: {e}"))?;
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown()
            .map_err(|e| anyhow::anyhow!("Failed to install Ctrl+Shutdown: {e}"))?;

        tokio::select! {
            _ = ctrl_break.recv() => {},
            _ = ctrl_close.recv() => {},
            _ = ctrl_shutdown.recv() => {},
        }
        Ok::<(), anyhow::Error>(())
    };

    #[cfg(not(any(unix, windows)))]
    let terminate = std::future::pending::<Result<()>>();

    tokio::select! {
        res = ctrl_c => {
            res?;
            tracing::info!("SIGINT (Ctrl+C) received");
        },
        res = terminate => {
            res?;
            tracing::info!("Termination signal received");
        },
    }
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let rt = create_basic_runtime();

    let res = format!(
        "Made by Tricked-dev - source: {SOURCE}\nStore: {}, Overwrites {:?}, Update Seconds: {}",
        &*STORE,
        &CONFIG.overwrites,
        &CONFIG.update_seconds,
    );
    println!("{}", LOGO);
    res.lines()
        .map(|s| tracing::info!("{}", s))
        .for_each(drop);

    rt.spawn(async {
        let dur = Duration::from_secs(CONFIG.update_seconds);
        let mut interval = time::interval(dur);
        interval.tick().await;
        loop {
            join(
                async {
                    if let Err(e) = fetch_auctions().await {
                        tracing::error!("Error occured while fetching auctions {e:?}\n",)
                    }
                },
                interval.tick(),
            )
            .await;
        }
    });

    rt.block_on(async {
        tokio::select! {
            res = start_server() => {
                if let Err(e) = res {
                    tracing::error!("Server crashed: {e}");
                }
            },
            shutdown_res = wait_for_shutdown() => {
                if let Err(e) = shutdown_res {
                    tracing::error!("Shutdown handler error: {e}");
                }
                tracing::info!("Shutting down");
            }
        }
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
