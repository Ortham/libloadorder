use std::convert::TryFrom;
use std::fmt;
use std::fmt::Display;
use std::fs::{copy, create_dir, File, FileTimes};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use criterion::{BenchmarkId, Criterion};
use encoding_rs::WINDOWS_1252;
use tempfile::TempDir;

use loadorder::GameId;
use loadorder::GameSettings;
use loadorder::LoadOrderMethod;
use loadorder::WritableLoadOrder;

fn write_load_order_file<T: AsRef<str> + Display>(game_settings: &GameSettings, filenames: &[T]) {
    let mut file = File::create(game_settings.load_order_file().unwrap()).unwrap();

    for filename in filenames {
        writeln!(file, "{}", filename).unwrap();
    }
}

fn write_active_plugins_file<T: AsRef<str>>(game_settings: &GameSettings, filenames: &[T]) {
    let mut file = File::create(game_settings.active_plugins_file()).unwrap();

    if game_settings.id() == GameId::Morrowind {
        writeln!(file, "isrealmorrowindini=false").unwrap();
        writeln!(file, "[Game Files]").unwrap();
    }

    for filename in filenames {
        if game_settings.id() == GameId::Morrowind {
            write!(file, "GameFile0=").unwrap();
        } else if game_settings.load_order_method() == LoadOrderMethod::Asterisk {
            write!(file, "*").unwrap();
        }
        file.write_all(&WINDOWS_1252.encode(filename.as_ref()).0)
            .unwrap();
        writeln!(file).unwrap();
    }
}

fn set_timestamps<T: AsRef<str>>(plugins_directory: &Path, filenames: &[T]) {
    for (index, filename) in filenames.iter().enumerate() {
        let times = FileTimes::new()
            .set_accessed(SystemTime::UNIX_EPOCH)
            .set_modified(
                SystemTime::UNIX_EPOCH + Duration::from_secs(u64::try_from(index).unwrap()),
            );
        File::options()
            .write(true)
            .open(plugins_directory.join(filename.as_ref()))
            .unwrap()
            .set_times(times)
            .unwrap();
    }
}

fn testing_plugins_dir(game_id: GameId) -> PathBuf {
    let game_folder = match game_id {
        GameId::Morrowind => "Morrowind",
        GameId::Oblivion => "Oblivion",
        _ => "Skyrim",
    };

    let plugins_folder = match game_id {
        GameId::Morrowind => "Data Files",
        _ => "Data",
    };

    Path::new("testing-plugins")
        .join(game_folder)
        .join(plugins_folder)
}

fn copy_to_test_dir(from_path: &str, to_file: &str, game_settings: &GameSettings) {
    let testing_plugins_dir = testing_plugins_dir(game_settings.id());
    let data_dir = game_settings.plugins_directory();
    if !data_dir.exists() {
        create_dir(&data_dir).unwrap();
    }
    copy(testing_plugins_dir.join(from_path), data_dir.join(to_file)).unwrap();
}

fn initialise_state(game_settings: &GameSettings, plugins_count: u16, active_plugins_count: u16) {
    let mut plugins: Vec<String> = Vec::new();

    // Make 10% of the load order master files.
    let masters_count = plugins_count / 10;

    for i in 0..masters_count {
        plugins.push(format!("Blank{}.esm", i));
        copy_to_test_dir(
            "Blank - Different.esm",
            plugins.last().unwrap(),
            game_settings,
        );
    }

    for i in masters_count..plugins_count + 1 {
        plugins.push(format!("Blank{}.esp", i));
        copy_to_test_dir("Blank.esp", plugins.last().unwrap(), game_settings);
    }

    let mut plugins_as_ref: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();
    if game_settings.load_order_file().is_some() {
        write_load_order_file(game_settings, &plugins_as_ref);
    }
    set_timestamps(&game_settings.plugins_directory(), &plugins_as_ref);
    plugins_as_ref.truncate(active_plugins_count.into());
    write_active_plugins_file(game_settings, &plugins_as_ref);
}

fn to_owned(strs: Vec<&str>) -> Vec<String> {
    strs.into_iter().map(String::from).collect()
}

#[derive(Clone)]
struct Parameters {
    settings: GameSettings,
    plugins_count: u16,
    active_plugins_count: u16,
    _directory: Rc<TempDir>,
}

