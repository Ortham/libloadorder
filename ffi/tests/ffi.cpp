#include <cassert>
#include <cstdbool>
#include <cstdio>
#include <cstdint>
#include <cstring>

#include <thread>
#include <vector>

#include "libloadorder.hpp"

void test_game_id_values() {
  printf("testing LIBLO_GAME_* values...\n");
  assert(LIBLO_GAME_TES3 == 1);
  assert(LIBLO_GAME_TES4 == 2);
  assert(LIBLO_GAME_TES5 == 3);
  assert(LIBLO_GAME_FO3 == 4);
  assert(LIBLO_GAME_FNV == 5);
  assert(LIBLO_GAME_FO4 == 6);
  assert(LIBLO_GAME_TES5SE == 7);
}

void test_lo_get_version() {
  printf("testing lo_get_version()...\n");
  unsigned int major;
  unsigned int minor;
  unsigned int patch;
  unsigned int return_code = lo_get_version(&major, &minor, &patch);

  assert(return_code == 0);
  assert(major == 10);
  assert(minor == 0);
  assert(patch == 0);
}

void test_lo_get_error_message() {
  printf("testing lo_get_error_message()...\n");
  const char * message = NULL;
  unsigned int return_code = lo_get_error_message(&message);

  assert(return_code == 0);
  assert(message == NULL);

  message = NULL;
  return_code = lo_get_version(NULL, NULL, NULL);
  assert(return_code == LIBLO_ERROR_INVALID_ARGS);

  return_code = lo_get_error_message(&message);
  assert(return_code == 0);
  assert(message != NULL);
  assert(strcmp(message, "Null pointer(s) passed") == 0);
}

void test_lo_free_string() {
  printf("testing lo_free_string()...\n");
  char * plugin = NULL;
  lo_free_string(plugin);
}

void test_lo_free_string_array() {
  printf("testing lo_free_string_array()...\n");
  char ** plugins = NULL;
  lo_free_string_array(plugins, 0);
}

lo_game_handle create_handle() {
  lo_game_handle handle = NULL;
  unsigned int return_code = lo_create_handle(&handle,
    LIBLO_GAME_TES4,
    "../../testing-plugins/Oblivion",
    "../../testing-plugins/Oblivion");

  assert(return_code == 0);
  assert(handle != NULL);

  return handle;
}

void test_lo_create_handle() {
  printf("testing lo_create_handle()...\n");
  lo_game_handle handle = create_handle();

  lo_destroy_handle(handle);
}

void test_lo_fix_plugin_lists() {
  printf("testing lo_fix_plugin_list()...\n");
  lo_game_handle handle = create_handle();
  unsigned int return_code = lo_fix_plugin_lists(handle);

  assert(return_code == 0);
  lo_destroy_handle(handle);
}

void test_lo_set_active_plugins() {
  printf("testing lo_set_active_plugins()...\n");
  lo_game_handle handle = create_handle();

  const char * plugins[] = { "Blank.esm" };
  unsigned int return_code = lo_set_active_plugins(handle, plugins, 1);

  assert(return_code == 0);
  lo_destroy_handle(handle);
}

void test_lo_get_active_plugins() {
  printf("testing lo_fix_plugin_list()...\n");
  lo_game_handle handle = create_handle();

  char ** plugins = NULL;
  size_t num_plugins = 0;
  unsigned int return_code = lo_get_active_plugins(handle, &plugins, &num_plugins);

  assert(return_code == 0);
  assert(num_plugins == 1);
  assert(strcmp(plugins[0], "Blank.esm") == 0);
  lo_free_string_array(plugins, num_plugins);
  lo_destroy_handle(handle);
}

void test_lo_set_plugin_active() {
  printf("testing lo_set_plugin_active()...\n");
  lo_game_handle handle = create_handle();

  unsigned int return_code = lo_set_plugin_active(handle, "Blank.esm", false);

  assert(return_code == 0);
  lo_destroy_handle(handle);
}

