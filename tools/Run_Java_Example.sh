#!/bin/bash
user_dir=/Users/mbryant
jna_package=$user_dir/Desktop/jnatest-master/jna-4.2.1.jar
target_dir=$user_dir/Desktop/metrics_controller/target/debug/
examples_dir=$user_dir/Desktop/metrics_controller/examples/

javac -cp $jna_package examples/JavaMetrics.java
CD_METRICS_LOG=debug java -cp $examples_dir:$jna_package:$target_dir JavaMetrics
