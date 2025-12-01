use bench_matrix::{
  criterion_runner::async_suite::AsyncBenchmarkSuite,
  AbstractCombination, MatrixCellValue,
};
use criterion::{criterion_group, criterion_main, AxisScale, BenchmarkGroup, Criterion, PlotConfiguration, Throughput};
use rand::prelude::*;
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
  NetworkSim,
  DiskSim,
}

#[derive(Debug, Clone)]
pub struct ConfigAsync {
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

static ASYNC_GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

// Extractor function remains the same as it operates on AbstractCombination indices
fn extract_async_config(combo: &AbstractCombination) -> Result<ConfigAsync, String> {
  let workload_str = combo.get_tag(0)?; // Corresponds to "WorkloadType" name
  let workload = match workload_str {
    "Network" => AsyncWorkloadType::NetworkSim,
    "Disk" => AsyncWorkloadType::DiskSim,
    _ => return Err(format!("Unknown async workload type: {}", workload_str)),
  };
  let packet_size = combo.get_u64(1)? as u32; // Corresponds to "PktSize" name
  let concurrent_ops = combo.get_u64(2)? as u16; // Corresponds to "Concurrency" name

  Ok(ConfigAsync {
    workload,
    packet_size,
    concurrent_ops,
  })
}

fn async_global_setup(cfg: &ConfigAsync) -> Result<(), String> {
  println!(
    "[ASYNC NAMED GLOBAL SETUP] Config: {:?}, Counter: {}",
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
    let mut local_rng = StdRng::from_os_rng();
    let data_packet = (0..cfg_clone.packet_size).map(|_| local_rng.random::<u8>()).collect();
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
    } else { // Handle case of 0 concurrent_ops if it means a single base operation
      tokio::time::sleep(Duration::from_micros(delay_micros_per_op)).await;
    }
    let _checksum = state.data_packet.iter().fold(0u8, |acc, &x| acc.wrapping_add(x));
    let duration = start_time.elapsed();
    // If concurrent_ops is 0, this logic might need adjustment depending on what ops_this_iteration tracks
    ctx.ops_this_iteration += if concurrent_ops > 0 { concurrent_ops as u32} else { 1 };
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
    "[ASYNC NAMED GLOBAL TEARDOWN] Config: {:?}, Counter: {}",
    cfg,
    ASYNC_GLOBAL_COUNTER.load(Ordering::SeqCst)
  );
  Ok(())
}

// This function will be called by the main benchmark runner
pub fn benchmark_async_suite_named(c: &mut Criterion) {
  let rt = Runtime::new().expect("Failed to create Tokio runtime for async_example benchmarks");
  println!("\n--- Running Async Named Benchmarks from async_named.rs ---");

  // Define parameter axes
  let parameter_axes = vec![
    // Axis 0: Workload Type
    vec![
      MatrixCellValue::Tag("Network".to_string()),
      MatrixCellValue::Tag("Disk".to_string()),
    ],
    // Axis 1: Packet Size
    vec![MatrixCellValue::Unsigned(64), MatrixCellValue::Unsigned(512)],
    // Axis 2: Concurrent Operations
    vec![MatrixCellValue::Unsigned(1), MatrixCellValue::Unsigned(4)],
  ];

  // Define names for these axes
  let parameter_names = vec![
    "WorkloadType".to_string(),
    "PktSize".to_string(),
    "Concurrency".to_string(),
  ];

  let async_suite = AsyncBenchmarkSuite::new(
    c,
    &rt,
    "AsyncNamedSuite".to_string(),  // Base name for the suite
    Some(parameter_names),          // Pass the defined parameter names
    parameter_axes,
    Box::new(extract_async_config), // Extractor function remains the same
    async_setup_fn,
    async_benchmark_logic_fn,
    async_teardown_fn,
  )
  .global_setup(async_global_setup)
  .global_teardown(async_global_teardown)
  .configure_criterion_group(|group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>| {
    group
      .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic))
      .sample_size(10) // You might want to adjust this based on benchmark stability/duration
      .measurement_time(Duration::from_secs(3)); // And this too
  })
  .throughput(|cfg: &ConfigAsync| {
    // Throughput can be based on concurrent_ops or total bytes processed, etc.
    // If concurrent_ops can be 0, decide what Throughput::Elements(0) means or adjust.
    // For now, let's assume concurrent_ops > 0 for throughput calculation.
    if cfg.concurrent_ops > 0 {
        Throughput::Elements(cfg.concurrent_ops as u64)
    } else {
        Throughput::Elements(1) // Default for single operation if concurrent_ops is 0
    }
  });

  async_suite.run();
}

criterion_group!(async_benches_named, benchmark_async_suite_named);
criterion_main!(async_benches_named);