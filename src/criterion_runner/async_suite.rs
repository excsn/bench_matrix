// bench_matrix/src/criterion_runner/async_suite.rs

#![cfg(feature = "criterion_integration")]

use super::{ExtractorFn, GlobalSetupFn, GlobalTeardownFn};
use crate::generator::generate_combinations;
use crate::params::{AbstractCombination, MatrixCellValue};

use criterion::{
  measurement::WallTime, AxisScale, Bencher, BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration, Throughput,
};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use tokio::runtime::Runtime;

pub type AsyncSetupFn<S, Cfg, CtxT, SetupErr = String> =
  fn(&Runtime, &Cfg) -> Pin<Box<dyn Future<Output = Result<(CtxT, S), SetupErr>> + Send>>;
pub type AsyncBenchmarkLogicFn<S, Cfg, CtxT> =
  fn(CtxT, S, &Cfg) -> Pin<Box<dyn Future<Output = (CtxT, S, Duration)> + Send>>;
pub type AsyncTeardownFn<S, Cfg, CtxT> = fn(CtxT, S, &Runtime, &Cfg) -> Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct AsyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr = String, SetupErr = String> {
  criterion: &'s mut Criterion<WallTime>,
  runtime: &'s Runtime,
  suite_base_name: String,
  parameter_axes: Vec<Vec<MatrixCellValue>>,
  extractor_fn: ExtractorFn<Cfg, ExtErr>,
  parameter_names: Option<Vec<String>>,
  global_setup_fn: Option<GlobalSetupFn<Cfg>>,
  setup_fn: AsyncSetupFn<S, Cfg, CtxT, SetupErr>,
  benchmark_logic_fn: AsyncBenchmarkLogicFn<S, Cfg, CtxT>,
  teardown_fn: AsyncTeardownFn<S, Cfg, CtxT>,
  global_teardown_fn: Option<GlobalTeardownFn<Cfg>>,
  criterion_group_configurator: Option<Box<dyn for<'g> Fn(&mut BenchmarkGroup<'g, WallTime>)>>,
  throughput_calculator: Option<Box<dyn Fn(&Cfg) -> Throughput>>,
}

