use bench_matrix::{
  criterion_runner::async_suite::AsyncBenchmarkSuite,
  AbstractCombination, MatrixCellValue,
};
use criterion::{criterion_group, criterion_main, AxisScale, BenchmarkGroup, Criterion, PlotConfiguration, Throughput};
use rand::{prelude::*, rng};
use std::{
  future::Future,
  pin::Pin,
  sync::atomic::{AtomicUsize, Ordering},
  time::{Duration, Instant},
};
use tokio::runtime::Runtime;

// --- Configuration for Async Benchmarks ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AsyncWorkloadType {
  // Made public if used by aggregator directly
  NetworkSim,
  DiskSim,
}

#[derive(Debug, Clone)]
pub struct ConfigAsync {
  // Made public
  pub workload: AsyncWorkloadType,
  pub packet_size: u32,
  pub concurrent_ops: u16,
}

#[derive(Debug, Default)]
struct AsyncContext {
  ops_this_iteration: u32,
}

struct AsyncState {
  data_packet: Vec<u8>,
  simulated_connections: Vec<String>,
}

// --- Helper: Global state (could be shared or specific to this module) ---
// If shared across bench files, it would need to be in a common lib or `pub static` in one.
// For this example, let's assume it's conceptually separate or managed by the aggregator.
static ASYNC_GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn extract_async_config(combo: &AbstractCombination) -> Result<ConfigAsync, String> {
  let workload_str = combo.get_tag(0)?;
  let workload = match workload_str {
    "Network" => AsyncWorkloadType::NetworkSim,
    "Disk" => AsyncWorkloadType::DiskSim,
    _ => return Err(format!("Unknown async workload type: {}", workload_str)),
  };
  let packet_size = combo.get_u64(1)? as u32;
  let concurrent_ops = combo.get_u64(2)? as u16;

  Ok(ConfigAsync {
    workload,
    packet_size,
    concurrent_ops,
  })
}

fn async_global_setup(cfg: &ConfigAsync) -> Result<(), String> {
  println!(
    "[ASYNC GLOBAL SETUP] File: async_example.rs, Config: {:?}, Counter: {}",
    cfg,
    ASYNC_GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst)
  );
  Ok(())
}

fn async_setup_fn(
  _runtime: &Runtime,
  cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = Result<(AsyncContext, AsyncState), String>> + Send>> {
  let cfg_clone = cfg.clone();
  Box::pin(async move {
    tokio::time::sleep(Duration::from_micros(10)).await;
    let mut rng = rng();
    let data_packet = (0..cfg_clone.packet_size).map(|_| rng.gen::<u8>()).collect();
    let simulated_connections = (0..cfg_clone.concurrent_ops)
      .map(|i| format!("conn-{}-{:?}-{}", i, cfg_clone.workload, cfg_clone.packet_size))
      .collect();
    Ok((
      AsyncContext::default(),
      AsyncState {
        data_packet,
        simulated_connections,
      },
    ))
  })
}

fn async_benchmark_logic_fn(
  mut ctx: AsyncContext,
  state: AsyncState,
  cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = (AsyncContext, AsyncState, Duration)> + Send>> {
  let packet_size = cfg.packet_size;
  let workload = cfg.workload.clone();
  let concurrent_ops = cfg.concurrent_ops;

  Box::pin(async move {
    let start_time = Instant::now();
    let delay_micros_per_op = match workload {
      AsyncWorkloadType::NetworkSim => 10 + packet_size as u64 / 200,
      AsyncWorkloadType::DiskSim => 20 + packet_size as u64 / 100,
    };
    if concurrent_ops > 0 {
      tokio::time::sleep(Duration::from_micros(delay_micros_per_op * concurrent_ops as u64)).await;
    } else {
      tokio::time::sleep(Duration::from_micros(delay_micros_per_op)).await;
    }
    let _checksum = state.data_packet.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    let duration = start_time.elapsed();
    ctx.ops_this_iteration += 1;
    (ctx, state, duration)
  })
}

fn async_teardown_fn(
  _ctx: AsyncContext,
  _state: AsyncState,
  _runtime: &Runtime,
  _cfg: &ConfigAsync,
) -> Pin<Box<dyn Future<Output = ()> + Send>> {
  Box::pin(async move {
    tokio::time::sleep(Duration::from_micros(5)).await;
  })
}

fn async_global_teardown(cfg: &ConfigAsync) -> Result<(), String> {
  println!(
    "[ASYNC GLOBAL TEARDOWN] File: async_example.rs, Config: {:?}, Counter: {}",
    cfg,
    ASYNC_GLOBAL_COUNTER.load(Ordering::SeqCst)
  );
  Ok(())
}

// This function will be called by the main benchmark runner
pub fn benchmark_async_suite(c: &mut Criterion) {
  let rt = Runtime::new().expect("Failed to create Tokio runtime for async_example benchmarks");
  println!("\n--- Running Async Benchmarks from async_example.rs ---");

  let parameter_axes = vec![
    vec![
      MatrixCellValue::Tag("Network".to_string()),
      MatrixCellValue::Tag("Disk".to_string()),
    ],
    vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(512)],
    vec![MatrixCellValue::Unsigned(1), MatrixCellValue::Unsigned(4)],
  ];

  let async_suite = AsyncBenchmarkSuite::new(
    c,
    &rt, // Pass reference to rt
    "AsyncExampleFileSuite".to_string(),
    parameter_axes,
    Box::new(extract_async_config),
    async_setup_fn,
    async_benchmark_logic_fn,
    async_teardown_fn,
  )
  .global_setup(async_global_setup)
  .global_teardown(async_global_teardown)
  .configure_criterion_group(|group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>| {
    group
      .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
      .sample_size(10)
      .measurement_time(Duration::from_secs(1));
  })
  .throughput(|_cfg: &ConfigAsync| {
    // _cfg if not used, or use cfg.concurrent_ops
    Throughput::Elements(1) // Or cfg.concurrent_ops as u64, etc.
  });

  async_suite.run();
}

criterion_group!(async_benches, benchmark_async_suite);
criterion_main!(async_benches);
