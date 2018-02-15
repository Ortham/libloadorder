#[macro_use]
extern crate criterion;
extern crate encoding;
extern crate filetime;
extern crate loadorder;
extern crate tempdir;

use std::fmt::Display;
use std::fs::{copy, create_dir, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use criterion::Criterion;
use encoding::{EncoderTrap, Encoding};
use encoding::all::WINDOWS_1252;
use filetime::{set_file_times, FileTime};
use tempdir::TempDir;

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
            FileTime::from_seconds_since_1970(index as u64, 0),
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

fn game_settings(game_id: GameId, game_dir: &Path) -> GameSettings {
    let local_path = game_dir.join("local");
    create_dir(&local_path).unwrap();
    GameSettings::with_local_path(game_id, game_dir, &local_path).unwrap()
}

fn initialise_state(game_settings: &GameSettings, plugins_count: u16, active_plugins_count: u16) {
    let mut plugins: Vec<String> = Vec::new();

    plugins.push(game_settings.master_file().to_string());
    copy_to_test_dir("Blank.esm", game_settings.master_file(), &game_settings);

    for i in 0..plugins_count {
        plugins.push(format!("Blank{}.esm", i));
        copy_to_test_dir(
            "Blank - Different.esm",
            &plugins.last().unwrap(),
            &game_settings,
        );
    }

    let mut plugins_as_ref: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();
    if game_settings.load_order_file().is_some() {
        write_load_order_file(&game_settings, &plugins_as_ref);
    }
    set_timestamps(&game_settings.plugins_directory(), &plugins_as_ref);
    plugins_as_ref.truncate(active_plugins_count as usize);
    write_active_plugins_file(&game_settings, &plugins_as_ref);
}

fn prepare(
    game_id: GameId,
    game_dir: &Path,
    plugins_count: u16,
    active_plugins_count: u16,
) -> Box<WritableLoadOrder> {
    let game_settings = game_settings(game_id, game_dir);

    initialise_state(&game_settings, plugins_count, active_plugins_count);

    game_settings.into_load_order()
}

fn readable_load_order_benchmark(c: &mut Criterion) {
    // ReadableLoadOrder methods are the same for all games, so just benchmark one.
    const LOAD_ORDERS: &[(u16, u16)] = &[(20, 20), (500, 250)];

    c.bench_function_over_inputs(
        "ReadableLoadOrder.plugin_names()",
        |b, &&(plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                GameId::Fallout4,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();

            b.iter(|| load_order.plugin_names())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.index_of()",
        |b, &&(plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                GameId::Fallout4,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugin = load_order.plugin_at(plugins_count as usize).unwrap();

            b.iter(|| load_order.index_of(&plugin))
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.plugin_at()",
        |b, &&(plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                GameId::Fallout4,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();

            b.iter(|| load_order.plugin_at(10))
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.active_plugin_names()",
        |b, &&(plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                GameId::Fallout4,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();

            b.iter(|| load_order.active_plugin_names())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "ReadableLoadOrder.is_active()",
        |b, &&(plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                GameId::Fallout4,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugin = load_order.plugin_at(plugins_count as usize).unwrap();

            b.iter(|| load_order.is_active(&plugin))
        },
        LOAD_ORDERS,
    );
}

fn writable_load_order_benchmark(c: &mut Criterion) {
    const LOAD_ORDERS: &[(GameId, u16, u16)] = &[
        (GameId::Fallout4, 20, 20),
        (GameId::Fallout4, 500, 250),
        (GameId::Skyrim, 20, 20),
        (GameId::Skyrim, 500, 250),
        (GameId::Oblivion, 20, 20),
        (GameId::Oblivion, 500, 250),
    ];

    c.bench_function_over_inputs(
        "WritableLoadOrder.load()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            b.iter(|| load_order.load().unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_load_order()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugins = load_order.plugin_names();
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_load_order(&plugin_refs).unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_plugin_index()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.set_plugin_index(&plugin_name, 10).unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.is_self_consistent()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();

            b.iter(|| load_order.is_self_consistent().unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.activate()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.activate(&plugin_name).unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.deactivate()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugin_name = load_order.plugin_at(5).unwrap().to_string();

            b.iter(|| load_order.deactivate(&plugin_name).unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.set_active_plugins()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();
            let plugins = load_order.active_plugin_names();
            let plugin_refs: Vec<&str> = plugins.iter().map(AsRef::as_ref).collect();

            b.iter(|| load_order.set_active_plugins(&plugin_refs).unwrap())
        },
        LOAD_ORDERS,
    );

    c.bench_function_over_inputs(
        "WritableLoadOrder.save()",
        |b, &&(game_id, plugins_count, active_plugins_count)| {
            let tmp_dir = TempDir::new("libloadorder_test_").unwrap();
            let mut load_order = prepare(
                game_id,
                &tmp_dir.path(),
                plugins_count,
                active_plugins_count,
            );

            load_order.load().unwrap();

            b.iter(|| load_order.save().unwrap())
        },
        LOAD_ORDERS,
    );
}

criterion_group!(
    benches,
    readable_load_order_benchmark,
    writable_load_order_benchmark
);
criterion_main!(benches);
