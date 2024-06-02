# voxel-raymarcher-rust
voxel-raymarcher-rust

## hahahaha
sudo sh -c 'echo 1 > /proc/sys/kernel/perf_event_paranoid'
perf record --call-graph dwarf ./target/release/voxel-raymarcher-rust
perf report
perf script | inferno-collapse-perf > out.perf-folded
inferno-flamegraph < out.perf-folded > flamegraph.svg