impl<'s, S, Cfg, CtxT, ExtErr, SetupErr> AsyncBenchmarkSuite<'s, S, Cfg, CtxT, ExtErr, SetupErr>
where
  S: Send + 'static,
  Cfg: Clone + Debug + Send + Sync + 'static,
  CtxT: Send + 'static,
  ExtErr: Debug,
  SetupErr: Debug,
{
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    criterion: &'s mut Criterion<WallTime>,
    runtime: &'s Runtime,
    suite_base_name: String,
    parameter_names: Option<Vec<String>>,
    parameter_axes: Vec<Vec<MatrixCellValue>>,
    extractor_fn: ExtractorFn<Cfg, ExtErr>,
    setup_fn: AsyncSetupFn<S, Cfg, CtxT, SetupErr>,
    benchmark_logic_fn: AsyncBenchmarkLogicFn<S, Cfg, CtxT>,
    teardown_fn: AsyncTeardownFn<S, Cfg, CtxT>,
  ) -> Self {
    if let Some(names) = &parameter_names {
      if names.len() != parameter_axes.len() {
        // Or panic, or return Result. For now, let's warn and proceed without names.
        eprintln!(
                "[BenchMatrix::Async] [WARN] Suite '{}': Mismatch between number of parameter_names ({}) and parameter_axes ({}). Parameter names will be ignored for ID generation.",
                suite_base_name,
                names.len(),
                parameter_axes.len()
            );
        // Fallback to None if there's a mismatch.
        // Or, one could make parameter_names: Vec<String> and require it to match.
      }
    }

    Self {
      criterion,
      runtime,
      suite_base_name,
      parameter_names,
      parameter_axes,
      extractor_fn,
      global_setup_fn: None,
      setup_fn,
      benchmark_logic_fn,
      teardown_fn,
      global_teardown_fn: None,
      criterion_group_configurator: None,
      throughput_calculator: None,
    }
  }

  pub fn global_setup(mut self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self {
    self.global_setup_fn = Some(Box::new(f));
    self
  }

  pub fn global_teardown(mut self, f: impl FnMut(&Cfg) -> Result<(), String> + 'static) -> Self {
    self.global_teardown_fn = Some(Box::new(f));
    self
  }

  pub fn configure_criterion_group(mut self, f: impl for<'g> Fn(&mut BenchmarkGroup<'g, WallTime>) + 'static) -> Self {
    self.criterion_group_configurator = Some(Box::new(f));
    self
  }

  pub fn throughput(mut self, f: impl Fn(&Cfg) -> Throughput + 'static) -> Self {
    self.throughput_calculator = Some(Box::new(f));
    self
  }

  pub fn run(mut self) {
    let abstract_combinations = generate_combinations(&self.parameter_axes);

    if abstract_combinations.is_empty() {
      let reason = if self.parameter_axes.is_empty() {
        "no parameter axes defined"
      } else {
        "no combinations generated (e.g., an axis was empty)"
      };
      eprintln!(
        "[BenchMatrix::Async] Suite '{}': {}. Nothing to run.",
        self.suite_base_name, reason
      );
      return;
    }

    let mut variants_run_count = 0;
    let mut variants_skipped_extraction = 0;
    let mut variants_skipped_global_setup = 0;

    for abstract_combo in abstract_combinations {
      let concrete_config = match (self.extractor_fn)(&abstract_combo) {
        Ok(cfg) => cfg,
        Err(e) => {
          eprintln!(
                        "[BenchMatrix::Async] [ERROR] Suite '{}', Combination ID '{}': Failed to extract concrete configuration: {:?}. Skipping this combination.",
                        self.suite_base_name, abstract_combo.id_suffix(), e
                    );
          variants_skipped_extraction += 1;
          continue;
        }
      };

      if let Some(ref mut global_setup) = self.global_setup_fn {
        if let Err(e) = global_setup(&concrete_config) {
          eprintln!(
                        "[BenchMatrix::Async] [ERROR] Suite '{}', Config (ID '{}', Detail {:?}): Global setup failed: {}. Skipping benchmarks for this configuration.",
                        self.suite_base_name, abstract_combo.id_suffix(), concrete_config, e
                    );
          variants_skipped_global_setup += 1;
          if let Some(ref mut global_teardown_on_setup_fail) = self.global_teardown_fn {
            if let Err(td_err) = global_teardown_on_setup_fail(&concrete_config) {
              eprintln!(
                                "[BenchMatrix::Async] [WARN] Suite '{}', Config (ID '{}'): Global teardown after global setup failure also failed: {}",
                                self.suite_base_name, abstract_combo.id_suffix(), td_err
                            );
            }
          }
          continue;
        }
      }

      let combo_id_suffix = if let Some(names) = &self.parameter_names {
        abstract_combo.id_suffix_with_names(names)
      } else {
        abstract_combo.id_suffix() // Fallback to old method
      };
      
      let group_name = format!("{}{}", self.suite_base_name, combo_id_suffix);
      let mut group = self.criterion.benchmark_group(&group_name);

      if let Some(ref configurator) = self.criterion_group_configurator {
        configurator(&mut group);
      } else {
        group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
        group.sample_size(10);
      }

      if let Some(ref throughput_calc) = self.throughput_calculator {
        group.throughput(throughput_calc(&concrete_config));
      }

      let bench_id = BenchmarkId::from_parameter(abstract_combo.id_suffix());

      let rt_for_iter = self.runtime;
      let cfg_for_iter_template = concrete_config.clone();
      let setup_fn_ptr = self.setup_fn;
      let benchmark_logic_fn_ptr = self.benchmark_logic_fn;
      let teardown_fn_ptr = self.teardown_fn;

      group.bench_function(bench_id, move |b: &mut Bencher<'_, WallTime>| {
        b.to_async(rt_for_iter).iter_custom(|iters_count_hint| {
          let cfg_clone_per_sample = cfg_for_iter_template.clone();
          async move {
            let mut total_duration_for_sample_batch = Duration::new(0, 0);
            for _i in 0..iters_count_hint {
              let (user_ctx, setup_data_instance) = Box::pin((setup_fn_ptr)(rt_for_iter, &cfg_clone_per_sample))
                .await
                .unwrap_or_else(|e| {
                  panic!(
                    "[BenchMatrix::Async] PANIC in sample: Async setup_fn failed for config '{:?}': {:?}",
                    cfg_clone_per_sample, e
                  )
                });

              let (ctx_after_bench, s_after_bench, measured_duration) = Box::pin((benchmark_logic_fn_ptr)(
                user_ctx,
                setup_data_instance,
                &cfg_clone_per_sample,
              ))
              .await;

              total_duration_for_sample_batch += measured_duration;

              Box::pin((teardown_fn_ptr)(
                ctx_after_bench,
                s_after_bench,
                rt_for_iter,
                &cfg_clone_per_sample,
              ))
              .await;
            }
            total_duration_for_sample_batch
          }
        });
      });

      variants_run_count += 1;
      // Global teardown is called *after* the group for this variant is finished.
      if let Some(ref mut global_teardown) = self.global_teardown_fn {
        if let Err(e) = global_teardown(&concrete_config) {
          eprintln!(
            "[BenchMatrix::Async] [WARN] Suite '{}', Config (ID '{}', Detail {:?}): Global teardown failed: {}",
            self.suite_base_name,
            abstract_combo.id_suffix(),
            concrete_config,
            e
          );
        }
      }
      group.finish();
    }

    if variants_skipped_extraction > 0 || variants_skipped_global_setup > 0 {
      eprintln!(
                "[BenchMatrix::Async] Suite '{}' summary: {} variants attempted, {} successfully run, {} skipped (extraction), {} skipped (global setup).",
                self.suite_base_name,
                variants_run_count + variants_skipped_extraction + variants_skipped_global_setup,
                variants_run_count,
                variants_skipped_extraction,
                variants_skipped_global_setup
            );
    } else if variants_run_count > 0 {
      println!(
        "[BenchMatrix::Async] Suite '{}': All {} variants set up for Criterion runs.",
        self.suite_base_name, variants_run_count
      );
    }
  }
}
