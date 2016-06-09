#!/bin/bash
user_dir= /home/mermi
jna_package=$user_dir/metrics/jna-4.2.2.jar
target_dir=$user_dir/metrics/metrics_controller/target/debug/
examples_dir=$user_dir/metrics/metrics_controller/examples/

javac -cp $jna_package examples/JavaMetrics.java
#CD_METRICS_LOG=debug java -cp $examples_dir:$jna_package:$target_dir JavaMetrics
