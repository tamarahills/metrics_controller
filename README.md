# metrics_controller

[![Build Status](https://travis-ci.org/tamarahills/metrics_controller.svg?branch=master)](https://travis-ci.org/tamarahills/metrics_controller)

* |cargo test| -- builds the library and the main target for testing, runs unit tests
* |RUST_TEST_THREADS=1 cargo test --features integration| -- builds the library and the main target for testing, runs integration tests in serial
* |cargo clean| -- cleans out the target
* |cargo build| -- builds just the library
* |cargo doc| -- creates 'rustdoc' documentation

## Toolchain
 run |multirust override nightly-2016-05-31| to make sure you have the correct rust version.

## Extra steps for Mac OS X

The metrics lib requires an up-to-date openssl library. In order to make sure you have the correct library, we recommend you install brew and run:

``` bash
brew install openssl
source tools/mac-os-x-setup.source.sh
```

## Raspberry Pi
To cross-compile the metrics library for Raspberry Pi:

``` bash
docker run -it russnicoletti/metrics_controller-pi
cd metrics_controller
cargopi build
```

## Logging
 The metrics library uses the `env_logger` package for logging functionality. Two notable features of this package
are:
* The log messages are written to stderror
* An environment variable is used to determine the log level

The environment variable specifying the log level is `CD_METRICS_LOG`. For example, to set the log level to `info` when running the example program, the command line would be:

    CD_METRICS_LOG=info target/debug/examples/main

This will enable info-level logging for the metrics library and all rust modules used by the metrics library. To limit the logging to metrics library messages, use the `env_logger` filtering mechanism and specify the prefix included in metrics library log messages -- `CD-METRICS`:

    CD_METRICS_LOG=info/CD-METRICS target/debug/examples/main

And, of course, to redirect the log messages to a file:

    CD_METRICS_LOG=info/CD-METRICS target/debug/examples/main 2> log

## Javascript Wrapper
  There is a Javascript wrapper called js/metrics.js that will facilitate metrics gathering for web based applications.

  To invoke the integration test:
    1.  Install chai: |npm install chai|
    2.  Install mocha: |npm install mocha|
    3.  Load test/metrics_test.html in a browser.  

  Note that the test may take up to 3 minutes to execute as it's verifying the data was received at the server.

## C interface
There is a C interface that can be used from C and Java applications.  

To utilize this:
  1.  Run |cargo build|
  2.  This will create a target under ./target/debug/libmetrics_controller.dylib (Mac), .so (Linux), or .dll (Windows).
  3.  Refer to ./examples/ffi_test.c for an example of how to invoke the library and for instructions to run
  the sample.