void test_lo_get_plugin_active() {
  printf("testing lo_get_plugin_active()...\n");
  lo_game_handle handle = create_handle();

  bool is_active = true;
  unsigned int return_code = lo_get_plugin_active(handle, "Blank.esm", &is_active);

  assert(return_code == 0);
  assert(!is_active);
  lo_destroy_handle(handle);
}

void test_lo_get_load_order_method() {
  printf("testing lo_get_load_order_method()...\n");
  lo_game_handle handle = create_handle();

  unsigned int method = 10;
  unsigned int return_code = lo_get_load_order_method(handle, &method);

  assert(return_code == 0);
  assert(method == LIBLO_METHOD_TIMESTAMP);
  lo_destroy_handle(handle);
}

void test_lo_set_load_order() {
  printf("testing lo_set_load_order()...\n");
  lo_game_handle handle = create_handle();

  const char * plugins[] = {
    "Blank.esm",
    "Blank - Different.esm",
    "Blank - Master Dependent.esm",
    "Blank - Different Master Dependent.esm",
    "Blank.esp",
    "Blank - Different.esp",
  };
  size_t num_plugins = 6;
  unsigned int return_code = lo_set_load_order(handle, plugins, num_plugins);

  assert(return_code == 0);
  lo_destroy_handle(handle);
}

void test_lo_get_load_order() {
  printf("testing lo_get_load_order()...\n");
  lo_game_handle handle = create_handle();

  char ** plugins = NULL;
  size_t num_plugins = 0;
  unsigned int return_code = lo_get_load_order(handle, &plugins, &num_plugins);

  assert(return_code == 0);
  assert(num_plugins == 10);
  assert(strcmp(plugins[0], "Blank.esm") == 0);
  assert(strcmp(plugins[4], "Blank.esp") == 0);
  lo_free_string_array(plugins, num_plugins);
  lo_destroy_handle(handle);
}

void test_lo_set_plugin_position() {
  printf("testing lo_set_plugin_position()...\n");
  lo_game_handle handle = create_handle();

  unsigned int return_code = lo_set_plugin_position(handle, "Blank.esp", 7);

  assert(return_code == 0);
  lo_destroy_handle(handle);
}

void test_lo_get_plugin_position() {
  printf("testing lo_get_plugin_position()...\n");
  lo_game_handle handle = create_handle();

  size_t position = 0;
  unsigned int return_code = lo_get_plugin_position(handle, "Blank.esp", &position);

  assert(return_code == 0);
  assert(position == 7);
  lo_destroy_handle(handle);
}

void test_lo_get_indexed_plugin() {
  printf("testing lo_get_indexed_plugin()...\n");
  lo_game_handle handle = create_handle();

  char * plugin = NULL;
  unsigned int return_code = lo_get_indexed_plugin(handle, 0, &plugin);

  assert(return_code == 0);
  assert(strcmp(plugin, "Blank.esm") == 0);
  lo_free_string(plugin);
  lo_destroy_handle(handle);
}

void test_thread_safety() {
  printf("testing test_thread_safety()...\n");
  lo_game_handle handle = create_handle();

  std::vector<std::thread> threads;
  for (int i = 0; i < 30; ++i) {
    threads.push_back(std::thread([&](){
      bool is_active = true;
      unsigned int return_code = lo_get_plugin_active(handle, "Blank.esm", &is_active);

      assert(return_code == 0);
      assert(!is_active);
    }));
  }

  for (auto& thread : threads) {
    thread.join();
  }

  lo_destroy_handle(handle);
}

int main(void) {
  test_game_id_values();

  test_lo_get_error_message();
  test_lo_free_string();
  test_lo_free_string_array();

  test_lo_create_handle();
  test_lo_fix_plugin_lists();

  test_lo_set_active_plugins();
  test_lo_get_active_plugins();
  test_lo_set_plugin_active();
  test_lo_get_plugin_active();

  test_lo_get_load_order_method();
  test_lo_set_load_order();
  test_lo_get_load_order();
  test_lo_set_plugin_position();
  test_lo_get_plugin_position();
  test_lo_get_indexed_plugin();

  test_thread_safety();

  remove("testing-plugins/Oblivion/plugins.txt");
  printf("SUCCESS\n");
  return 0;
}
