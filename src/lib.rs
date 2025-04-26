/*
 * This file is part of libloadorder
 *
 * Copyright (C) 2017 Oliver Hamlet
 *
 * libloadorder is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * libloadorder is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with libloadorder. If not, see <http://www.gnu.org/licenses/>.
 */
// Deny some rustc lints that are allow-by-default.
#![deny(
    ambiguous_negative_literals,
    impl_trait_overcaptures,
    let_underscore_drop,
    missing_copy_implementations,
    missing_debug_implementations,
    non_ascii_idents,
    redundant_imports,
    redundant_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unit_bindings,
    unreachable_pub,
    unsafe_code
)]
#![deny(clippy::pedantic)]
// Allow a few clippy pedantic lints.
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
// Selectively deny clippy restriction lints.
#![deny(
    clippy::allow_attributes,
    clippy::as_conversions,
    clippy::as_underscore,
    clippy::assertions_on_result_states,
    clippy::big_endian_bytes,
    clippy::cfg_not_test,
    clippy::clone_on_ref_ptr,
    clippy::create_dir,
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::default_numeric_fallback,
    clippy::doc_include_without_cfg,
    clippy::empty_drop,
    clippy::error_impl_error,
    clippy::exit,
    clippy::exhaustive_enums,
    clippy::expect_used,
    // clippy::filetype_is_file,
    clippy::float_cmp_const,
    clippy::fn_to_numeric_cast_any,
    clippy::get_unwrap,
    clippy::host_endian_bytes,
    clippy::if_then_some_else_none,
    clippy::indexing_slicing,
    clippy::infinite_loop,
    clippy::integer_division,
    clippy::integer_division_remainder_used,
    clippy::iter_over_hash_type,
    clippy::let_underscore_must_use,
    clippy::lossy_float_literal,
    clippy::map_err_ignore,
    clippy::map_with_unused_argument_over_ranges,
    clippy::mem_forget,
    clippy::missing_assert_message,
    clippy::missing_asserts_for_indexing,
    clippy::mixed_read_write_in_expression,
    clippy::multiple_inherent_impl,
    clippy::multiple_unsafe_ops_per_block,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::needless_raw_strings,
    clippy::non_ascii_literal,
    clippy::non_zero_suggestions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::partial_pub_fields,
    clippy::pathbuf_init_then_push,
    clippy::precedence_bits,
    clippy::print_stderr,
    clippy::print_stdout,
    clippy::rc_buffer,
    clippy::rc_mutex,
    clippy::redundant_type_annotations,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::str_to_string,
    clippy::string_lit_chars_any,
    clippy::string_slice,
    clippy::string_to_string,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::todo,
    clippy::try_err,
    clippy::undocumented_unsafe_blocks,
    clippy::unimplemented,
    clippy::unnecessary_safety_comment,
    clippy::unneeded_field_pattern,
    clippy::unreachable,
    clippy::unused_result_ok,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::use_debug,
    clippy::verbose_file_reads,
)]
#![cfg_attr(
    test,
    allow(
        clippy::assertions_on_result_states,
        clippy::indexing_slicing,
        clippy::panic,
        clippy::unwrap_used,
    )
)]

mod enums;
mod game_settings;
mod ghostable_path;
mod ini;
mod load_order;
mod openmw_config;
mod plugin;
#[cfg(test)]
mod tests;

pub use crate::enums::{Error, GameId, LoadOrderMethod};
pub use crate::game_settings::GameSettings;
pub use crate::load_order::{ReadableLoadOrder, WritableLoadOrder};

fn is_enderal(game_path: &std::path::Path) -> bool {
    game_path.join("Enderal Launcher.exe").exists()
}
