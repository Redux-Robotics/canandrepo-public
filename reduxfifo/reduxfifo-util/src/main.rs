#![allow(unused)]
use fifocore::{FIFOCore, ReduxFIFOSessionConfig};

fn main() -> anyhow::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or("debug,jni=off,warp=info,hyper=info"),
    );

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("ReduxFIFO")
        .build()
        .expect("could not start ReduxFIFO");

    let fifocore = FIFOCore::new(rt.handle().clone());
    rt.block_on(async_main(fifocore))
}

async fn async_main(fifocore: FIFOCore) -> anyhow::Result<()> {
    // 4 ok, 6 fail?
    let can_device_id = 0;
    println!("Connect to websocket...");
    //let bus_id = fifocore.open_or_get_bus("ws://10.43.22.2:7244/ws/0")?;
    let bus_id = fifocore.open_or_get_bus("slcan:115200:/dev/cu.usbmodem101")?;
    let session = fifocore.open_managed_session(
        bus_id,
        256,
        ReduxFIFOSessionConfig::new(
            frc_can_id::build_frc_can_id(0x2, 0xe, 0x0, can_device_id),
            frc_can_id::build_frc_can_id(0x1f, 0xff, 0x0, 0x00),
        ),
    )?;

    //let rb = session.read_buffer(256);

    loop {}
}
