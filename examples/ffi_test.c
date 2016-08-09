/*
 * This is a sample that shows how to invoke the metrics library from a C application.
 * To run this sample on MacOS:
 * 1.  cargo build
 * 2.  gcc ./examples/ffi_test.c -L ./target/debug/ -lmetrics_controller -o ffitest
 * 3.  LD_LIBRARY_PATH=./target/debug ./ffitest
 */

#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

void init_metrics(const char* app_name,
                     const char* app_version,
                     const char* app_update_channel,
                     const char* app_platform,
                     const char* locale,
                     const char* device,
                     const char* arch,
                     const char* os,
                     const char* os_version);
int record_event(const char* category, const char* action,
                     const char* label, int value);
int record_floating_point_event(const char* category, const char* action,
                                   const char* label, float value);

int main() {
    init_metrics("myapp",
                 "1.0",
                 "default",
                 "c",
                 "en-us",
                 "pi",
                 "LAMP",
                 "linux",
                 "redhat");

    for(int i = 0; i < 21; i++) {
      record_event("test", "click", "order", i);
      record_floating_point_event("test", "click", "order", i * .1);
      sleep(1);
    }
    sleep(45);
    return 0;
}
