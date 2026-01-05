use clap::Parser as _;
use fifocore::FIFOCore;

#[derive(clap::Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(
        //last = true,
        num_args = 1..,
        help = "args to pass through to Cargo"
    )]
    buses_to_open: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::try_parse()?;
    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or("debug,jni=off,hyper=debug"),
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("ReduxFIFO")
        .build()
        .expect("could not start ReduxFIFO");

    let fifocore = FIFOCore::new(rt.handle().clone());
    rt.block_on(async_main(fifocore, cli))
}

async fn async_main(fifocore: FIFOCore, cli: Cli) -> anyhow::Result<()> {
    let (shutdown_send, shutdown_recv) = tokio::sync::watch::channel(false);
    let web_task = fifocore
        .runtime()
        .spawn(canandmiddleware::rest_server::run_web_server(
            shutdown_recv,
            fifocore.clone(),
        ));
    for bus in cli.buses_to_open {
        log::info!("attempt open bus {bus}");
        let id = fifocore.open_or_get_bus(&bus).unwrap();
        log::info!("opened bus {bus} on id {id}");
    }

    wait_for_term().await.unwrap();
    let _ = shutdown_send.send(true);
    web_task.await?;
    Ok(())
}

#[cfg(unix)]
async fn wait_for_term() -> anyhow::Result<()> {
    let mut signal_future =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = signal_future.recv() => {}
    }
    Ok(())
}

#[cfg(not(unix))]
async fn wait_for_term() -> anyhow::Result<()> {
    tokio::signal::ctrl_c().await?;
    Ok(())
}