impl Parameters {
    fn new(game_id: GameId, plugins_count: u16, active_plugins_count: u16) -> Parameters {
        let directory = TempDir::new().unwrap();
        let local_path = directory.path().join("local");

        create_dir(&local_path).unwrap();

        let settings =
            GameSettings::with_local_path(game_id, directory.path(), &local_path).unwrap();

        initialise_state(&settings, plugins_count, active_plugins_count);

        Parameters {
            settings,
            plugins_count,
            active_plugins_count,
            _directory: Rc::new(directory),
        }
    }

    fn load_order(&self) -> Box<dyn WritableLoadOrder> {
        self.settings.clone().into_load_order()
    }

    fn loaded_load_order(&self) -> Box<dyn WritableLoadOrder> {
        let mut load_order = self.load_order();
        load_order.load().unwrap();

        load_order
    }
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({:?}, {} plugins, {} active)",
            self.settings.id(),
            self.plugins_count,
            self.active_plugins_count
        )
    }
}

macro_rules! parameterised_benchmark {
    ( $criterion:expr, $benchmark_name:expr, $parameters:expr, $func:expr ) => {{
        let mut group = $criterion.benchmark_group($benchmark_name);
        for parameter in &$parameters {
            group.bench_with_input(BenchmarkId::from_parameter(parameter), parameter, $func);
        }
    }};
}

fn readable_load_order_benchmark(c: &mut Criterion) {
    // ReadableLoadOrder methods are the same for all games, so just benchmark one.
    let load_orders: Vec<Parameters> = vec![
        Parameters::new(GameId::Fallout4, 20, 20),
        Parameters::new(GameId::Fallout4, 500, 250),
    ];

    parameterised_benchmark!(
        c,
        "ReadableLoadOrder.plugin_names()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.plugin_names())
        }
    );

    parameterised_benchmark!(
        c,
        "ReadableLoadOrder.index_of()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            let plugin = load_order
                .plugin_at(parameters.plugins_count.into())
                .unwrap();

            b.iter(|| load_order.index_of(plugin))
        }
    );

    parameterised_benchmark!(
        c,
        "ReadableLoadOrder.plugin_at()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.plugin_at(10))
        }
    );

    parameterised_benchmark!(
        c,
        "ReadableLoadOrder.active_plugin_names()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.active_plugin_names())
        }
    );

    parameterised_benchmark!(
        c,
        "ReadableLoadOrder.is_active()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            let plugin = load_order
                .plugin_at(parameters.plugins_count.into())
                .unwrap()
                .to_owned();

            b.iter(|| load_order.is_active(&plugin))
        }
    );
}

fn benchmarks_writable_load_order_slow(c: &mut Criterion) {
    let load_orders: Vec<Parameters> = vec![
        Parameters::new(GameId::Oblivion, 20, 20),
        Parameters::new(GameId::Oblivion, 500, 250),
        Parameters::new(GameId::Skyrim, 20, 20),
        Parameters::new(GameId::Skyrim, 500, 250),
        Parameters::new(GameId::Fallout4, 20, 20),
        Parameters::new(GameId::Fallout4, 500, 250),
        Parameters::new(GameId::Fallout4, 1500, 250),
    ];

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.load()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.load_order();

            b.iter(|| load_order.load())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.set_load_order()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugins = to_owned(load_order.plugin_names());
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_load_order(&plugin_refs).unwrap())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.save()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            b.iter(|| load_order.save().unwrap())
        }
    );
}

fn writable_load_order_benchmark(c: &mut Criterion) {
    let load_orders: Vec<Parameters> = vec![
        Parameters::new(GameId::Oblivion, 20, 20),
        Parameters::new(GameId::Oblivion, 500, 250),
        Parameters::new(GameId::Skyrim, 20, 20),
        Parameters::new(GameId::Skyrim, 500, 250),
        Parameters::new(GameId::Fallout4, 20, 20),
        Parameters::new(GameId::Fallout4, 500, 250),
    ];

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.set_plugin_index()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.set_plugin_index(&plugin_name, 10).unwrap())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.is_self_consistent()",
        load_orders,
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.is_self_consistent().unwrap())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.activate()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.activate(&plugin_name).unwrap())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.deactivate()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.deactivate(&plugin_name).unwrap())
        }
    );

    parameterised_benchmark!(
        c,
        "WritableLoadOrder.set_active_plugins()",
        load_orders,
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugins = to_owned(load_order.active_plugin_names());
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_active_plugins(&plugin_refs).unwrap())
        }
    );
}

criterion::criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = readable_load_order_benchmark, writable_load_order_benchmark
}
criterion::criterion_group! {
    name = slow_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .sample_size(25);
    targets = benchmarks_writable_load_order_slow
}
criterion::criterion_main!(benches, slow_benches);
