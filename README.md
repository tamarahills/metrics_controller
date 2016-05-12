# metrics_controller

[![Build Status](https://travis-ci.org/tamarahills/metrics_controller.svg?branch=master)](https://travis-ci.org/tamarahills/metrics_controller)

* |cargo test| -- builds the library and the main target for testing, runs unit tests
* |RUST_TEST_THREADS=1 cargo test --features integration| -- builds the library and the main target for testing, runs integration tests in serial
* |cargo clean| -- cleans out the target
* |cargo build| -- builds just the library
* |cargo doc| -- creates 'rustdoc' documentation

## Toolchain
 run |multirust override nightly-2016-05-07| to make sure you have the correct rust version.

### Building on OSX El Capitan
When building from a clean state on MacOS El Capitan, if you see the following error:

    failed to run custom build command for `openssl v0.7.6`
     ...
    src/c_helpers.c:1:10: fatal error: 'openssl/ssl.h' file not found

export the following environment variables pointing to your openssl sdk. For example, if you have used homebrew to install openssl:

    export OPENSSL_INCLUDE_DIR=/usr/local/Cellar/openssl/1.0.2f/include/
    export DEP_OPENSSL_INCLUDE=/usr/local/Cellar/openssl/1.0.2f/include/

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
