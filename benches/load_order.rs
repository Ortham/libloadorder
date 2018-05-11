#[macro_use]
extern crate criterion;
extern crate encoding;
extern crate filetime;
extern crate loadorder;
extern crate tempfile;

use std::fmt;
use std::fmt::Display;
use std::fs::{copy, create_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use criterion::Criterion;
use encoding::all::WINDOWS_1252;
use encoding::{EncoderTrap, Encoding};
use filetime::{set_file_times, FileTime};
use tempfile::TempDir;

use loadorder::GameId;
use loadorder::GameSettings;
use loadorder::LoadOrderMethod;
use loadorder::WritableLoadOrder;

fn write_load_order_file<T: AsRef<str> + Display>(game_settings: &GameSettings, filenames: &[T]) {
    let mut file = File::create(&game_settings.load_order_file().unwrap()).unwrap();

    for filename in filenames {
        writeln!(file, "{}", filename).unwrap();
    }
}

fn write_active_plugins_file<T: AsRef<str>>(game_settings: &GameSettings, filenames: &[T]) {
    let mut file = File::create(&game_settings.active_plugins_file()).unwrap();

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
        file.write_all(&WINDOWS_1252
            .encode(filename.as_ref(), EncoderTrap::Strict)
            .unwrap())
            .unwrap();
        writeln!(file, "").unwrap();
    }
}

fn set_timestamps<T: AsRef<str>>(plugins_directory: &Path, filenames: &[T]) {
    for (index, filename) in filenames.iter().enumerate() {
        set_file_times(
            &plugins_directory.join(filename.as_ref()),
            FileTime::zero(),
            FileTime::from_unix_time(index as i64, 0),
        ).unwrap();
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

    plugins.push(game_settings.master_file().to_string());
    copy_to_test_dir("Blank.esm", game_settings.master_file(), game_settings);

    for i in 0..plugins_count {
        plugins.push(format!("Blank{}.esm", i));
        copy_to_test_dir(
            "Blank - Different.esm",
            plugins.last().unwrap(),
            game_settings,
        );
    }

    let mut plugins_as_ref: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();
    if game_settings.load_order_file().is_some() {
        write_load_order_file(game_settings, &plugins_as_ref);
    }
    set_timestamps(&game_settings.plugins_directory(), &plugins_as_ref);
    plugins_as_ref.truncate(active_plugins_count as usize);
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
    directory: Rc<TempDir>,
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
            directory: Rc::new(directory),
        }
    }

    fn load_order(&self) -> Box<WritableLoadOrder> {
        self.settings.clone().into_load_order()
    }

    fn loaded_load_order(&self) -> Box<WritableLoadOrder> {
        let mut load_order = self.load_order();
        load_order.load().unwrap();

        load_order
    }
}

impl fmt::Debug for Parameters {
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

fn readable_load_order_benchmark(c: &mut Criterion) {
    // ReadableLoadOrder methods are the same for all games, so just benchmark one.
    let load_orders: Vec<Parameters> = vec![
        Parameters::new(GameId::Fallout4, 20, 20),
        Parameters::new(GameId::Fallout4, 500, 250),
    ];

    c.bench_function_over_inputs(
        "ReadableLoadOrder.plugin_names()",
        |b, parameters| {
            let mut load_order = parameters.load_order();

            load_order.load().unwrap();

            b.iter(|| load_order.plugin_names())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.index_of()",
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            let plugin = load_order
                .plugin_at(parameters.plugins_count as usize)
                .unwrap();

            b.iter(|| load_order.index_of(plugin))
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.plugin_at()",
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.plugin_at(10))
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.active_plugin_names()",
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.active_plugin_names())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.is_active()",
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            let plugin = load_order
                .plugin_at(parameters.plugins_count as usize)
                .unwrap()
                .to_owned();

            b.iter(|| load_order.is_active(&plugin))
        },
        load_orders.clone(),
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
    ];

    c.bench_function_over_inputs(
        "WritableLoadOrder.load()",
        |b, parameters| {
            let mut load_order = parameters.load_order();

            b.iter(|| load_order.load())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_load_order()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugins = to_owned(load_order.plugin_names());
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_load_order(&plugin_refs).unwrap())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.save()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            b.iter(|| load_order.save().unwrap())
        },
        load_orders.clone(),
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

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_plugin_index()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.set_plugin_index(&plugin_name, 10).unwrap())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.is_self_consistent()",
        |b, parameters| {
            let load_order = parameters.loaded_load_order();

            b.iter(|| load_order.is_self_consistent().unwrap())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.activate()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.activate(&plugin_name).unwrap())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.deactivate()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.deactivate(&plugin_name).unwrap())
        },
        load_orders.clone(),
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_active_plugins()",
        |b, parameters| {
            let mut load_order = parameters.loaded_load_order();

            let plugins = to_owned(load_order.active_plugin_names());
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_active_plugins(&plugin_refs).unwrap())
        },
        load_orders.clone(),
    );
}

criterion_group!{
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = readable_load_order_benchmark, writable_load_order_benchmark
}
criterion_group!{
    name = slow_benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(2))
        .sample_size(25);
    targets = benchmarks_writable_load_order_slow
}
criterion_main!(benches, slow_benches);
