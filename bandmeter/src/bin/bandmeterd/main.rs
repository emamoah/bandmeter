mod stats;
mod util;

use crate::{stats::Stats, util::*};
use anyhow::Context as _;
use aya::{
    Btf,
    maps::perf::{AsyncPerfEventArray, AsyncPerfEventArrayBuffer},
    programs::FExit,
    util::online_cpus,
};
use bandmeter_common::Event;
use bytes::BytesMut;
use std::fs;
use std::sync::atomic::AtomicBool;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::mpsc,
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

pub static RECORD_PERIOD_SECS: i64 = 60;
pub static STREAMING: AtomicBool = AtomicBool::new(false);

async fn await_termination() -> std::io::Result<()> {
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    tokio::select! {
        _ = sigterm.recv() => Ok(()),
        _ = sigint.recv() => Ok(())
    }
}

struct Tick {
    period_start: i64,
}

async fn init_record_interval(
    cancellation_token: CancellationToken,
    tx: mpsc::Sender<Tick>,
) -> anyhow::Result<()> {
    loop {
        let period = RECORD_PERIOD_SECS;
        let current_time = util::get_timestamp();
        let since_period_start = current_time % period;
        let remainder = period - since_period_start;
        let duration = tokio::time::Duration::from_secs(remainder as u64);
        let period_start = current_time - since_period_start;

        tokio::select! {
            _ = cancellation_token.cancelled() => break,
            _ = tokio::time::sleep(duration) => {
                if let Err(_) = tx.send(Tick{period_start}).await {
                    break;
                }
            }
        }
    }

    Ok(())
}

async fn read_events(
    mut perf_buffer: AsyncPerfEventArrayBuffer<aya::maps::MapData>,
    channel_sender: mpsc::UnboundedSender<Event>,
    cancellation_token: CancellationToken,
    cpu: u32,
) -> anyhow::Result<()> {
    let mut events_buffer = (0..16)
        .map(|_| BytesMut::with_capacity(size_of::<Event>()))
        .collect::<Vec<_>>();

    'outer: loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                info!("Stop signal received on CPU {}, quitting", cpu);
                break 'outer;
            }
            events = perf_buffer.read_events(&mut events_buffer) => {
                let events = events.with_context(|| format!("error reading events on CPU {}", cpu))?;

                if events.lost > 0 {
                    warn!("Lost {} events", events.lost);
                }

                for i in 0..events.read {
                    let event_bytes = (&events_buffer[i]).as_ptr() as *const Event;
                    let event = unsafe { event_bytes.read_unaligned() };

                    if let Err(_) = channel_sender.send(event) {
                        info!("CPU {:<2}: main queue receiver closed, exiting loop", cpu);
                        break 'outer;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn handle_events(
    mut event_rx: mpsc::UnboundedReceiver<Event>,
    mut timer_rx: mpsc::Receiver<Tick>,
) -> anyhow::Result<()> {
    let mut stats = Stats::new()?;

    loop {
        tokio::select! {
            Some(tick) = timer_rx.recv() => {
                stats.flush(tick.period_start)?;
            }

            Some(event) = event_rx.recv() => {
                stats.update(&event);
            }

            else => {
                info!("Queues closed, flushing remaining stats");

                let current_time = util::get_timestamp();
                let period_start = current_time - (current_time % RECORD_PERIOD_SECS);
                stats.flush(period_start)?;

                break;
            }
        }
    }

    Ok(())
}

fn set_rlimit() {
    // Bump the memlock rlimit. This is needed for older kernels that don't use the
    // new memcg based accounting, see https://lwn.net/Articles/837122/
    let rlim = libc::rlimit {
        rlim_cur: libc::RLIM_INFINITY,
        rlim_max: libc::RLIM_INFINITY,
    };
    let ret = unsafe { libc::setrlimit(libc::RLIMIT_MEMLOCK, &rlim) };
    if ret != 0 {
        debug!("Remove limit on locked memory failed, ret is: {ret}");
    }
}

fn load_ebpf_program() -> anyhow::Result<aya::Ebpf> {
    // This will include your eBPF object file as raw bytes at compile-time and load it at
    // runtime. This approach is recommended for most real-world use cases. If you would
    // like to specify the eBPF program at runtime rather than at compile-time, you can
    // reach for `Bpf::load_file` instead.
    let mut ebpf = aya::Ebpf::load(aya::include_bytes_aligned!(concat!(
        env!("OUT_DIR"),
        "/bandmeter-ebpf-bin"
    )))
    .context("unable to load eBPF bytecode")?;

    let btf = Btf::from_sys_fs().context("BTF from sysfs")?;
    for prog in vec![
        "inet_recvmsg",
        "inet6_recvmsg",
        "inet_sendmsg",
        "inet6_sendmsg",
    ] {
        let program: &mut FExit = ebpf
            .program_mut(&format!("handle_{prog}"))
            .unwrap()
            .try_into()?;
        program
            .load(prog, &btf)
            .context(format!("unable to load eBPF program '{}'", prog))?;
        program
            .attach()
            .context(format!("unable to attach eBPF program '{}'", prog))?;
    }

    Ok(ebpf)
}

pub async fn start() -> anyhow::Result<()> {
    set_rlimit();

    let mut ebpf = load_ebpf_program()?;
    let mut events = AsyncPerfEventArray::try_from(ebpf.take_map("EVENT_QUEUE").unwrap())?;

    let (main_queue_tx, main_queue_rx) = mpsc::unbounded_channel::<Event>();

    let mut task_set = JoinSet::new();

    let (timer_tx, timer_rx) = mpsc::channel::<Tick>(1);
    task_set.spawn(handle_events(main_queue_rx, timer_rx));

    let cxl_token = CancellationToken::new();

    let cpus = online_cpus().map_err(|(_, error)| error)?;
    for cpu in cpus {
        let perf_buf = events.open(cpu, None)?;

        let cxl_token = cxl_token.clone();

        task_set.spawn(read_events(perf_buf, main_queue_tx.clone(), cxl_token, cpu));
    }

    drop(main_queue_tx);

    task_set.spawn(init_record_interval(cxl_token.clone(), timer_tx));

    task_set.spawn(async move {
        if let Err(_) = await_termination().await {
            warn!("Unable to set up signal handlers, clean shutdown impossible");
            return Ok(());
        }

        cxl_token.cancel();

        Ok(())
    });

    info!("All tasks started");
    while let Some(join_result) = task_set.join_next().await {
        join_result??
    }
    info!("Shutdown complete");

    Ok(())
}

async fn prog_main() -> anyhow::Result<()> {
    let db_dir = db_dir()?;
    fs::create_dir_all(&db_dir).context(format!("error creating directory {db_dir}"))?;

    let db = util::get_db()?;
    db.execute(
        "CREATE TABLE IF NOT EXISTS stats(
            timestamp_utc INTEGER NOT NULL,
            exe           TEXT,
            raddr         TEXT    NOT NULL,
            send          INTEGER NOT NULL,
            recv          INTEGER NOT NULL
        )",
        (),
    )
    .context("unable to execute CREATE TABLE query")?;

    db.execute("CREATE INDEX IF NOT EXISTS idx_exe ON stats (exe)", ())
        .context("unable to execute CREATE INDEX query")?;

    start().await
}

#[tokio::main]
async fn main() {
    if let Err(e) = prog_main().await {
        error!("Error: {e:?}")
    }
}
