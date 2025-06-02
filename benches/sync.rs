use bench_matrix::{
  criterion_runner::sync_suite::SyncBenchmarkSuite,
  AbstractCombination, MatrixCellValue,
};
use criterion::{criterion_group, criterion_main, AxisScale, Criterion, PlotConfiguration, Throughput};
use rand::{prelude::*, rng};
use std::{
  sync::atomic::{AtomicUsize, Ordering},
  thread,
  time::{Duration, Instant},
};

// --- Configuration for Sync Benchmarks ---
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncAlgorithm {
  // Made public
  SortData,
  ProcessData,
}

#[derive(Debug, Clone)]
pub struct ConfigSync {
  // Made public
  pub algorithm: SyncAlgorithm,
  pub data_elements: usize,
  pub intensity: String,
}

#[derive(Debug, Default)]
struct SyncContext {
  items_processed_in_batch: usize,
}

struct SyncState {
  dataset: Vec<u64>,
  aux_buffer: Vec<u64>,
}

static SYNC_GLOBAL_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn extract_sync_config(combo: &AbstractCombination) -> Result<ConfigSync, String> {
  let algo_str = combo.get_tag(0)?;
  let algorithm = match algo_str {
    "Sort" => SyncAlgorithm::SortData,
    "Process" => SyncAlgorithm::ProcessData,
    _ => return Err(format!("Unknown sync algorithm type: {}", algo_str)),
  };
  let data_elements = combo.get_u64(1)? as usize;
  let intensity = combo.get_string(2)?.to_string();

  Ok(ConfigSync {
    algorithm,
    data_elements,
    intensity,
  })
}

fn sync_global_setup(cfg: &ConfigSync) -> Result<(), String> {
  println!(
    "[SYNC GLOBAL SETUP] File: sync_example.rs, Config: {:?}, Counter: {}",
    cfg,
    SYNC_GLOBAL_COUNTER.fetch_add(1, Ordering::SeqCst)
  );
  Ok(())
}

fn sync_setup_fn(cfg: &ConfigSync) -> Result<(SyncContext, SyncState), String> {
  thread::sleep(Duration::from_micros(20));
  let mut rng = rng();
  let dataset: Vec<u64> = (0..cfg.data_elements).map(|_| rng.gen_range(0..100_000)).collect();
  let aux_buffer = vec![0; cfg.data_elements];
  Ok((SyncContext::default(), SyncState { dataset, aux_buffer }))
}

fn sync_benchmark_logic_fn(
  mut ctx: SyncContext,
  mut state: SyncState,
  cfg: &ConfigSync,
) -> (SyncContext, SyncState, Duration) {
  let start_time = Instant::now();
  let intensity_multiplier = match cfg.intensity.as_str() {
    "Low" => 1,
    "Medium" => 3,
    "High" => 10,
    _ => 1,
  };
  match cfg.algorithm {
    SyncAlgorithm::SortData => {
      let mut data_to_sort = state.dataset.clone();
      for _ in 0..intensity_multiplier {
        data_to_sort.sort_unstable();
      }
      state.aux_buffer = data_to_sort;
    }
    SyncAlgorithm::ProcessData => {
      let mut sum = 0u64;
      for &val in &state.dataset {
        for _ in 0..intensity_multiplier {
          sum = sum.wrapping_add(val.wrapping_mul(3).wrapping_sub(val / 2));
        }
      }
      if !state.aux_buffer.is_empty() {
        state.aux_buffer[0] = sum;
      }
    }
  }
  let duration = start_time.elapsed();
  ctx.items_processed_in_batch += 1;
  (ctx, state, duration)
}

fn sync_teardown_fn(_ctx: SyncContext, _state: SyncState, _cfg: &ConfigSync) {
  thread::sleep(Duration::from_micros(10));
}

fn sync_global_teardown(cfg: &ConfigSync) -> Result<(), String> {
  println!(
    "[SYNC GLOBAL TEARDOWN] File: sync_example.rs, Config: {:?}, Counter: {}",
    cfg,
    SYNC_GLOBAL_COUNTER.load(Ordering::SeqCst)
  );
  Ok(())
}

// This function will be called by the main benchmark runner
pub fn benchmark_sync_suite(c: &mut Criterion) {
  let parameter_axes = vec![
    vec![
      MatrixCellValue::Tag("Sort".to_string()),
      MatrixCellValue::Tag("Process".to_string()),
    ],
    vec![MatrixCellValue::Unsigned(100), MatrixCellValue::Unsigned(500)], // data_elements
    vec![
      MatrixCellValue::String("Low".to_string()),
      MatrixCellValue::String("Medium".to_string()),
    ], // intensity
  ];

  let sync_suite = SyncBenchmarkSuite::new(
    c,
    "SyncExampleFileSuite".to_string(), // Unique suite name
    parameter_axes,
    Box::new(extract_sync_config),
    sync_setup_fn,
    sync_benchmark_logic_fn,
    sync_teardown_fn,
  )
  .global_setup(sync_global_setup)
  .global_teardown(sync_global_teardown)
  .configure_criterion_group(|group| {
    group
      .plot_config(PlotConfiguration::default().summary_scale(AxisScale::Linear))
      .sample_size(15)
      .measurement_time(Duration::from_secs(1));
  })
  .throughput(|cfg: &ConfigSync| Throughput::Elements(cfg.data_elements as u64));

  sync_suite.run();
}

criterion_group!(sync_benches, benchmark_sync_suite);
criterion_main!(sync_benches);
